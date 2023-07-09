// TODO: Remove this
#![allow(dead_code, unused_variables)]

use std::todo;

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

    pub initial_temperature: f32,
    pub temperature: f32,
    pub cooling_rate: f32,
}

enum Change {
    Swap {
        musician_a: usize,
        musician_b: usize,
    },
    Move {
        musician: usize,
        location: GridCoord,
    },
}

impl Annealer {
    fn compute_score(&self) -> Score {
        let mut res = vec![];
        for coord in &self.placements {
            res.push(self.grid_transform.apply(coord).into());
        }
        crate::scoring::new_scorer::new_score(&self.problem.data, &res, None)
    }

    fn apply(&mut self, change: &Change) {
        match change {
            Change::Swap {
                musician_a,
                musician_b,
            } => {}
            Change::Move { musician, location } => {}
        }
    }
}

fn neighbor(
    problem: &Problem,
    grid: &DiamondGrid<Option<usize>>,
    placements: &[GridCoord],
    musician_i: usize,
    temperature: usize,
) -> Change {
    let mut rng = rand::thread_rng();
    let musician = &placements[musician_i];
    let temperature = temperature as isize;

    let horizontal_moves = rng.gen_range(0..=temperature) * if rng.gen_bool(0.5) { 1 } else { -1 };
    let vertical_moves =
        (temperature - horizontal_moves.abs()) * if rng.gen_bool(0.5) { 1 } else { -1 };

    let new_x = (musician.x as isize + horizontal_moves).rem_euclid(grid.size.width() as isize);
    let new_y = (musician.y as isize + vertical_moves).rem_euclid(grid.size.height() as isize);

    let new_location = GridCoord::new(new_x, new_y);
    let existing_musician = placements.iter().position(|p| *p == new_location);

    if let Some(musician_b) = existing_musician {
        return Change::Swap {
            musician_a: musician_i,
            musician_b,
        };
    }

    Change::Move {
        musician: musician_i,
        location: new_location,
    }
}

impl Solver for Annealer {
    fn name(&self) -> String {
        "expand".to_owned()
    }

    fn initialize(&mut self, problem: &Problem, solution: SolutionDto) {
        // NOTE: This can be changed
        assert!(
            solution.placements.is_empty(),
            "expand: must be the start of the chain"
        );
        self.problem = problem.clone();

        (self.grid_size, self.grid_transform) = fit_circles_grid(
            problem.data.stage_bottom_left,
            problem.data.stage_width,
            problem.data.stage_height,
            5.002,
        );
        self.grid = DiamondGrid::new(self.grid_size, |_| None);

        // figure out an initial placement for muscians
        let mut placement = self.grid_size.all_grid_coordinates();
        let (random_placement, _) = (&mut placement[..])
            .partial_shuffle(&mut rand::thread_rng(), problem.data.musicians.len());
        for (i, placement) in random_placement.iter().enumerate() {
            self.placements.push(*placement);
            self.grid[placement] = Some(i);
        }

        // compute the score
        self.score = self.compute_score();

        // figure out the initial temperature
        let grid_width = self.grid_size.width();
        let grid_height = self.grid_size.height();
        self.initial_temperature = ((grid_width.pow(2) + grid_width.pow(2)) as f32).sqrt() / 3.;
        self.temperature = self.initial_temperature;
        self.cooling_rate = 0.9;
        debug!("annealer({}): initialized", self.problem.id);
    }

    fn solve_step(&mut self) -> (SolutionDto, bool) {
        todo!()
    }
}
