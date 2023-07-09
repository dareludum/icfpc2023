use std::todo;

use log::debug;
use rand::Rng;

use crate::{
    common::{Grid, GridLocation},
    dto::SolutionDto,
};

use super::{Problem, Score, Solver};

#[derive(Default, Clone)]
pub struct Annealer {
    problem: Problem,
    grid: Grid,
    placements: Vec<GridLocation>,
    curr_score: Score,
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
        location: GridLocation,
    },
}

fn neighbor(
    problem: &Problem,
    grid: &Grid,
    placements: &[GridLocation],
    musician_i: usize,
    temperature: usize,
) -> Change {
    let mut rng = rand::thread_rng();
    let musician = placements[musician_i].clone();
    let temperature = temperature as isize;

    let horizontal_moves = rng.gen_range(0..=temperature) * if rng.gen_bool(0.5) { 1 } else { -1 };
    let vertical_moves =
        (temperature - horizontal_moves.abs()) * if rng.gen_bool(0.5) { 1 } else { -1 };

    let new_x = musician.x as isize + horizontal_moves;
    let new_y = musician.y as isize + vertical_moves;

    // bouncing
    let new_location = GridLocation {
        x: if new_x < 0 {
            (-new_x) as usize
        } else if new_x >= grid.width as isize {
            (grid.width as isize - (new_x - grid.width as isize + 1)) as usize
        } else {
            new_x as usize
        },
        y: if new_y < 0 {
            (-new_y) as usize
        } else if new_y >= grid.height as isize {
            (grid.height as isize - (new_y - grid.height as isize + 1)) as usize
        } else {
            new_y as usize
        },
    };

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

    fn get_grid(&self) -> Option<&Grid> {
        Some(&self.grid)
    }

    fn initialize(&mut self, problem: &Problem, solution: SolutionDto) {
        // NOTE: This can be changed
        assert!(
            solution.placements.is_empty(),
            "expand: must be the start of the chain"
        );
        self.problem = problem.clone();
        self.grid = Grid::new(&self.problem);

        let stride = self.grid.positions.len() / self.problem.data.musicians.len();
        //for (idx, _) in (0..self.grid.positions.len())
        //    .step_by(stride)
        //    .zip(0..self.problem.data.musicians.len())
        //{
        //    self.placements.push(self.grid.positions[idx].p);
        //}

        // mark out positions which cannot currently be used, because they are too close to existing ones
        //for placement in &self.placements {
        //    for pos in self.grid.positions.iter_mut() {
        //        let dist2 = distance2(pos, placement);
        //        if dist2 <= 100.0 {
        //            pos.taken = true;
        //        }
        //    }
        //}

        // self.curr_score = crate::scorer::score(&self.problem.data, &self.placements);
        debug!("expand({}): initialized", self.problem.id);
    }

    fn solve_step(&mut self) -> (SolutionDto, bool) {
        todo!()
    }
}
