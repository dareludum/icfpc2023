use std::path::Path;

use log::{debug, warn};

use crate::{
    dto::{Point2D, SolutionDto},
    solvers::Solution,
};

use super::{Problem, Solver};

#[derive(Default, Clone)]
pub struct LoadBest {
    problem: Problem,
    solution: SolutionDto,
    name: String,
}

impl Solver for LoadBest {
    fn name(&self) -> String {
        self.name.clone()
    }

    fn get_problem(&self) -> &Problem {
        &self.problem
    }

    fn initialize(&mut self, problem: &Problem, solution: SolutionDto) {
        assert!(
            solution.placements.is_empty(),
            "load_best({}): must be the start of the chain",
            problem.id
        );

        if let Ok((solution, meta)) = Solution::load(Path::new("./solutions/best"), problem) {
            debug!("load_best({}): score = {}", problem.id, meta.score);
            self.solution = solution.data;
            self.name = meta.solver_name;
        } else {
            warn!(
                "load_best({}): no best solution - keeping the empty one to not fail",
                problem.id
            );
            self.solution = SolutionDto {
                placements: vec![Point2D { x: 0.0, y: 0.0 }; problem.data.musicians.len()],
                volumes: None,
            };
            self.name = "invalid".to_owned();
        }
        self.problem = problem.clone();
    }

    fn solve_step(&mut self) -> (SolutionDto, bool) {
        (self.solution.clone(), true)
    }
}
