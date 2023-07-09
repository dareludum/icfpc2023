use std::collections::HashMap;

use rayon::prelude::*;

use crate::{
    dto::{Attendee, Instrument, PillarDto, Point2D, ProblemDto},
    geometry::{distance2, line_circle_intersection, Coords2D},
    solvers::Score,
};

pub fn calculate_impact(attendee: &Attendee, instrument: &Instrument, placement: &Point2D) -> i64 {
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
    volumes: Option<&Vec<f32>>,
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

        let volume = volumes.map(|vs| vs[i]).unwrap_or(1.0) as f64;
        let impact = calculate_impact(attendee, &musicians[i], &placements[i]) as f64;

        if !closeness_factors.is_empty() {
            happiness += (volume * closeness_factors[i] * impact).ceil() as i64;
        } else {
            happiness += (volume * impact).ceil() as i64;
        }
    }

    happiness
}

pub fn is_sound_blocked(
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

pub fn score(problem: &ProblemDto, placements: &[Point2D], volumes: Option<&Vec<f32>>) -> Score {
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
                    volumes,
                )
            })
            .sum(),
    )
}
