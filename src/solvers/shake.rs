use log::debug;

use crate::{
    common::calculate_invalid_positions,
    dto::{Point2D, SolutionDto},
    geometry::Coords2D,
    scorer::score,
};

use super::{Problem, Score, Solver};

#[derive(Default, Clone)]
pub struct Shake {
    problem: Problem,
    solution: SolutionDto,
    curr_score: Score,
    idx: usize,
    idx_change: usize,
    any_improvement_this_cycle: bool,
}

impl Solver for Shake {
    fn name(&self) -> String {
        "shake".to_owned()
    }

    fn initialize(&mut self, problem: &Problem, solution: SolutionDto) {
        assert!(
            !solution.placements.is_empty(),
            "shake: must not be the start of the chain"
        );
        self.problem = problem.clone();
        self.solution = solution;
        self.curr_score = score(&self.problem.data, &self.solution.placements);
        self.idx = 0;
        self.idx_change = 0;
        self.any_improvement_this_cycle = false;
    }

    fn solve_step(&mut self) -> (SolutionDto, bool) {
        const DELTA: f32 = 0.01;
        const CHANGES: &[(f32, f32)] = &[
            // N
            (0.0, DELTA),
            // S
            (0.0, -DELTA),
            // E
            (DELTA, 0.0),
            // W
            (-DELTA, 0.0),
            // NE
            (DELTA, DELTA),
            // SE
            (DELTA, -DELTA),
            // NW
            (-DELTA, DELTA),
            // SW
            (-DELTA, -DELTA),
        ];

        loop {
            for i_pos in self.idx..self.solution.placements.len() {
                #[allow(clippy::needless_range_loop)]
                for i_change in self.idx_change..CHANGES.len() {
                    let curr_pos = self.solution.placements[i_pos];
                    let change = CHANGES[i_change];
                    let new_pos = Point2D {
                        x: curr_pos.x() + change.x(),
                        y: curr_pos.y() + change.y(),
                    };
                    self.solution.placements[i_pos] = new_pos;
                    let any_invalid =
                        !calculate_invalid_positions(&self.solution.placements, &self.problem.data)
                            .is_empty();
                    if any_invalid {
                        self.solution.placements[i_pos] = curr_pos;
                        continue;
                    }
                    let new_score = score(&self.problem.data, &self.solution.placements);
                    if new_score.0 <= self.curr_score.0 {
                        self.solution.placements[i_pos] = curr_pos;
                        continue;
                    }
                    debug!("shake: {} => {}", self.curr_score.0, new_score.0);
                    self.curr_score = new_score;
                    self.idx = i_pos;
                    self.idx_change = i_change;
                    self.any_improvement_this_cycle = true;
                    return (self.solution.clone(), false);
                }
            }

            if self.any_improvement_this_cycle {
                debug!("shake: new cycle");
                self.idx = 0;
                self.idx_change = 0;
                self.any_improvement_this_cycle = false;
                continue;
            } else {
                debug!("shake: done");
                self.idx = self.solution.placements.len();
                return (self.solution.clone(), true);
            }
        }
    }
}
