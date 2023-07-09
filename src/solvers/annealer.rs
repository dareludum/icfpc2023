// TODO: Remove this
#![allow(dead_code, unused_variables)]

use log::debug;

use crate::{
    diamond_grid::{fit_circles_grid, DiamondGrid, GridCoord, GridSize, GridTransform},
    dto::SolutionDto,
};

use rand::{seq::SliceRandom, Rng};

use super::{Problem, Score, Solver};

#[derive(Default, Clone)]
pub struct Annealer {
    problem: Problem,
    grid_size: GridSize,
    grid_transform: GridTransform,
    grid: DiamondGrid<Option<usize>>,
    placements: Vec<GridCoord>,
    score: Score,

    pub temperature_scale: f32,
    pub max_steps: usize,
    pub step_i: usize,
}

#[derive(Clone)]
enum MusicianChange {
    Swap {
        musician_a: usize,
        musician_b: usize,
    },
    Move {
        musician: usize,
        location: GridCoord,
    },
}

impl MusicianChange {
    fn apply(&self, placements: &mut [GridCoord], grid: &mut DiamondGrid<Option<usize>>) -> Self {
        match self {
            MusicianChange::Swap {
                musician_a,
                musician_b,
            } => {
                let loc_a = placements[*musician_a];
                let loc_b = placements[*musician_b];
                placements[*musician_b] = loc_a;
                placements[*musician_a] = loc_b;
                grid[&loc_a] = Some(*musician_b);
                grid[&loc_b] = Some(*musician_a);
                self.clone()
            }
            MusicianChange::Move { musician, location } => {
                let old_location = placements[*musician];
                placements[*musician] = *location;
                grid[&old_location] = None;
                grid[location] = Some(*musician);
                MusicianChange::Move {
                    musician: *musician,
                    location: old_location,
                }
            }
        }
    }
}

impl Annealer {
    fn serialize(&self) -> SolutionDto {
        let mut res = vec![];
        for coord in &self.placements {
            res.push(self.grid_transform.apply(coord).into());
        }
        SolutionDto {
            placements: res,
            volumes: None,
        }
    }

    fn compute_score(&self, solution: &SolutionDto) -> Score {
        self.problem
            .score(&solution.placements, solution.volumes.as_ref())
    }
}

fn neighbor(
    problem: &Problem,
    grid: &DiamondGrid<Option<usize>>,
    placements: &[GridCoord],
    musician_i: usize,
    temperature: usize,
) -> MusicianChange {
    let mut rng = rand::thread_rng();
    let musician = &placements[musician_i];
    let temperature = temperature;

    let displacement = musician.random_displacement(&mut rng, temperature);
    let new_location = grid.size.displace(musician, displacement);

    let existing_musician = placements.iter().position(|p| *p == new_location);

    if let Some(musician_b) = existing_musician {
        return MusicianChange::Swap {
            musician_a: musician_i,
            musician_b,
        };
    }

    MusicianChange::Move {
        musician: musician_i,
        location: new_location,
    }
}

// fn temperature_exponential_decay(
//     step: usize,
//     max_steps: usize,
//     initial_temperature: f32,
//     decay_rate: f32,
// ) -> f32 {
//     initial_temperature * (-decay_rate * (step as f32 / max_steps as f32)).exp()
// }

fn pareto(alpha: f64, xmin: f64) -> f64 {
    let u: f64 = rand::random::<f64>();
    let x = xmin * (1.0 / u).powf(1.0 / alpha);
    x
}

fn _cauchy(loc: f64, scale: f64) -> f64 {
    let u: f64 = rand::random::<f64>();
    return loc + scale * (u - 0.5).tan();
}

impl Solver for Annealer {
    fn name(&self) -> String {
        "annealer".to_owned()
    }

    fn get_problem(&self) -> &Problem {
        &self.problem
    }

    fn initialize(&mut self, problem: &Problem, solution: SolutionDto) {
        // NOTE: This can be changed
        assert!(
            solution.placements.is_empty(),
            "annealer: must be the start of the chain"
        );
        self.problem = problem.clone();
        let musician_count = problem.data.musicians.len();

        let padding = 5.002;
        let (corner_x, corner_y) = problem.data.stage_bottom_left;
        (self.grid_size, self.grid_transform) = fit_circles_grid(
            (corner_x + padding, corner_y + padding),
            problem.data.stage_width - padding * 2.,
            problem.data.stage_height - padding * 2.,
            5.002,
        );
        self.grid = DiamondGrid::new(self.grid_size, |_| None);

        // figure out an initial placement for musicians
        let mut placement = self.grid_size.all_grid_coordinates();
        let (random_placement, _) =
            (&mut placement[..]).partial_shuffle(&mut rand::thread_rng(), musician_count);
        for (i, placement) in random_placement.iter().enumerate() {
            self.placements.push(*placement);
            self.grid[placement] = Some(i);
        }

        // compute the score
        self.score = self.compute_score(&self.serialize());

        // figure out the initial temperature
        let grid_width = self.grid_size.width();
        let grid_height: usize = self.grid_size.height();
        self.temperature_scale = ((grid_width.pow(2) + grid_width.pow(2)) as f32).sqrt() / 3.;
        self.max_steps = musician_count * 100;
        debug!(
            "annealer({}): initialized for {}",
            self.problem.id, self.max_steps
        );
    }

    fn solve_step(&mut self) -> (SolutionDto, bool) {
        let mut rng = rand::thread_rng();
        let raw_temperature = 1f32 - self.step_i as f32 / self.max_steps as f32;
        let scaled_temperature = (raw_temperature * self.temperature_scale).ceil() as usize;

        let temperature = pareto(
            (1.0 + raw_temperature as f64).exp(),
            scaled_temperature as f64,
        );

        debug!(
            "annealer({}): step {} raw_temperature={} scaled_temperature={} temperature={}",
            self.problem.id, self.step_i, raw_temperature, scaled_temperature, temperature
        );

        // generate a neighbor mutation
        let musician_i = rand::thread_rng().gen_range(0..self.problem.data.musicians.len());
        let neighbor = neighbor(
            &self.problem,
            &self.grid,
            &self.placements,
            musician_i,
            scaled_temperature,
        );

        let reverse_change = neighbor.apply(&mut self.placements, &mut self.grid);
        let new_solution = self.serialize();
        let new_score = self.compute_score(&new_solution);
        let score_delta = new_score.0 - self.score.0;

        if score_delta > 0 {
            self.score = new_score;
        } else {
            if rng.gen_bool(raw_temperature as f64) {
                self.score = new_score;
            } else {
                reverse_change.apply(&mut self.placements, &mut self.grid);
            }
        }

        self.step_i += 1;
        (self.serialize(), self.step_i >= self.max_steps)
    }
}
