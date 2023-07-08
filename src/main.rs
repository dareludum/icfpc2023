use std::path::Path;
use std::time::Instant;
use std::{ffi::OsString, fs::DirEntry, path::PathBuf};

use clap::Parser;
use cmd::default::*;
use cmd::stats::*;
use cmd::Args;
use cmd::Commands;
use dto::SolutionDto;
use env_logger::Env;
use helpers::*;
use solvers::Problem;
use solvers::SOLVERS;

mod cmd;
mod collider;
mod common;
mod dto;
mod geometry;
mod gui;
mod helpers;
mod kdtree;
mod new_scorer;
mod scorer;
mod solvers;

#[macro_use]
extern crate enum_map;

fn get_problem_paths(args: &Args, force_batch: bool) -> Result<Vec<PathBuf>, std::io::Error> {
    if !args.problems.is_empty() {
        Ok(args
            .problems
            .iter()
            .map(|p| PathBuf::from(format!("./problems/{p}.json")))
            .collect())
    } else if args.batch || force_batch {
        Ok(get_all_problem_paths()?)
    } else {
        Ok(vec![PathBuf::from("./problems/42.json")])
    }
}

fn get_all_problem_paths() -> Result<Vec<PathBuf>, std::io::Error> {
    let paths: Vec<PathBuf> = std::fs::read_dir("./problems")?
        .collect::<Result<Vec<DirEntry>, _>>()?
        .iter()
        .filter_map(|f| {
            let x = os_str_to_str(f.path().file_name());
            if x.ends_with(".json") {
                Some(f.path())
            } else {
                None
            }
        })
        .collect();

    Ok(paths)
}

fn get_solvers(args: &Args) -> Option<Vec<String>> {
    if !args.solvers.is_empty() {
        Some(args.solvers.clone())
    } else if args.batch {
        Some(SOLVERS.iter().map(|s| s.to_string()).collect())
    } else {
        None
    }
}

fn list_current_solvers() -> Vec<String> {
    let mut current_solvers = vec![];
    let solvers_dir = std::fs::read_dir(PathBuf::from("./solutions/current"))
        .expect("Can't list solutions current dir");

    for solver in solvers_dir {
        let (id_dir, file_name) = solver
            .and_then(|x| {
                let ftype = x.file_type()?;
                Ok((ftype, x.file_name()))
            })
            .map(|(typ, fname)| (typ.is_dir(), fname))
            .unwrap_or((false, OsString::new()));

        if id_dir {
            current_solvers.push(os_str_to_str(Some(&file_name)));
        }
    }

    current_solvers
}

fn main() -> std::io::Result<()> {
    let args = Args::parse();
    let solvers = get_solvers(&args);
    let log_level = args.log_level.clone().unwrap_or_else(|| "info".to_string());
    let gui = args.gui;

    env_logger::Builder::from_env(Env::default().default_filter_or(log_level)).init();

    match &args.command {
        Some(Commands::Stats) => {
            let problem_paths = get_problem_paths(&args, true)?;

            let mut problems: Vec<String> = problem_paths
                .iter()
                .map(|p| os_str_to_str(p.file_stem()))
                .collect();

            problems.sort_by_key(|x| x.parse::<u8>().unwrap());
            stats(&problems, &solvers.unwrap_or_else(list_current_solvers))
        }
        Some(Commands::Score { problem, solution }) => {
            let problem = Problem::load(Path::new(problem))?;
            let solution = SolutionDto::load(Path::new(solution))?;
            let before_score = Instant::now();
            let score = scorer::score(&problem.data, &solution.placements);
            let score_time = before_score.elapsed();

            let before_fast_score = Instant::now();
            let fast_score = new_scorer::new_score(&problem.data, &solution.placements);
            let fast_score_time = before_fast_score.elapsed();
            println!("score: {} ({}us)", score.0, score_time.as_micros());
            println!(
                "fast score: {} ({}us)",
                fast_score.0,
                fast_score_time.as_micros()
            );
            Ok(())
        }
        _ => {
            let problem_paths = get_problem_paths(&args, false)?;
            default_command(&problem_paths, solvers, gui)
        }
    }
}
