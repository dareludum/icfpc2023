use std::collections::{BTreeSet, HashMap, HashSet};

use rayon::prelude::*;

use crate::{
    common::Position,
    dto::{Instrument, Placement, ProblemDto, SolutionDto},
    scorer::ImpactMap,
};

use super::{Problem, Solver};

#[derive(Default, Clone)]
pub struct Greedy {
    problem: ProblemDto,
    allowed_positions: Vec<Position>,
    placements: Vec<Placement>,
    remaining_musicians: HashSet<usize>,
    impact_maps: HashMap<Instrument, ImpactMap>,
}

impl Solver for Greedy {
    fn name(&self) -> &'static str {
        "greedy"
    }

    fn initialize(&mut self, problem: &Problem) {
        self.problem = problem.data.clone();
        let x = self.problem.stage_bottom_left.0 + 10.0;
        let y = self.problem.stage_bottom_left.1 + 10.0;
        let until_x = self.problem.stage_bottom_left.0 + self.problem.stage_width - 10.0;
        let until_y = self.problem.stage_bottom_left.1 + self.problem.stage_height - 10.0;

        println!("greedy: {} total musicians", self.problem.musicians.len());
        let max_instrument = self.problem.musicians.iter().map(|i| i.0).max().unwrap();
        println!("greedy: {} total instruments", max_instrument);

        let max_position_count = 10000.0 / max_instrument as f32;
        let min_position_count = self.problem.musicians.len() as f32 * 4.0;
        const MIN_DELTA: f32 = 0.5;
        let mut delta = MIN_DELTA;
        loop {
            let position_count = ((until_x - x) / delta) * ((until_y - y) / delta);
            if position_count < min_position_count {
                delta /= 1.1;
                break;
            }
            if position_count < max_position_count {
                break;
            }
            delta *= 1.1;
        }

        println!("greedy: delta = {}", delta);

        let mut curr_y = y;
        while curr_y < until_y {
            let mut curr_x = x;
            while curr_x < until_x {
                self.allowed_positions.push(Position {
                    p: Placement {
                        x: curr_x,
                        y: curr_y,
                    },
                    taken: false,
                });
                curr_x += delta;
            }
            curr_y += delta;
        }

        println!("greedy: {} total positions", self.allowed_positions.len());

        for i in 0..self.problem.musicians.len() {
            self.remaining_musicians.insert(i);
            self.placements.push(Placement {
                x: f32::NAN,
                y: f32::NAN,
            });
        }

        // Compute impact maps
        println!("greedy: computing impact maps");
        self.impact_maps = (0..=max_instrument)
            .map(|i| Instrument(i))
            .collect::<Vec<_>>()
            .par_iter()
            .map(|i| {
                let impact_map =
                    ImpactMap::new(i, &self.problem.attendees, &self.allowed_positions);
                (*i, impact_map)
            })
            .collect();

        println!("greedy: initialized");
    }

    fn solve_step(&mut self) -> (SolutionDto, bool) {
        let mut best_pos_idx = usize::MAX;
        let mut best_instrument = Instrument(u32::MAX);
        let mut best_score = i64::MIN;

        let mut remaining_instruments = BTreeSet::new();
        for idx in &self.remaining_musicians {
            remaining_instruments.insert(self.problem.musicians[*idx]);
        }
        let remaining_instruments = remaining_instruments.into_iter().collect::<Vec<_>>();

        for i in remaining_instruments.iter() {
            let scores = &self.impact_maps[i].scores;
            let best = scores
                .iter()
                .zip(self.allowed_positions.iter())
                .enumerate()
                .filter(|(_idx, (_s, p))| !p.taken)
                .max_by_key(|(_idx, (s, _p))| s.0)
                .unwrap();
            if best_score < best.1 .0 .0 {
                best_score = best.1 .0 .0;
                best_instrument = *i;
                best_pos_idx = best.0;
            }
        }

        let idx = *self
            .remaining_musicians
            .iter()
            .find(|idx| self.problem.musicians[**idx] == best_instrument)
            .unwrap();
        self.remaining_musicians.remove(&idx);

        let best_pos = self.allowed_positions[best_pos_idx];
        self.placements[idx] = best_pos.p;

        // Remove the positions near the new musician
        for pos in &mut self.allowed_positions {
            let x = pos.p.x - best_pos.p.x;
            let y = pos.p.y - best_pos.p.y;
            let dist = (x * x + y * y).sqrt();
            if dist <= 10.0 {
                pos.taken = true;
            }
        }

        self.impact_maps.par_iter_mut().for_each(|(_i, im)| {
            im.update(
                &best_instrument,
                &best_pos.p,
                &self.problem.attendees,
                &self.allowed_positions,
            );
        });

        println!("greedy: {} musicians left", self.remaining_musicians.len());

        (
            SolutionDto {
                placements: self.placements.clone(),
            },
            self.remaining_musicians.is_empty(),
        )
    }
}
