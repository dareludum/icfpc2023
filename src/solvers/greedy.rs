use std::collections::{BTreeMap, HashMap, HashSet};

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

    fn get_impact_map(&self, instrument: &Instrument) -> Option<&ImpactMap> {
        self.impact_maps.get(instrument)
    }

    fn get_grid(&self) -> Option<&[Position]> {
        Some(&self.allowed_positions)
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

        let weight_factor = self.problem.musicians.len() as f32 / max_instrument as f32;
        let max_position_count = 1000000.0 / max_instrument as f32 / weight_factor;
        let min_position_count = self.problem.musicians.len() as f32 * 4.0;
        const MIN_DELTA: f32 = 0.5;
        let mut delta = MIN_DELTA;
        loop {
            let position_count = (((until_x - x) / delta) + 1.0) * (((until_y - y) / delta) + 1.0);
            if position_count < min_position_count {
                if delta > MIN_DELTA {
                    delta /= 1.1;
                }
                break;
            }
            if position_count < max_position_count {
                break;
            }
            delta *= 1.01;
        }

        println!("greedy: delta = {}", delta);

        let mut curr_y = y;
        while curr_y <= until_y {
            let mut curr_x = x;
            while curr_x <= until_x {
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

        let mut remaining_instruments = BTreeMap::new();
        for idx in &self.remaining_musicians {
            let instrument = self.problem.musicians[*idx];
            if !remaining_instruments.contains_key(&instrument) {
                remaining_instruments.insert(instrument, 0);
            } else {
                *remaining_instruments.get_mut(&instrument).unwrap() += 1;
            }
        }

        for i in remaining_instruments.keys() {
            let impact_map = &self.impact_maps[i];
            if best_score < impact_map.best_score.0 {
                best_score = impact_map.best_score.0;
                best_instrument = *i;
                best_pos_idx = impact_map.best_score_pos_idx;
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
        let mut new_taken_positions = HashSet::new();
        for (idx, pos) in self.allowed_positions.iter_mut().enumerate() {
            let x = pos.p.x - best_pos.p.x;
            let y = pos.p.y - best_pos.p.y;
            let dist = (x * x + y * y).sqrt();
            if dist <= 10.0 {
                pos.taken = true;
                new_taken_positions.insert(idx);
            }
        }

        if remaining_instruments[&best_instrument] == 0 {
            remaining_instruments.remove(&best_instrument);
        }

        let blocked_positions = ImpactMap::calculate_blocked_positions(
            &best_pos.p,
            &self.problem.attendees,
            &self.allowed_positions,
        );
        self.impact_maps.par_iter_mut().for_each(|(i, im)| {
            if !remaining_instruments.contains_key(i) {
                return;
            }
            im.update(
                &best_instrument,
                &self.problem.attendees,
                &new_taken_positions,
                &blocked_positions,
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
