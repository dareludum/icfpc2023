use std::{
    cmp::Ordering,
    path::{Path, PathBuf},
};

use log::info;
use rayon::prelude::{ParallelBridge, ParallelIterator};

use crate::{
    gui::gui_main,
    solvers::{create_solver, Problem, Solution, Solver},
};

fn solve_problem(
    solvers: &[Box<dyn Solver>],
    base_solution_dir: &Path,
    problem_path: &Path,
) -> std::io::Result<()> {
    let problem = Problem::load(problem_path)?;

    let solvers = solvers.to_owned();

    for mut solver in solvers {
        // solve
        info!("solving problem {} using {}", problem.id, solver.name());
        let solution = solver.solve(&problem);

        print!(
            "{:15}{}: {} ",
            format!("[problem {}]", problem.id),
            solver.name(),
            solution.score.0
        );

        let full_solver_name = solver.name();
        let cur_solver_dir = &base_solution_dir.join("current").join(&full_solver_name);
        let best_dir = &base_solution_dir.join("best");
        std::fs::create_dir_all(cur_solver_dir)?;
        std::fs::create_dir_all(best_dir)?;

        // write the solution
        solution.save(full_solver_name.clone(), &problem, cur_solver_dir)?;

        if solution.score.0 < 0 {
            println!("Saved, but won't compare with best");
            continue;
        }

        // compare with the best solution
        let best_sol = match Solution::load(best_dir, &problem) {
            Ok(sol) => Some(sol),
            Err(ref e) if e.kind() == std::io::ErrorKind::NotFound => None,
            Err(e) => return Err(e),
        };

        let new_best_sol = match &best_sol {
            Some((_, best_sol)) => solution.score.0.cmp(&best_sol.score),
            None => Ordering::Greater,
        };

        if new_best_sol == Ordering::Greater {
            solution.save(full_solver_name, &problem, best_dir)?;
        }

        match (&best_sol, &new_best_sol) {
            // new best
            (Some((_, best_sol)), Ordering::Greater) => {
                let improvement = solution.score.0 - best_sol.score;
                println!(
                    "!!! WE ARE WINNING SON !!!, improvement of {}! previous best: {}",
                    improvement, best_sol.score
                );
            }
            // likely the same solver
            (Some((_, _)), Ordering::Equal) => {
                println!("ties current best");
            }
            // nothing special, no new best
            (Some((_, best_sol)), Ordering::Less) => {
                println!("worse than best: {}", best_sol.score);
            }
            // first solution ever
            (None, _) => {
                println!("!!! FIRST BLOOD !!!");
            }
        }
    }
    Ok(())
}

fn solve(solvers: &[String], problem_paths: &[PathBuf], parallel: bool) -> std::io::Result<()> {
    let base_solution_dir = PathBuf::from("./solutions/");

    let solvers: Vec<_> = solvers
        .iter()
        .map(|solver_name| create_solver(solver_name))
        .collect();

    if parallel {
        problem_paths
            .iter()
            .par_bridge()
            .map(|problem_path| solve_problem(&solvers, &base_solution_dir, problem_path))
            .collect::<std::io::Result<()>>()
    } else {
        #[allow(clippy::map_collect_result_unit)]
        problem_paths
            .iter()
            .map(|problem_path| solve_problem(&solvers, &base_solution_dir, problem_path))
            .collect::<std::io::Result<()>>()
    }
}

pub fn default_command(
    problem_paths: &[PathBuf],
    solvers: Option<Vec<String>>,
    gui: bool,
    parallel: bool,
) -> Result<(), std::io::Error> {
    match (problem_paths, solvers, gui) {
        ([problem_path], None, _) => {
            gui_main(&std::path::PathBuf::from(problem_path), "expand");
            Ok(())
        }
        ([problem_path], Some(solvers), true) => {
            if solvers.len() != 1 {
                panic!("Only one solver can be used in GUI mode");
            }

            let solver = solvers.first().expect("Expected exactly one solver");
            gui_main(&std::path::PathBuf::from(problem_path), solver);
            Ok(())
        }
        (paths, Some(solvers), false) => solve(&solvers, paths, parallel),
        (_, Some(_), true) => panic!("GUI mode is not supported with multiple solvers"),
        (_, None, _) => panic!("No problem paths and solvers provided"),
    }
}
