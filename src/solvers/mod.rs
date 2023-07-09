mod annealer;
mod chain;
mod expand;
mod genetic;
mod greedy;
mod load_best;
mod mix;
mod shake;
mod vol10;

use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::path::Path;

use derivative::Derivative;
use dyn_clone::DynClone;
use log::debug;
use serde::{Deserialize, Serialize};

use crate::common::{prune_attendees_and_pillars, Grid};
use crate::dto::{Attendee, Instrument, PillarDto, Point2D};
use crate::scoring::impact_map::ImpactMap;
use crate::scoring::Scorer;
use crate::{
    dto::{ProblemDto, SolutionDto, SolutionMetaDto},
    helpers::os_str_to_str,
};

use self::annealer::Annealer;
use self::chain::Chain;
use self::expand::Expand;
use self::genetic::Genetic;
use self::greedy::Greedy;
use self::load_best::LoadBest;
use self::mix::Mix;
use self::shake::Shake;
use self::vol10::Vol10;

#[derive(Default, Clone, Derivative)]
#[derivative(Debug)]
pub struct Problem {
    pub id: String,
    pub data: ProblemDto,
    pub removed_attendees: Vec<Attendee>,
    pub removed_pillars: Vec<PillarDto>,
    #[derivative(Debug = "ignore")]
    scorer: Box<dyn Scorer>,
}

impl Problem {
    pub fn load(problem_path: &Path) -> std::io::Result<Self> {
        let id = os_str_to_str(problem_path.file_stem());
        let file = File::open(problem_path)?;
        let reader = BufReader::new(file);

        let mut problem = Problem {
            id: id.clone(),
            data: serde_json::from_reader(reader)?,
            removed_attendees: vec![],
            removed_pillars: vec![],
            ..Default::default()
        };
        if !problem.data.pillars.is_empty() {
            #[derive(Serialize, Deserialize, Debug, Clone)]
            struct PrunedData {
                pruned_attendees: Vec<Attendee>,
                pruned_pillars: Vec<PillarDto>,
            }

            let mut pruned_data_path = problem_path.parent().unwrap().to_owned();
            pruned_data_path.set_file_name("problems_extra");
            let pruned_data_path = pruned_data_path.join(id + "_pruned_data.json");

            let (pruned_attendees, pruned_pillars) = if pruned_data_path.exists() {
                debug!("prune: found cached pruned data, loading");
                let reader = BufReader::new(File::open(pruned_data_path)?);
                let pruned_data: PrunedData = serde_json::from_reader(reader)?;
                (pruned_data.pruned_attendees, pruned_data.pruned_pillars)
            } else {
                debug!(
                    "prune: trying to prune {} attendees ({} pillars)",
                    problem.data.attendees.len(),
                    problem.data.pillars.len()
                );
                let (pruned_attendees, pruned_pillars) = prune_attendees_and_pillars(&problem.data);
                if !pruned_data_path.exists() {
                    debug!("prune: caching pruned data");
                    let file = File::create(pruned_data_path)?;
                    let writer = BufWriter::new(file);
                    serde_json::to_writer(
                        writer,
                        &PrunedData {
                            pruned_attendees: pruned_attendees.clone(),
                            pruned_pillars: pruned_pillars.clone(),
                        },
                    )?;
                }
                (pruned_attendees, pruned_pillars)
            };

            if pruned_attendees.len() < problem.data.attendees.len() {
                debug!(
                    "prune: {} => {} attendees (-{}%)",
                    problem.data.attendees.len(),
                    pruned_attendees.len(),
                    (((problem.data.attendees.len() as f32 - pruned_attendees.len() as f32)
                        / (problem.data.attendees.len() as f32))
                        * 100.0) as i32
                );
            } else {
                debug!(
                    "prune: pruned 0 attendees ({} total)",
                    problem.data.attendees.len()
                );
            }

            if pruned_pillars.len() < problem.data.attendees.len() {
                debug!(
                    "prune: {} => {} pillars (-{}%)",
                    problem.data.pillars.len(),
                    pruned_pillars.len(),
                    (((problem.data.pillars.len() as f32 - pruned_pillars.len() as f32)
                        / (problem.data.pillars.len() as f32))
                        * 100.0) as i32
                );
            } else {
                debug!(
                    "prune: pruned 0 pillars ({} total)",
                    problem.data.pillars.len()
                );
            }

            let pruned_attendees_set = pruned_attendees.iter().collect::<HashSet<_>>();
            let pruned_pillars_set = pruned_pillars.iter().collect::<HashSet<_>>();

            for a in &problem.data.attendees {
                if !pruned_attendees_set.contains(a) {
                    problem.removed_attendees.push(a.clone());
                }
            }
            for p in &problem.data.pillars {
                if !pruned_pillars_set.contains(p) {
                    problem.removed_pillars.push(p.clone());
                }
            }

            problem.data.attendees = pruned_attendees;
            problem.data.pillars = pruned_pillars;
        }
        Ok(problem)
    }

    pub fn score(&self, placements: &[Point2D], volumes: Option<&Vec<f32>>) -> Score {
        self.scorer.score(&self.data, placements, volumes)
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

    fn set_parameters(&mut self, parameters: HashMap<String, i64>) {
        assert!(
            parameters.is_empty(),
            "Solver {} doesn't accept parameters",
            self.name()
        );
    }
    fn get_problem(&self) -> &Problem;
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
                score: problem.scorer.score(
                    &problem.data,
                    &solution.placements,
                    solution.volumes.as_ref(),
                ),
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
    let (solver_name, parameters) = if solver_name.contains('{') {
        assert!(
            solver_name.ends_with('}'),
            "Invalid solver name {}",
            solver_name
        );
        let (solver_name, rest) = solver_name.split_at(solver_name.find('{').unwrap());
        let rest = &rest[1..rest.len() - 1];
        let parameters = rest
            .split(',')
            .map(|s| {
                let (name, value) = s.split_at(
                    s.find('=')
                        .expect("Invalid parameter format, expected `name=value`"),
                );
                (
                    name.to_owned(),
                    value[1..]
                        .parse::<i64>()
                        .expect("Failed to parse solver parameter as i64"),
                )
            })
            .collect::<HashMap<_, _>>();
        (solver_name, parameters)
    } else {
        (solver_name, HashMap::new())
    };
    let mut solver: Box<dyn Solver> = match solver_name {
        "annealer" => Box::<Annealer>::default(),
        "expand" => Box::<Expand>::default(),
        "genetic" => Box::<Genetic>::default(),
        "greedy" => Box::<Greedy>::default(),
        "load_best" => Box::<LoadBest>::default(),
        "mix" => Box::<Mix>::default(),
        "shake" => Box::<Shake>::default(),
        "vol10" => Box::<Vol10>::default(),
        n => panic!("Unknown solver `{}`", n),
    };
    solver.set_parameters(parameters);
    solver
}
