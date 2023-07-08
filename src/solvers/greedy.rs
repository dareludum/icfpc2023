use log::debug;
use std::collections::{BTreeMap, HashMap, HashSet};

use rayon::prelude::*;

use crate::{
    common::Grid,
    dto::{Instrument, Point2D, ProblemDto, SolutionDto},
    geometry::distance2,
    scorer::{ImpactMap, PillarBlockageMap},
};

use super::{Problem, Solver};

#[derive(Default, Clone)]
pub struct Greedy {
    problem: ProblemDto,
    grid: Grid,
    placements: Vec<Point2D>,
    remaining_musicians: HashSet<usize>,
    impact_maps: HashMap<Instrument, ImpactMap>,
    pillar_blockage_map: PillarBlockageMap,
}

impl Solver for Greedy {
    fn name(&self) -> &'static str {
        "greedy"
    }

    fn get_impact_map(&self, instrument: &Instrument) -> Option<&ImpactMap> {
        self.impact_maps.get(instrument)
    }

    fn get_grid(&self) -> Option<&Grid> {
        Some(&self.grid)
    }

    fn initialize(&mut self, problem: &Problem) {
        self.problem = problem.data.clone();

        self.grid = Grid::new(&self.problem);

        let max_instrument = self.problem.musicians.iter().map(|i| i.0).max().unwrap();

        for i in 0..self.problem.musicians.len() {
            self.remaining_musicians.insert(i);
            self.placements.push(Point2D {
                x: f32::NAN,
                y: f32::NAN,
            });
        }

        debug!("greedy: computing pillar blockage map");
        self.pillar_blockage_map =
            PillarBlockageMap::new(&self.grid, &self.problem.pillars, &self.problem.attendees);
        debug!(
            "greedy: {} blocked pairs",
            self.pillar_blockage_map.blocked_positions.len()
        );

        debug!("greedy: computing impact maps");
        self.impact_maps = (0..=max_instrument)
            .map(Instrument)
            .collect::<Vec<_>>()
            .par_iter()
            .map(|i| {
                let impact_map = ImpactMap::new(
                    i,
                    &self.problem.attendees,
                    &self.grid,
                    &self.pillar_blockage_map,
                );
                (*i, impact_map)
            })
            .collect();

        debug!("greedy: initialized");
    }

    fn solve_step(&mut self) -> (SolutionDto, bool) {
        let mut best_pos_idx = usize::MAX;
        let mut best_instrument = Instrument(u32::MAX);
        let mut best_score = i64::MIN;

        let mut remaining_instruments = BTreeMap::new();
        for idx in &self.remaining_musicians {
            let instrument = self.problem.musicians[*idx];
            if let std::collections::btree_map::Entry::Vacant(e) =
                remaining_instruments.entry(instrument)
            {
                e.insert(0);
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

        let best_pos = self.grid.positions[best_pos_idx];
        self.placements[idx] = best_pos.p;

        // Remove the positions near the new musician
        let mut new_taken_positions = HashSet::new();
        for (idx, pos) in self.grid.positions.iter_mut().enumerate() {
            let dist2 = distance2(pos, &best_pos);
            if dist2 <= 100.0 {
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
            &self.grid,
        );
        self.impact_maps.par_iter_mut().for_each(|(i, im)| {
            if !remaining_instruments.contains_key(i) {
                return;
            }
            im.update(
                &best_instrument,
                &self.problem.attendees,
                &self.grid,
                &new_taken_positions,
                &blocked_positions,
                &self.pillar_blockage_map,
            );
        });

        debug!("greedy: {} musicians left", self.remaining_musicians.len());

        (
            SolutionDto {
                placements: self.placements.clone(),
            },
            self.remaining_musicians.is_empty(),
        )
    }
}
