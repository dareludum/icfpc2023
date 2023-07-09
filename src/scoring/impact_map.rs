use std::{collections::HashSet, iter::repeat};

use rayon::prelude::*;

use crate::{
    common::Grid,
    dto::{Attendee, Instrument, PillarDto, Point2D},
    solvers::Score,
};

use super::scorer::{calculate_impact, is_sound_blocked};

#[derive(Clone)]
pub struct ImpactMap {
    pub scores: Vec<Score>,
    pub best_score_pos_idx: usize,
    pub best_score: Score,
}

impl ImpactMap {
    pub fn new(
        instrument: &Instrument,
        attendees: &[Attendee],
        grid: &Grid,
        pillar_blockage: &PillarBlockageMap,
    ) -> Self {
        let mut scores = vec![];
        for (idx_pos, pos) in grid.positions.iter().enumerate() {
            let score =
                Self::score_instrument(attendees, idx_pos, &pos.p, instrument, pillar_blockage);
            scores.push(score);
        }

        let (best_score_pos_idx, best_score) = Self::get_best_score(&scores, grid);

        ImpactMap {
            scores,
            best_score_pos_idx,
            best_score,
        }
    }

    fn score_instrument(
        attendees: &[Attendee],
        idx_pos: usize,
        placement: &Point2D,
        instrument: &Instrument,
        pillar_blockage: &PillarBlockageMap,
    ) -> Score {
        let mut score = 0;

        for (idx, attendee) in attendees.iter().enumerate() {
            if !pillar_blockage.is_sound_blocked(idx_pos, idx) {
                score += calculate_impact(attendee, instrument, placement);
            }
        }

        Score(score)
    }

    fn get_best_score(scores: &[Score], grid: &Grid) -> (usize, Score) {
        let best = scores
            .iter()
            .zip(&grid.positions)
            .enumerate()
            .filter(|(_idx, (_s, p))| !p.taken)
            .max_by_key(|(_idx, (s, _p))| s.0)
            .unwrap();
        (best.0, *best.1 .0)
    }

    pub fn calculate_blocked_positions(
        new_pos: &Point2D,
        attendees: &[Attendee],
        grid: &Grid,
    ) -> Vec<(usize, usize)> {
        grid.positions
            .par_iter()
            .enumerate()
            // We don't care for those anymore, so can keep them invalid
            .filter(|(_idx, pos)| !pos.taken)
            .flat_map(|(idx, pos)| {
                let mut result = vec![];
                for (idx_attendee, attendee) in attendees.iter().enumerate() {
                    if is_sound_blocked(&pos.p, new_pos, 5.0, attendee) {
                        result.push((idx, idx_attendee));
                    }
                }
                result
            })
            .collect()
    }

    pub fn update(
        &mut self,
        instrument: &Instrument,
        attendees: &[Attendee],
        grid: &Grid,
        new_taken_positions: &HashSet<usize>,
        blocked_positions: &[(usize, usize)],
        pillar_blockage: &PillarBlockageMap,
    ) {
        let mut needs_best_score_update = new_taken_positions.contains(&self.best_score_pos_idx);
        for (idx, idx_attendee) in blocked_positions {
            if pillar_blockage.is_sound_blocked(*idx, *idx_attendee) {
                continue;
            }
            let pos = &grid.positions[*idx];
            let attendee = &attendees[*idx_attendee];
            self.scores[*idx].0 -= calculate_impact(attendee, instrument, &pos.p);
            if *idx == self.best_score_pos_idx {
                needs_best_score_update = true;
            }
        }
        if needs_best_score_update {
            let (best_score_pos_idx, best_score) = Self::get_best_score(&self.scores, grid);
            self.best_score_pos_idx = best_score_pos_idx;
            self.best_score = best_score;
        }
    }
}

#[derive(Default, Clone)]
pub struct PillarBlockageMap {
    pub blocked_positions: HashSet<(usize, usize)>,
}

impl PillarBlockageMap {
    pub fn new(grid: &Grid, pillars: &[PillarDto], attendees: &[Attendee]) -> Self {
        if pillars.is_empty() {
            return PillarBlockageMap::default();
        }
        let blocked_positions = (0..grid.positions.len())
            .flat_map(|idx_pos| repeat(idx_pos).zip(0..attendees.len()))
            .par_bridge()
            .filter(|(idx_pos, idx_attendee)| {
                pillars.iter().any(|p| {
                    is_sound_blocked(
                        &grid.positions[*idx_pos].p,
                        &p.center,
                        p.radius,
                        &attendees[*idx_attendee],
                    )
                })
            })
            .collect();

        PillarBlockageMap { blocked_positions }
    }

    pub fn is_sound_blocked(&self, idx_pos: usize, idx_attendee: usize) -> bool {
        self.blocked_positions.contains(&(idx_pos, idx_attendee))
    }
}
