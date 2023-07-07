mod no_op;

use std::path::PathBuf;
use std::fs::File;
use std::io::BufReader;

use dyn_clone::DynClone;

use crate::{dto::{SolvedSolutionDto, ProblemDto}, helpers::os_str_to_str};

pub struct Problem {
    pub id: String,
    pub data: ProblemDto,
}


impl Problem {
    pub fn load(problem_path: &PathBuf) -> std::io::Result<Self> {
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

pub struct Score(u64);


pub struct Solution {
    pub score: Score,
}


impl Solution {
    pub fn load(dir: &PathBuf, problem: &Problem) -> std::io::Result<(Self, SolvedSolutionDto)> {
        let problem_base = dir.join(&problem.id);
        let meta_path = problem_base.with_file_name(format!("{}_meta.json", problem.id));

        // TODO: Add any other solution loading here

        let current_best_json: String = std::fs::read_to_string(meta_path)?.into();
        let metadata: SolvedSolutionDto =
            serde_json::from_str(&current_best_json).expect("Deserialization error");
        let solution = Solution {
            score: Score(metadata.score),
        };
        Ok((solution, metadata))
    }

    pub fn save(
        &self,
        solver_name: String,
        problem: &Problem,
        dir: &PathBuf,
    ) -> std::io::Result<SolvedSolutionDto> {
        let problem_base = dir.join(&problem.id);
        let meta_path = problem_base.with_file_name(format!("{}_meta.json", problem.id));

        // TODO: Add any other solution saving here

        let solution_meta = SolvedSolutionDto {
            solver_name: solver_name,
            score: 0,
        };
        let solution_meta_json = serde_json::to_string_pretty(&solution_meta)?;
        std::fs::write(meta_path, solution_meta_json)?;
        Ok(solution_meta)
    }
}

pub trait Solver: DynClone + Sync + Send {
    fn name(&self) -> &str;
    // TODO: Add the proper types for the contest problem
    fn solve_core(&self, problem: &Problem) -> ();

    fn solve(&self, problem: &Problem) -> Solution {
        let _result = self.solve_core(problem);

        // TODO: Process the result

        Solution { score: Score(0) }
    }
}

dyn_clone::clone_trait_object!(Solver);

pub const SOLVERS: &[&str] = &["no_op"];

pub fn create_solver(solver_name: &str) -> Box<dyn Solver> {
    // TODO: Copy-paste processors support from previous year if needed
    create_individual_solver(solver_name)
}

fn create_individual_solver(solver_name: &str) -> Box<dyn Solver> {
    match solver_name {
        "no_op" => Box::new(no_op::NoOp {}),
        n => panic!("Unknown solver `{}`", n),
    }
}
