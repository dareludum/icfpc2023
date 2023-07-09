use std::{cmp, collections::HashSet};

use log::debug;
use rand::Rng;

use crate::{
    common::{calculate_invalid_positions, generate_random_placement, get_random_coords},
    dto::{Point2D, ProblemDto, SolutionDto},
    scoring::new_scorer::new_score,
};

use super::Solver;

#[derive(Clone, Debug)]
pub struct Genetic {
    pub population_size: u32,
    problem: ProblemDto,
    population: Vec<Individual>,
    max_generations: u32,
    generation: u32,
    mutation_rate: f32,
    elitism_rate: f32,
    crossover_rate: f32,
}

#[derive(Clone, Debug)]
struct Individual {
    fitness: i64,
    placements: Vec<Point2D>,
}

impl Default for Genetic {
    fn default() -> Self {
        Self {
            population_size: 20,
            problem: ProblemDto::default(),
            population: Vec::new(),
            max_generations: 10,
            generation: 0,
            mutation_rate: 0.5,
            elitism_rate: 0.05,
            crossover_rate: 0.33,
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

        self.population.sort_by_key(|x| cmp::Reverse(x.fitness));
    }

    fn solve_step(&mut self) -> (SolutionDto, bool) {
        debug!(
            "generation: {} out of {}",
            self.generation, self.max_generations
        );

        self.selection();

        self.generation += 1;
        let is_finished = self.generation >= self.max_generations;

        let best_population = self.population.first().expect("population is empty");
        debug!("Best fitness: {}", best_population.fitness);

        let solution = SolutionDto {
            placements: best_population.placements.clone(),
            ..Default::default()
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
            placements.push(get_random_coords(problem));

            for _ in 0..problem.musicians.len() {
                placements.push(generate_random_placement(problem, &placements));
            }

            let individual = Individual {
                fitness: new_score(problem, &placements).0,
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

        if total_fitness <= 0 {
            // In case due to floating point underflow or similar we didn't select an individual, return the last one
            debug!("Roulette wheel selection failed, returning last individual");
            return &population[population.len() - 1];
        }

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

        // Elitism: keep x% of the best individuals
        let elitism_size = (self.population_size as f32 * self.elitism_rate) as usize;
        for i in 0..elitism_size {
            new_population.push(self.population[i].clone());
            debug!(
                "Added elite individual with fitness {} to the new population",
                self.population[i].fitness
            );
        }

        for _ in 0..self.population_size as usize - elitism_size {
            let parent1 = Self::roulette_wheel_selection(&self.population);
            let parent2 = Self::roulette_wheel_selection(&self.population);

            let (mut child1, mut child2) = self.swap_crossover(parent1, parent2);

            if rng.gen_range(0.0..1.0) < self.mutation_rate {
                child1.mutate(&self.problem);
            }

            if rng.gen_range(0.0..1.0) < self.mutation_rate {
                child2.mutate(&self.problem);
            }

            child1.recalculate_fitness(&self.problem);
            child2.recalculate_fitness(&self.problem);

            new_population.push(child1);
            new_population.push(child2);
        }

        self.population = new_population;
        self.population.sort_by_key(|x| cmp::Reverse(x.fitness));

        debug!(
            "Finished selection, population size: {}",
            self.population.len()
        );
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

        // swap x% of the positions
        for _ in 0..(size as f32 * self.crossover_rate) as usize {
            // Select a random musician
            let musician = rng.gen_range(0..size - 1);

            // Swap the positions of this musician in the children
            child1.placements[musician] = parent2.placements[musician];
            child2.placements[musician] = parent1.placements[musician];
        }

        random_repair_invalid_positions(&self.problem, &mut child1.placements);
        random_repair_invalid_positions(&self.problem, &mut child2.placements);

        (child1, child2)
    }
}

fn random_repair_invalid_positions(problem: &ProblemDto, placements: &mut [Point2D]) {
    let mut invalid_positions = calculate_invalid_positions(placements, problem);

    while !invalid_positions.is_empty() {
        for invalid_position in invalid_positions.iter() {
            let placement = generate_random_placement(problem, placements);
            placements[*invalid_position] = placement;
        }

        invalid_positions = calculate_invalid_positions(placements, problem);
    }
}

impl Individual {
    fn recalculate_fitness(&mut self, problem: &ProblemDto) {
        self.fitness = new_score(problem, &self.placements).0;
    }

    fn mutate(&mut self, problem: &ProblemDto) {
        let mut rng = rand::thread_rng();
        let musician = rng.gen_range(0..self.placements.len());
        let mut placement_set: HashSet<Point2D> = HashSet::new();

        for placement in &self.placements {
            placement_set.insert(*placement);
        }

        let mut placement = generate_random_placement(problem, &self.placements);

        while placement_set.contains(&placement) {
            placement = generate_random_placement(problem, &self.placements);
        }

        self.placements[musician] = placement;
    }
}
