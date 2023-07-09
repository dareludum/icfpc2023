use log::debug;

use crate::dto::SolutionDto;

use super::{Problem, Solver};

#[derive(Default, Clone)]
pub struct Vol10 {
    problem: Problem,
    solution: SolutionDto,
}

impl Solver for Vol10 {
    fn name(&self) -> String {
        "vol10".to_owned()
    }

    fn initialize(&mut self, problem: &Problem, solution: SolutionDto) {
        assert!(
            !solution.placements.is_empty(),
            "vol10({}): must not be the start of the chain",
            problem.id
        );
        self.problem = problem.clone();
        self.solution = solution;
    }

    fn solve_step(&mut self) -> (SolutionDto, bool) {
        debug!("vol10({}): setting all volume to 10", self.problem.id,);

        (
            SolutionDto {
                placements: self.solution.placements.clone(),
                volumes: Some(vec![10.0; self.solution.placements.len()]),
            },
            true,
        )
    }
}
