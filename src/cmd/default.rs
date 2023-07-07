use std::path::{Path, PathBuf};

use rayon::prelude::{IntoParallelRefIterator, ParallelIterator};

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
        let full_solver_name = solver.name().to_owned();
        let cur_solver_dir = &base_solution_dir.join("current").join(&full_solver_name);
        let best_dir = &base_solution_dir.join("best");
        std::fs::create_dir_all(cur_solver_dir)?;

        // solve
        let solution = solver.solve(&problem);

        // write the solution
        let solution_meta = solution.save(full_solver_name.clone(), &problem, cur_solver_dir)?;

        // compare with the best solution
        let best_sol = match Solution::load(best_dir, &problem) {
            Ok(sol) => Some(sol),
            Err(ref e) if e.kind() == std::io::ErrorKind::NotFound => None,
            Err(e) => return Err(e),
        };

        let new_best_sol = match &best_sol {
            Some((_, best_sol)) if solution_meta.score < best_sol.score => true,
            None => true,
            _ => false,
        };

        if new_best_sol {
            solution.save(full_solver_name, &problem, best_dir)?;
        }

        print!(
            "{:15}{}: {} ",
            format!("[problem {}]", problem.id),
            solver.name(),
            solution_meta.score
        );

        match (&best_sol, &new_best_sol) {
            // new best
            (Some((_, best_sol)), true) => {
                let improvement = best_sol.score - solution_meta.score;
                println!(
                    "!!! WE ARE WINNING SON !!!, improvement of {}! previous best: {}",
                    improvement, best_sol.score
                );
            }
            // nothing special, no new best
            (Some((_, best_sol)), false) => {
                println!("lower than best: {}", best_sol.score);
            }
            // first solution ever
            (None, _) => {
                println!("!!! FIRST BLOOD !!!");
            }
        }
    }
    Ok(())
}

fn solve(solvers: &[String], problem_paths: &[PathBuf]) -> std::io::Result<()> {
    let base_solution_dir = PathBuf::from("./solutions/");

    let solvers: Vec<_> = solvers
        .iter()
        .map(|solver_name| create_solver(solver_name))
        .collect();

    problem_paths
        .par_iter()
        .map(|problem_path| solve_problem(&solvers, &base_solution_dir, problem_path))
        .collect::<std::io::Result<()>>()
}

pub fn default_command(
    problem_paths: &[PathBuf],
    solvers: Option<Vec<String>>,
) -> Result<(), std::io::Error> {
    match (problem_paths, solvers) {
        ([problem_path], None) => {
            gui_main(&std::path::PathBuf::from(problem_path), "no_op");
            Ok(())
        }
        (paths, Some(solvers)) => solve(&solvers, paths),
        (_, None) => panic!("No problem paths and solvers provided"),
    }
}
