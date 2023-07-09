use log::debug;

use crate::{
    common::Grid,
    dto::{Instrument, SolutionDto},
    scoring::impact_map::ImpactMap,
};

use super::{Problem, Solver};

#[derive(Clone)]
pub struct Chain {
    solver0: Box<dyn Solver>,
    solver1: Box<dyn Solver>,
    step0: bool,
    problem: Problem,
}

impl Solver for Chain {
    fn name(&self) -> String {
        format!("{}+{}", self.solver0.name(), self.solver1.name())
    }

    fn get_impact_map(&self, instrument: &Instrument) -> Option<&ImpactMap> {
        self.get_solver().get_impact_map(instrument)
    }

    fn get_grid(&self) -> Option<&Grid> {
        self.get_solver().get_grid()
    }

    fn initialize(&mut self, problem: &Problem, solution: SolutionDto) {
        self.solver0.initialize(problem, solution);
        self.step0 = true;
        self.problem = problem.clone();
    }

    fn solve_step(&mut self) -> (SolutionDto, bool) {
        if !self.step0 {
            self.solver1.solve_step()
        } else {
            let (s, done) = self.solver0.solve_step();
            if done {
                debug!(
                    "chain({}): switching to {}",
                    self.problem.id,
                    self.solver1.name()
                );
                self.solver1.initialize(&self.problem, s.clone());
                self.step0 = false;
            }
            (s, false)
        }
    }
}

impl Chain {
    pub fn new(solver0: Box<dyn Solver>, solver1: Box<dyn Solver>) -> Self {
        Chain {
            solver0,
            solver1,
            step0: true,
            problem: Problem::default(),
        }
    }

    #[allow(clippy::borrowed_box)]
    fn get_solver(&self) -> &Box<dyn Solver> {
        if self.step0 {
            &self.solver0
        } else {
            &self.solver1
        }
    }
}
