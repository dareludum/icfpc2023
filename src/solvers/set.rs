use std::collections::HashMap;

use log::debug;

use crate::{
    dto::SolutionDto,
    scoring::{new_scorer::NewScorer, scorer::LegacyScorer},
};

use super::{Parameter, Problem, Solver};

#[derive(Clone, Copy)]
enum Scorer {
    Legacy,
    New,
}

impl Scorer {
    fn name(&self) -> &'static str {
        match self {
            Scorer::Legacy => "legacy",
            Scorer::New => "new",
        }
    }
}

#[derive(Default, Clone)]
pub struct Set {
    // Parameters
    scorer: Option<Scorer>,
    // Data
    problem: Problem,
    solution: SolutionDto,
}

impl Solver for Set {
    fn name(&self) -> String {
        let mut name = "set".to_owned();
        if let Some(scorer) = self.scorer {
            name += &format!("_scorer_{}", scorer.name(),);
        }
        name
    }

    fn set_parameters(&mut self, parameters: HashMap<String, Parameter>) {
        for (k, v) in parameters.into_iter() {
            match (k.as_str(), v) {
                ("scorer", Parameter::String(v)) => {
                    self.scorer = Some(match v.as_str() {
                        "legacy" => Scorer::Legacy,
                        "new" => Scorer::New,
                        _ => panic!("Unknown scorer {}", v),
                    })
                }
                _ => panic!("Unknown parameter {}", k),
            }
        }
    }

    fn get_problem(&self) -> &Problem {
        &self.problem
    }

    fn initialize(&mut self, problem: &Problem, solution: SolutionDto) {
        self.problem = problem.clone();
        self.solution = solution;
    }

    fn solve_step(&mut self) -> (SolutionDto, bool) {
        if let Some(scorer) = self.scorer {
            debug!(
                "set({}): setting scorer to {}",
                self.problem.id,
                scorer.name()
            );
            self.problem.scorer = match scorer {
                Scorer::Legacy => Box::<LegacyScorer>::default(),
                Scorer::New => Box::<NewScorer>::default(),
            }
        }

        debug!("set({}): done", self.problem.id,);

        (self.solution.clone(), true)
    }
}
