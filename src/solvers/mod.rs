mod chain;
mod expand;
mod greedy;
mod genetic;
mod shake;

use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::path::Path;

use dyn_clone::DynClone;

use crate::common::Grid;
use crate::dto::Instrument;
use crate::scorer::ImpactMap;
use crate::{
    dto::{ProblemDto, SolutionDto, SolutionMetaDto},
    helpers::os_str_to_str,
    scorer::score,
};

use self::chain::Chain;
use self::expand::Expand;
use self::greedy::Greedy;
use self::genetic::Genetic;
use self::shake::Shake;

#[derive(Default, Clone)]
pub struct Problem {
    pub id: String,
    pub data: ProblemDto,
}

impl Problem {
    pub fn load(problem_path: &Path) -> std::io::Result<Self> {
        let id = os_str_to_str(problem_path.file_stem());
        let file = File::open(problem_path)?;
        let reader = BufReader::new(file);

        // Read the JSON contents of the file as an instance of `User`.
        Ok(Problem {
            id,
            data: serde_json::from_reader(reader)?,
        })
    }
}

#[derive(Default, Clone, Copy)]
pub struct Score(pub i64);

#[derive(Default)]
pub struct Solution {
    pub score: Score,
    pub data: SolutionDto,
}

impl Solution {
    pub fn load(dir: &Path, problem: &Problem) -> std::io::Result<(Self, SolutionMetaDto)> {
        let problem_base = dir.join(&problem.id);

        // load the solution itself
        let data = SolutionDto::load(
            &problem_base.with_file_name(format!("{}_solution.json", problem.id)),
        )?;

        // load solution metadata
        let metadata: SolutionMetaDto = {
            let path = problem_base.with_file_name(format!("{}_meta.json", problem.id));
            let file = File::open(path)?;
            let reader = BufReader::new(file);
            serde_json::from_reader(reader)?
        };

        let solution = Solution {
            score: Score(metadata.score),
            data,
        };
        Ok((solution, metadata))
    }

    pub fn save(&self, solver_name: String, problem: &Problem, dir: &Path) -> std::io::Result<()> {
        let problem_base = dir.join(&problem.id);

        let solution_meta = SolutionMetaDto {
            solver_name,
            score: self.score.0,
        };

        // write metadata
        {
            let path = problem_base.with_file_name(format!("{}_meta.json", problem.id));
            let file = File::create(path)?;
            let writer = BufWriter::new(file);
            serde_json::to_writer(writer, &solution_meta)?;
        }

        // write the solution
        {
            let path = problem_base.with_file_name(format!("{}_solution.json", problem.id));
            let file = File::create(path)?;
            let writer = BufWriter::new(file);
            serde_json::to_writer(writer, &self.data)?;
        }

        Ok(())
    }
}

pub trait Solver: DynClone + Sync + Send {
    fn name(&self) -> String;

    fn initialize(&mut self, problem: &Problem, solution: SolutionDto);
    fn solve_step(&mut self) -> (SolutionDto, bool);

    fn solve(&mut self, problem: &Problem) -> Solution {
        self.initialize(problem, SolutionDto::default());
        loop {
            let (solution, done) = self.solve_step();
            if !done {
                continue;
            }
            return Solution {
                score: score(&problem.data, &solution.placements),
                data: solution,
            };
        }
    }

    fn get_impact_map(&self, _instrument: &Instrument) -> Option<&ImpactMap> {
        None
    }

    fn get_grid(&self) -> Option<&Grid> {
        None
    }
}

dyn_clone::clone_trait_object!(Solver);

pub const SOLVERS: &[&str] = &["expand", "greedy", "genetic"];

pub fn create_solver(solver_name: &str) -> Box<dyn Solver> {
    if solver_name.contains('+') {
        let mut solvers = solver_name.split('+').map(create_individual_solver);
        let chain = Box::new(Chain::new(solvers.next().unwrap(), solvers.next().unwrap()));
        solvers.fold(chain, |chain, next| Box::new(Chain::new(chain, next)))
    } else {
        create_individual_solver(solver_name)
    }
}

fn create_individual_solver(solver_name: &str) -> Box<dyn Solver> {
    match solver_name {
        "expand" => Box::<Expand>::default(),
        "greedy" => Box::<Greedy>::default(),
        "genetic" => Box::<Genetic>::default(),
        "shake" => Box::<Shake>::default(),
        n => panic!("Unknown solver `{}`", n),
    }
}
