use std::collections::HashMap;

use log::debug;
use rand::prelude::*;

use crate::{
    common::calculate_invalid_positions,
    dto::{Point2D, SolutionDto},
    geometry::Coords2D,
};

use super::{Parameter, Problem, Score, Solver};

#[derive(Clone)]
struct Particle {
    positions: Vec<Point2D>,
    best_positions: Vec<Point2D>,
    best_score: Score,
    velocities: Vec<Point2D>,
}

#[derive(Default, Clone)]
pub struct Swarm {
    // Parameters
    cycles_cap: Option<u32>,
    // Data
    problem: Problem,
    min_x: f32,
    min_y: f32,
    max_x: f32,
    max_y: f32,
    best_positions: Vec<Point2D>,
    best_score: Score,
    particles: Vec<Particle>,
    cycles_count: u32,
}

impl Solver for Swarm {
    fn name(&self) -> String {
        let mut name = "expand".to_owned();
        if let Some(cap) = self.cycles_cap {
            name += &format!("_cap_{}", cap);
        }
        name
    }

    fn set_parameters(&mut self, parameters: HashMap<String, Parameter>) {
        for (k, v) in parameters.into_iter() {
            match (k.as_str(), v) {
                ("cap", Parameter::Int(v)) => self.cycles_cap = Some(v as u32),
                _ => panic!("Unknown parameter {}", k),
            }
        }
    }

    fn get_problem(&self) -> &Problem {
        &self.problem
    }

    fn initialize(&mut self, problem: &Problem, solution: SolutionDto) {
        self.problem = problem.clone();

        const SWARM_SIZE: usize = 20;

        let mut rng = rand::thread_rng();

        debug!(
            "swarm({}): initializing {} particles",
            self.problem.id, SWARM_SIZE
        );

        self.min_x = self.problem.data.stage_bottom_left.x() + 10.0;
        self.min_y = self.problem.data.stage_bottom_left.y() + 10.0;
        self.max_x = self.problem.data.stage_bottom_left.x() + self.problem.data.stage_width - 10.0;
        self.max_y =
            self.problem.data.stage_bottom_left.y() + self.problem.data.stage_height - 10.0;

        let mut best_positions = vec![];
        let mut best_score = Score(i64::MIN);
        for i in 0..SWARM_SIZE {
            let mut positions = vec![];
            if !solution.placements.is_empty() {
                positions = solution.placements.clone();
            } else {
                for _ in 0..self.problem.data.musicians.len() {
                    positions.push(Point2D {
                        x: rng.gen_range(self.min_x..self.max_x),
                        y: rng.gen_range(self.min_y..self.max_y),
                    })
                }
            }

            let mut velocities = vec![];
            for _ in 0..self.problem.data.musicians.len() {
                velocities.push(Point2D {
                    x: rng.gen_range(-self.problem.data.stage_width..self.problem.data.stage_width)
                        / 50.0,
                    y: rng
                        .gen_range(-self.problem.data.stage_height..self.problem.data.stage_height)
                        / 50.0,
                })
            }

            let score = self.score(&positions);
            if score.0 > best_score.0 {
                best_positions = positions.clone();
                best_score = score;
            }

            let p = Particle {
                positions: positions.clone(),
                best_positions: positions,
                best_score: score,
                velocities,
            };

            self.particles.push(p);

            loop {
                let invalid =
                    calculate_invalid_positions(&self.particles[i].positions, &self.problem.data);
                if invalid.is_empty() {
                    break;
                }

                let p = &mut self.particles[i];

                for idx in invalid {
                    let x = &mut p.positions[idx].x;
                    *x += p.velocities[idx].x;
                    while *x < self.min_x {
                        *x += self.max_x - self.min_x;
                    }
                    while *x > self.max_x {
                        *x -= self.max_x - self.min_x;
                    }
                    let y = &mut p.positions[idx].y;
                    *y += p.velocities[idx].y;
                    while *y < self.min_y {
                        *y += self.max_y - self.min_y;
                    }
                    while *y > self.max_y {
                        *y -= self.max_y - self.min_y;
                    }
                }
            }

            debug!(
                "swarm({}): initialized particle {}",
                self.problem.id,
                self.particles.len() - 1
            );
        }

        self.best_positions = best_positions;
        self.best_score = best_score;

        debug!("swarm({}): initialized", self.problem.id);
    }

