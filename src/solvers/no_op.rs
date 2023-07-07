use crate::dto::{Placement, ProblemDto, SolutionDto};

use super::{Problem, Solver};

#[derive(Default, Clone)]
pub struct NoOp {
    problem: ProblemDto,
}

impl Solver for NoOp {
    fn name(&self) -> &'static str {
        "no_op"
    }

    fn initialize(&mut self, problem: &Problem) {
        self.problem = problem.data.clone();
    }

    fn solve_step(&self) -> (SolutionDto, bool) {
        (
            SolutionDto {
                placements: vec![Placement { x: 0.0, y: 0.0 }; self.problem.attendees.len()],
            },
            true,
        )
    }
}
