use std::{fs, path::Path};

use crate::dto::SolvedSolutionDto;

pub fn stats(problems_n: &[String], solvers: &[String]) -> Result<(), std::io::Error> {
    let mut sum_best: u64 = 0;

    for n in problems_n {
        let best_fname = format!("./solutions/best/{n}_meta.json");
        let best_path = Path::new(&best_fname);

        let best: SolvedSolutionDto = serde_json::from_str(&fs::read_to_string(best_path)?)?;
        sum_best += best.score;
        let mut current_solved = Vec::with_capacity(problems_n.len());

        for solver in solvers {
            let path_s = format!("./solutions/current/{solver}/{n}_meta.json");
            let path = Path::new(&path_s);
            if let Ok(true) = path.try_exists() {
                let dto: SolvedSolutionDto = serde_json::from_str(&fs::read_to_string(path)?)?;
                current_solved.push(dto);
            };
        }

        current_solved.sort_by_key(|x| x.score);

        println!("Problem {n}");
        println!("------------------------------------");
        println!("best: {} score={}", best.solver_name, best.score);
        current_solved
            .iter()
            .for_each(|x| println!("{} score={}", x.solver_name, x.score));
        println!("------------------------------------");
    }
    println!("------------------------------------");
    println!("Sum of all best: {sum_best}");

    Ok(())
}