    fn solve_step(&mut self) -> (SolutionDto, bool) {
        let mut rng = rand::thread_rng();

        let weight: f32 = rng.gen_range(0.01..0.03);
        const COGNITIVE_COEFF: f32 = 2.1;
        const SOCIAL_COEFF: f32 = 1.7;

        for i in 0..self.particles.len() {
            let i_prev = i.wrapping_sub(1) % self.particles.len();
            let i_next = (i + 1) % self.particles.len();
            let i_best =
                if self.particles[i_prev].best_score.0 > self.particles[i_next].best_score.0 {
                    i_prev
                } else {
                    i_next
                };

            for idx in 0..self.problem.data.musicians.len() {
                let best_position = self.particles[i_best].best_positions[idx];
                // let best_position = self.best_positions[idx];
                let p = &mut self.particles[i];

                // Update velocity
                let rp = rng.gen::<f32>();
                let rg = rng.gen::<f32>();
                p.velocities[idx].x = weight * p.velocities[idx].x
                    + COGNITIVE_COEFF * rp * (p.best_positions[idx].x - p.positions[idx].x)
                    + SOCIAL_COEFF * rg * (best_position.x - p.positions[idx].x);
                p.velocities[idx].y = weight * p.velocities[idx].y
                    + COGNITIVE_COEFF * rp * (p.best_positions[idx].y - p.positions[idx].y)
                    + SOCIAL_COEFF * rg * (best_position.y - p.positions[idx].y);

                // Update position
                let x = &mut p.positions[idx].x;
                *x += p.velocities[idx].x;
                while *x < self.min_x {
                    *x += self.max_x - self.min_x;
                }
                while *x > self.max_x {
                    *x -= self.max_x - self.min_x;
                }
                let y = &mut p.positions[idx].y;
                *y += p.velocities[idx].y;
                while *y < self.min_y {
                    *y += self.max_y - self.min_y;
                }
                while *y > self.max_y {
                    *y -= self.max_y - self.min_y;
                }
            }

            loop {
                let invalid =
                    calculate_invalid_positions(&self.particles[i].positions, &self.problem.data);
                if invalid.is_empty() {
                    break;
                }

                let p = &mut self.particles[i];

                for idx in invalid {
                    let x = &mut p.positions[idx].x;
                    *x += p.velocities[idx].x;
                    while *x < self.min_x {
                        *x += self.max_x - self.min_x;
                    }
                    while *x > self.max_x {
                        *x -= self.max_x - self.min_x;
                    }
                    let y = &mut p.positions[idx].y;
                    *y += p.velocities[idx].y;
                    while *y < self.min_y {
                        *y += self.max_y - self.min_y;
                    }
                    while *y > self.max_y {
                        *y -= self.max_y - self.min_y;
                    }
                }
            }

            let score = self.score(&self.particles[i].positions);

            let p = &mut self.particles[i];
            if score.0 > p.best_score.0 {
                debug!(
                    "swarm({}): new best for particle {}: {}",
                    self.problem.id, i, score.0
                );
                p.best_positions = p.positions.clone();
                p.best_score = score;
                if score.0 > self.best_score.0 {
                    debug!("swarm({}): new global best: {}", self.problem.id, score.0);
                    self.best_positions = p.best_positions.clone();
                    self.best_score = score;
                }
            }
        }

        self.cycles_count += 1;

        debug!(
            "swarm({}): cycle {} done",
            self.problem.id, self.cycles_count
        );

        (
            SolutionDto {
                // placements: self.particles[0].positions.clone(),
                placements: self.best_positions.clone(),
                ..Default::default()
            },
            self.cycles_count >= self.cycles_cap.unwrap_or(u32::MAX),
        )
    }
}

impl Swarm {
    fn score(&self, positions: &[Point2D]) -> Score {
        // let invalid = calculate_invalid_positions(&positions, &self.problem.data);
        // Score(self.problem.score(positions, None).0 / (1 + invalid.len()) as i64)
        self.problem.score(positions, None)
    }
}
