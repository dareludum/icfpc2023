use std::todo;

use log::debug;
use rand::seq::SliceRandom;

use crate::{
    common::{Grid, GridLocation},
    dto::{Point2D, SolutionDto},
    geometry::distance2,
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
    placements: Vec<GridLocation>,
    musician_i: usize,
    temperature: usize,
) -> Change {
    return Change::Swap {
        musician_a: 0,
        musician_b: 0,
    };
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
