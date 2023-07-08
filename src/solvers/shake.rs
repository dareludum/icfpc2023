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
    orig_score: Score,
    curr_score: Score,
    idx: usize,
    idx_change: usize,
    delta: f32,
    any_improvement_this_cycle: bool,
}

impl Solver for Shake {
    fn name(&self) -> String {
        "shake".to_owned()
    }

    fn initialize(&mut self, problem: &Problem, solution: SolutionDto) {
        assert!(
            !solution.placements.is_empty(),
            "shake({}): must not be the start of the chain",
            problem.id
        );
        self.problem = problem.clone();
        self.solution = solution;
        self.curr_score = score(&self.problem.data, &self.solution.placements);
        self.orig_score = self.curr_score;
        self.idx = 0;
        self.idx_change = 0;
        self.delta = 1.0;
        self.any_improvement_this_cycle = false;
    }

    fn solve_step(&mut self) -> (SolutionDto, bool) {
        loop {
            let changes: &[(f32, f32)] = &[
                // N
                (0.0, self.delta),
                // S
                (0.0, -self.delta),
                // E
                (self.delta, 0.0),
                // W
                (-self.delta, 0.0),
                // NE
                (self.delta, self.delta),
                // SE
                (self.delta, -self.delta),
                // NW
                (-self.delta, self.delta),
                // SW
                (-self.delta, -self.delta),
            ];

            for i_pos in self.idx..self.solution.placements.len() {
                #[allow(clippy::needless_range_loop)]
                for i_change in self.idx_change..changes.len() {
                    let curr_pos = self.solution.placements[i_pos];
                    let change = changes[i_change];
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
                    debug!(
                        "shake({}): {} => {}",
                        self.problem.id, self.orig_score.0, new_score.0
                    );
                    self.curr_score = new_score;
                    self.idx = i_pos;
                    self.idx_change = i_change;
                    self.any_improvement_this_cycle = true;
                    return (self.solution.clone(), false);
                }
            }

            if self.any_improvement_this_cycle {
                debug!(
                    "shake({}): new cycle ({} => {})",
                    self.problem.id, self.orig_score.0, self.curr_score.0
                );
                self.idx = 0;
                self.idx_change = 0;
                self.any_improvement_this_cycle = false;
                continue;
            } else {
                if self.delta > 0.01 {
                    let old_delta = self.delta;
                    self.delta /= 2.0;
                    if self.delta < 0.01 {
                        self.delta = 0.01;
                    }
                    debug!(
                        "shake({}): delta {} => {}",
                        self.problem.id, old_delta, self.delta
                    );
                    continue;
                }

                debug!("shake({}): done", self.problem.id);
                self.idx = self.solution.placements.len();
                return (self.solution.clone(), true);
            }
        }
    }
}