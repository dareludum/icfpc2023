use std::collections::HashSet;

use log::debug;
use rand::Rng;

use crate::{
    dto::{Point2D, ProblemDto, SolutionDto},
    scorer::score,
};

use super::Solver;

#[derive(Clone)]
pub struct Genetic {
    pub population_size: u32,
    problem: ProblemDto,
    population: Vec<Individual>,
    max_generations: u32,
    generation: u32,
}

#[derive(Clone)]
struct Individual {
    fitness: i64,
    placements: Vec<Point2D>,
}

impl Default for Genetic {
    fn default() -> Self {
        Self {
            population_size: 100,
            problem: ProblemDto::default(),
            population: Vec::new(),
            max_generations: 10,
            generation: 0,
        }
    }
}

impl Solver for Genetic {
    fn name(&self) -> String {
        "genetic".to_string()
    }

    fn initialize(&mut self, problem: &super::Problem, solution: SolutionDto) {
        self.problem = problem.data.clone();
        self.population = self.create_initial_population(self.population_size, &self.problem);
        if !solution.placements.is_empty() {
            self.population[0].placements = solution.placements;
            self.population[0].recalculate_fitness(&problem.data);
        }
    }

    fn solve_step(&mut self) -> (SolutionDto, bool) {
        debug!(
            "generation: {} out of {}",
            self.generation, self.max_generations
        );

        self.selection();
        // main loop here

        self.generation += 1;
        let is_finished = self.generation >= self.max_generations;

        let best_population = self.population.iter().max_by_key(|i| i.fitness).unwrap();
        debug!("Best fitness: {}", best_population.fitness);

        let solution = SolutionDto {
            placements: best_population.placements.clone(),
        };

        (solution, is_finished)
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
                let placement = generate_random_placement(problem, placed, &placements);

                // debug!("Placed musician at {:?}", placement);
                placements.push(placement.clone());
                placed += 1;
            }

            let individual = Individual {
                fitness: score(&problem, &placements).0,
                placements,
            };

            population.push(individual);

