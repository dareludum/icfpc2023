use log::debug;
use rand::Rng;

use crate::dto::{Point2D, ProblemDto, SolutionDto};

use super::Solver;

#[derive(Clone)]
pub struct Genetic {
    pub population_size: u32,
    problem: ProblemDto,
    population: Vec<Individual>,
}

#[derive(Clone)]
struct Individual {
    fitness: i64,
    placements: Vec<Point2D>,
}

impl Default for Genetic {
    fn default() -> Self {
        Self {
            population_size: 1,
            problem: ProblemDto::default(),
            population: Vec::new(),
        }
    }
}

impl Solver for Genetic {
    fn name(&self) -> &str {
        "genetic"
    }

    fn initialize(&mut self, problem: &super::Problem) {
        self.problem = problem.data.clone();
        self.population = self.create_initial_population(self.population_size, &self.problem);
    }

    fn solve_step(&mut self) -> (SolutionDto, bool) {
        let solution = SolutionDto {
            placements: self.population[0].placements.clone(),
        };

        (solution, false)
    }
}

impl Genetic {
    fn create_initial_population(
        &self,
        population_size: u32,
        problem: &ProblemDto,
    ) -> Vec<Individual> {
        let mut population = Vec::new();

        for _ in 0..population_size {
            let mut placements = Vec::new();
            placements.push(get_random_coords(&problem));
            let mut placed = 1;

            for _ in 0..problem.musicians.len() {
                let mut placement = get_random_coords(&problem);
                let mut correct_placed = false;

                while !correct_placed {
                    correct_placed = true;

                    for i in 0..placed {
                        let other_placement = placements[i as usize];

                        if distance(&placement, &other_placement) < 10.0 {
                            debug!(
                                "Musicians too close, retrying, placement: {:?} other_placement: {:?}",
                                placement, other_placement
                            );
                            placement = get_random_coords(&problem);
                            correct_placed = false;
                            break;
                        }
                    }

                    debug!("Placed musician at {:?}", placement);
                }

                placements.push(placement.clone());
                placed += 1;
            }
            population.push(Individual {
                fitness: 0,
                placements,
            });
        }

        population
    }
}

fn distance(a: &Point2D, b: &Point2D) -> f64 {
    a.as_vec().metric_distance(&b.as_vec()) as f64
}

fn get_random_coords(problem: &ProblemDto) -> Point2D {
    let mut rng = rand::thread_rng();

    Point2D {
        x: rng.gen_range(
            (problem.stage_bottom_left.0 + 10.0)
                ..problem.stage_bottom_left.0 + problem.stage_width - 10.0,
        ),
        y: rng.gen_range(
            (problem.stage_bottom_left.1 + 10.0)
                ..problem.stage_bottom_left.1 + problem.stage_height - 10.0,
        ),
    }
}
