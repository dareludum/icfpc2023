// TODO: Remove this
#![allow(dead_code, unused_variables)]

use std::time::Instant;

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
    pub start_time: Option<Instant>,
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
        assert!(!res.is_empty());
        SolutionDto {
            placements: res,
            volumes: Some(vec![10.0; self.placements.len()]),
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
    distance: usize,
) -> MusicianChange {
    let mut rng = rand::thread_rng();
    let musician = &placements[musician_i];

    let displacement = musician.random_displacement(&mut rng, distance);
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
// pareto distribution is really biased towards mean
fn pareto(alpha: f64, xmin: f64) -> f64 {
    let u: f64 = rand::random::<f64>();
    xmin * (1.0 / u).powf(1.0 / alpha)
}

// cauchy distribution can generate negative numbers and 0, so use with abs() and max(1)
fn _cauchy(loc: f64, scale: f64) -> f64 {
    let u: f64 = rand::random::<f64>();
    loc + scale * (u - 0.5).tan()
}

/// x between 0 and 1, starts at 0
fn cooling_cycle(x: f32) -> f32 {
    let x_sq = (x - 1.).powi(2);
    let x_cube = x_sq.powi(2);
    let plateau = (-0.5 * ((x - 0.7) / 0.2).powi(2)).exp() / 5.;
    let quench_size = 0.1f32;
    let res = (1. / (1. - quench_size)) * ((x_sq + x_cube) / 2. + plateau - quench_size);
    if res < 0. {
        0.
    } else {
        res
    }
}

fn acceptance_probability(score_delta: i64, raw_temperature: f32) -> f32 {
    let score_scale = 1. / 1_000_000f32;
    if raw_temperature == 0. {
        return 0.;
    }
    (score_delta as f32 * score_scale / raw_temperature).exp()
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

        let stage_width = problem.data.stage_width;
        let stage_height = problem.data.stage_height;
        assert!(stage_width >= 20.);
        assert!(stage_height >= 20.);
        let padding_x = if stage_width < 20.00002 { 5. } else { 5.002 };
        let padding_y = if stage_height < 20.00002 { 5. } else { 5.002 };
        let (corner_x, corner_y) = problem.data.stage_bottom_left;
        let width = stage_width - padding_x * 2.;
        let height = stage_height - padding_y * 2.;
        (self.grid_size, self.grid_transform) = fit_circles_grid(
            (corner_x + padding_x, corner_y + padding_y),
            width.max(0.),
            height.max(0.),
            5.002,
        );
        self.grid = DiamondGrid::new(self.grid_size, |_| None);

        // figure out an initial placement for musicians
        let mut placement = self.grid_size.all_grid_coordinates();
        let (random_placement, _) =
            placement[..].partial_shuffle(&mut rand::thread_rng(), musician_count);
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
        self.max_steps = musician_count * 500;
        self.start_time = Some(Instant::now());
        debug!(
            "annealer({}): initialized for {}",
            self.problem.id, self.max_steps
        );
    }

    fn solve_step(&mut self) -> (SolutionDto, bool) {
        let mut rng = rand::thread_rng();
        let progress = self.step_i as f32 / (self.max_steps - 1) as f32;
        let raw_temperature = cooling_cycle(progress);

        // generate a neighbor mutation
        let musician_i = rand::thread_rng().gen_range(0..self.problem.data.musicians.len());
        // distance_mean is the mean of the distance distribution, essentially the peak
        let distance_mean = raw_temperature * self.temperature_scale;
        // the less raw_temperature is, the more likely distribution is to be close to distance_mean
        let distance =
            pareto((1.0 + raw_temperature as f64).exp(), distance_mean as f64).ceil() as usize;
        let neighbor = neighbor(
            &self.problem,
            &self.grid,
            &self.placements,
            musician_i,
            distance.max(1),
        );

        let reverse_change = neighbor.apply(&mut self.placements, &mut self.grid);
        let new_solution = self.serialize();
        let new_score = self.compute_score(&new_solution);
        let score_delta = new_score.0 - self.score.0;

        let decision_stats = if score_delta > 0 {
            self.score = new_score;
            None
        } else {
            let probability = acceptance_probability(score_delta, raw_temperature);
            let take_the_loss = rng.gen_bool(probability as f64);
            // debug!("loss of {score_delta} taken {take_the_loss} prob {probability:.4}");
            if take_the_loss {
                self.score = new_score;
            } else {
                reverse_change.apply(&mut self.placements, &mut self.grid);
            }
            Some((probability, take_the_loss))
        };

        debug!(
            "annealer({}): step {:>5}   raw_temperature={:<5.3}   distance_mean={:<5.2}   distance={:<5.2}  score_delta={:<10} {:?}",
            self.problem.id, self.step_i, raw_temperature, distance_mean, distance, score_delta, decision_stats
        );

        self.step_i += 1;
        let done_steps = self.step_i >= self.max_steps;
        let elapsed = self.start_time.unwrap().elapsed();
        let timeout = elapsed.as_secs() > 60 * 20;
        (self.serialize(), done_steps || timeout)
    }
}
