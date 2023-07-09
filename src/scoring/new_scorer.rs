use rayon::prelude::*;

use crate::collider::Collider;
use crate::dto::Point2D;
use crate::{dto::ProblemDto, solvers::Score};

fn compute_closeness(problem: &ProblemDto, placements: &[Point2D]) -> Vec<f32> {
    // sort musicians by instrument
    let instrument_count: u32 = problem.musicians.iter().map(|ins| ins.0).max().unwrap() + 1;
    let mut musicians_per_ins: Vec<Vec<usize>> = (0..instrument_count).map(|_| vec![]).collect();
    for (musician_i, instrument) in problem.musicians.iter().enumerate() {
        musicians_per_ins[instrument.0 as usize].push(musician_i);
    }

    let mut musicians_closeness: Vec<f32> = vec![0f32; problem.musicians.len()];
    // for each music instrument
    for ins_musicians in musicians_per_ins.into_iter() {
        let mut ins_musicians_scores = vec![1f32; ins_musicians.len()];
        // compute all the pairwise distances between musicians using the instrument, and update their scores
        for (musician_ins_i, musician_i) in ins_musicians.iter().enumerate() {
            let musician_pos = placements[*musician_i].as_vec();
            for other_musician_ins_i in (musician_ins_i + 1)..ins_musicians.len() {
                let other_musician_i = ins_musicians[other_musician_ins_i];
                let other_musician_pos = placements[other_musician_i].as_vec();
                let distance = (musician_pos - other_musician_pos).norm();

                // add the result to the score of both musicians
                let closeness = 1f32 / distance;
                ins_musicians_scores[musician_ins_i] += closeness;
                ins_musicians_scores[other_musician_ins_i] += closeness;
            }
        }

        // store back the closeness of all musicians using the instrument into the global array
        for (musician_i, closeness) in ins_musicians
            .into_iter()
            .zip(ins_musicians_scores.into_iter())
        {
            musicians_closeness[musician_i] = closeness;
        }
    }

    musicians_closeness
}

#[allow(dead_code)]
pub fn new_score(
    problem: &ProblemDto,
    placements: &[Point2D],
    volumes: Option<&Vec<f32>>,
) -> Score {
    let collider = Collider::new(problem, placements);
    let has_pillars = !problem.pillars.is_empty();

    // compute the closeness score per musician
    let musicians_closeness = if has_pillars {
        compute_closeness(problem, placements)
    } else {
        vec![]
    };

    Score(
        problem
            .attendees
            .par_iter()
            .enumerate()
            .map(|(attendee_i, attendee)| {
                let mut attendee_score = 0i64;
                for (musician_i, instrument_i) in problem.musicians.iter().enumerate() {
                    if collider.is_hidden(attendee_i, musician_i) {
                        continue;
                    }
                    let taste: f32 = attendee.tastes[instrument_i.0 as usize];
                    let musician_location = placements[musician_i];
                    let distance_sq = (attendee.x - musician_location.x).powi(2)
                        + (attendee.y - musician_location.y).powi(2);
                    let impact = ((1_000_000f32 * taste) / distance_sq).ceil();
                    let volume = volumes.map(|vs| vs[musician_i]).unwrap_or(1.0);

                    let score = if has_pillars {
                        let closeness = musicians_closeness[musician_i];
                        (impact * closeness * volume).ceil() as i64
                    } else {
                        (impact * volume) as i64
                    };
                    attendee_score += score
                }
                attendee_score
            })
            .sum(),
    )
}