            debug!(
                "Created individual with fitness {}",
                population[population.len() - 1].fitness
            );
        }

        population
    }

    fn roulette_wheel_selection(population: &Vec<Individual>) -> &Individual {
        let mut rng = rand::thread_rng();

        // Calculate the total fitness of the population
        let total_fitness: i64 = population.iter().map(|individual| individual.fitness).sum();

        // Select a random point on the wheel
        let mut selection_point: i64 = rng.gen_range(0.0..total_fitness as f64) as i64;

        // Find the first individual that will spin the wheel past the selection point
        for individual in population {
            selection_point -= individual.fitness;
            if selection_point <= 0 {
                return individual;
            }
        }

        // In case due to floating point underflow or similar we didn't select an individual, return the last one
        debug!("Roulette wheel selection failed, returning last individual");
        &population[population.len() - 1]
    }

    fn selection(&mut self) {
        let mut rng = rand::thread_rng();
        let mut new_population = Vec::new();

        for _ in 0..self.population_size {
            let parent1 = Self::roulette_wheel_selection(&self.population);
            let parent2 = Self::roulette_wheel_selection(&self.population);

            let (mut child1, mut child2) = self.swap_crossover(parent1, parent2);

            if rng.gen_range(0.0..1.0) < 0.5 {
                child1.mutate(&self.problem);
            }

            if rng.gen_range(0.0..1.0) < 0.5 {
                child2.mutate(&self.problem);
            }

            child1.recalculate_fitness(&self.problem);
            child2.recalculate_fitness(&self.problem);

            new_population.push(child1);
            new_population.push(child2);
        }

        self.population = new_population;
    }

    fn crossover_and_repair(
        &self,
        parent1: &Individual,
        parent2: &Individual,
    ) -> (Individual, Individual) {
        let mut rng = rand::thread_rng();
        let size = parent1.placements.len();

        let mut child1 = Individual {
            placements: vec![Point2D { x: 0.0, y: 0.0 }; size],
            fitness: 0,
        };
        let mut child2 = Individual {
            placements: vec![Point2D { x: 0.0, y: 0.0 }; size],
            fitness: 0,
        };

        // Create children by randomly selecting genes from each parent
        for i in 0..size {
            if rng.gen() {
                // generates a random boolean
                child1.placements[i] = parent1.placements[i].clone();
                child2.placements[i] = parent2.placements[i].clone();
            } else {
                child1.placements[i] = parent2.placements[i].clone();
                child2.placements[i] = parent1.placements[i].clone();
            }
        }

        // self.repair(&mut child1);
        // self.repair(&mut child2);

        (child1, child2)
    }

    fn crossover(&self, parent1: &Individual, parent2: &Individual) -> (Individual, Individual) {
        let mut rng = rand::thread_rng();
        let size = parent1.placements.len();

        // Generate two random crossover points
        let crossover_point1 = rng.gen_range(0..size);
        let crossover_point2 = rng.gen_range(0..size);

        // Ensure crossover_point1 is less than crossover_point2
        let (crossover_point1, crossover_point2) = if crossover_point1 > crossover_point2 {
            (crossover_point2, crossover_point1)
        } else {
            (crossover_point1, crossover_point2)
        };

        // Create children with musicians vectors of the same size as parents
        let mut child1 = Individual {
            placements: vec![Point2D { x: 0.0, y: 0.0 }; size],
            fitness: 0,
        };
        let mut child2 = Individual {
            placements: vec![Point2D { x: 0.0, y: 0.0 }; size],
            fitness: 0,
        };

        // Copy segments between crossover points from parents to children
        child1.placements[crossover_point1..crossover_point2]
            .clone_from_slice(&parent1.placements[crossover_point1..crossover_point2]);
        child2.placements[crossover_point1..crossover_point2]
            .clone_from_slice(&parent2.placements[crossover_point1..crossover_point2]);

        let mut parent2_remaining = parent2.placements.to_vec();
        let mut parent1_remaining = parent1.placements.to_vec();

        parent2_remaining.retain(|item| !child1.placements.contains(item));
        parent1_remaining.retain(|item| !child2.placements.contains(item));

        // Fill the remaining spots in the children
        for i in 0..size {
            if i < crossover_point1 || i >= crossover_point2 {
                child1.placements[i] = parent2_remaining.pop().unwrap();
                child2.placements[i] = parent1_remaining.pop().unwrap();
            }
        }

        debug!(
            "Crossover completed: {}, {}",
            crossover_point1, crossover_point2
        );
        (child1, child2)
    }

    fn swap_crossover(
        &self,
        parent1: &Individual,
        parent2: &Individual,
    ) -> (Individual, Individual) {
        let mut rng = rand::thread_rng();
        let size = parent1.placements.len();

        // Children start as exact copies of parents
        let mut child1 = parent1.clone();
        let mut child2 = parent2.clone();

        // Select a random musician
        // let musician = rng.gen_range(0..size);

        // Swap the positions of this musician in the children
        // child1.placements[musician] = parent2.placements[musician].clone();
        // child2.placements[musician] = parent1.placements[musician].clone();

        (child1, child2)
    }
}

fn generate_random_placement(
    problem: &ProblemDto,
    placed: i32,
    placements: &Vec<Point2D>,
) -> Point2D {
    let mut placement = get_random_coords(&problem);
    let mut correct_placed = false;

    while !correct_placed {
        correct_placed = true;

        for i in 0..placed {
            let other_placement = placements[i as usize];

            if distance(&placement, &other_placement) < 10.0 {
                // debug!(
                //     "Musicians too close, retrying, placement: {:?} other_placement: {:?}",
                //     placement, other_placement
                // );
                placement = get_random_coords(&problem);
                correct_placed = false;
                break;
            }
        }
    }
    placement
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

impl Individual {
    fn recalculate_fitness(&mut self, problem: &ProblemDto) {
        self.fitness = score(&problem, &self.placements).0;
    }

    fn mutate(&mut self, problem: &ProblemDto) {
        let mut rng = rand::thread_rng();
        let musician = rng.gen_range(0..self.placements.len());
        let mut placement_set: HashSet<Point2D> = HashSet::new();

        for placement in &self.placements {
            placement_set.insert(*placement);
        }

        let mut placement =
            generate_random_placement(problem, self.placements.len() as i32, &self.placements);

        while placement_set.contains(&placement) {
            placement =
                generate_random_placement(problem, self.placements.len() as i32, &self.placements);
        }

        self.placements[musician] = placement;
    }
}
