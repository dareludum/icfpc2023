use std::{
    collections::{HashMap, HashSet},
    iter::repeat,
};

use rayon::prelude::*;

use crate::{
    common::Grid,
    dto::{Attendee, Instrument, PillarDto, Point2D, ProblemDto},
    geometry::{distance2, line_circle_intersection, Coords2D},
    solvers::Score,
};

fn calculate_impact(attendee: &Attendee, instrument: &Instrument, placement: &Point2D) -> i64 {
    let distance_square = distance2(placement, attendee);

    let impact =
        1000000_f64 * attendee.tastes[instrument.0 as usize] as f64 / distance_square as f64;

    impact.ceil() as i64
}

fn calculate_closeness_factor(placement: Point2D, same_instrument_placements: &[Point2D]) -> f64 {
    let mut factor = 0.0f64;

    for other_placement in same_instrument_placements {
        if placement == *other_placement {
            continue;
        }

        factor += 1.0 / (distance2(&placement, other_placement).sqrt() as f64);
    }

    1.0 + factor
}

fn calculate_attendee_happiness(
    attendee: &Attendee,
    musicians: &[Instrument],
    placements: &[Point2D],
    pillars: &[PillarDto],
    closeness_factors: &[f64],
) -> i64 {
    let mut happiness = 0;

    'hap_loop: for i in 0..musicians.len() {
        for other_i in 0..musicians.len() {
            if other_i == i {
                continue;
            }

            if is_sound_blocked(&placements[i], &placements[other_i], 5.0, attendee) {
                continue 'hap_loop;
            }
        }

        for pillar in pillars {
            if is_sound_blocked(&placements[i], &pillar.center, pillar.radius, attendee) {
                continue 'hap_loop;
            }
        }

        let impact = calculate_impact(attendee, &musicians[i], &placements[i]);

        if !closeness_factors.is_empty() {
            happiness += (closeness_factors[i] * impact as f64).ceil() as i64;
        } else {
            happiness += impact;
        }
    }

    happiness
}

fn is_sound_blocked(
    pos: &impl Coords2D,
    blocker_center: &impl Coords2D,
    blocker_radius: f32,
    attendee: &impl Coords2D,
) -> bool {
    line_circle_intersection(attendee, pos, blocker_center, blocker_radius)
}

fn calculate_closeness_factors(musicians: &[Instrument], placements: &[Point2D]) -> Vec<f64> {
    let mut closeness_factors = vec![];

    let placements_by_instrument = musicians.iter().zip(placements.iter()).fold(
        HashMap::new(),
        |mut acc, (instrument, placement)| {
            acc.entry(*instrument).or_insert(vec![]).push(*placement);
            acc
        },
    );

    for i in 0..musicians.len() {
        let closeness_factor =
            calculate_closeness_factor(placements[i], &placements_by_instrument[&musicians[i]]);
        closeness_factors.push(closeness_factor);
    }

    closeness_factors
}
pub fn score(problem: &ProblemDto, placements: &[Point2D]) -> Score {
    // if there are pillars then it is a task from spec v2
    let closeness_factors = if problem.pillars.is_empty() {
        vec![]
    } else {
        calculate_closeness_factors(&problem.musicians, placements)
    };

    Score(
        problem
            .attendees
            .par_iter()
            .map(|attendee| {
                calculate_attendee_happiness(
                    attendee,
                    &problem.musicians,
                    placements,
                    &problem.pillars,
                    &closeness_factors,
                )
            })
            .sum(),
    )
}

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
