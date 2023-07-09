use crate::{
    common::generate_random_placement,
    dto::{Point2D, ProblemDto, SolutionDto},
    new_scorer,
};

use super::{Score, Solver};

#[derive(Default, Clone, Debug)]
pub struct Climber {
    problem: ProblemDto,
    placements: Vec<Point2D>,
    score: Score,
    max_iterations: u32,
}

impl Solver for Climber {
    fn name(&self) -> String {
        "climber".to_string()
    }

    fn initialize(&mut self, problem: &super::Problem, solution: crate::dto::SolutionDto) {
        self.problem = problem.data.clone();
        self.placements = if !solution.placements.is_empty() {
            solution.placements
        } else {
            initialize_placements(&self.problem)
        };
        self.score = new_scorer::new_score(&self.problem, &self.placements);
        self.max_iterations = 1000;
    }

    fn solve_step(&mut self) -> (crate::dto::SolutionDto, bool) {
        (
            SolutionDto {
                placements: self.placements.clone(),
            },
            true,
        )
    }
}

fn initialize_placements(problem: &ProblemDto) -> Vec<Point2D> {
    // TODO: write an algorithm to place the musicians in rows near the edges of the stage
    // if the first row is full, then fill behind it

    let mut placements = vec![];

    for _ in 0..problem.musicians.len() {
        placements.push(generate_random_placement(problem, &placements));
    }

    placements
}
