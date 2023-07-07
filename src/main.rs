use std::{ffi::OsString, fs::DirEntry, path::PathBuf};

use clap::Parser;
use cmd::default::*;
use cmd::stats::*;
use cmd::Args;
use cmd::Commands;
use helpers::*;
use solvers::SOLVERS;

mod cmd;
mod dto;
mod gui;
mod helpers;
mod solvers;
mod scorer;

fn get_problem_paths(args: &Args, force_batch: bool) -> Result<Vec<PathBuf>, std::io::Error> {
    if !args.problems.is_empty() {
        Ok(args
            .problems
            .iter()
            .map(|p| PathBuf::from(format!("./problems/{p}.png")))
            .collect())
    } else if args.batch || force_batch {
        Ok(get_all_problem_paths()?)
    } else {
        Ok(vec![PathBuf::from("./problems/3.png")])
    }
}

fn get_all_problem_paths() -> Result<Vec<PathBuf>, std::io::Error> {
    let paths: Vec<PathBuf> = std::fs::read_dir("./problems")?
        .collect::<Result<Vec<DirEntry>, _>>()?
        .iter()
        .filter_map(|f| {
            let x = os_str_to_str(f.path().file_name());
            if x.ends_with(".png") && !x.ends_with(".source.png") {
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
        _ => {
            let problem_paths = get_problem_paths(&args, false)?;
            default_command(&problem_paths, solvers)
        }
    }
}
