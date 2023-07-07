use rayon::prelude::*;

use crate::{
    common::Position,
    dto::{Attendee, Instrument, Placement, ProblemDto, SolutionDto},
    solvers::Score,
};

fn calculate_distance(attendee: &Attendee, placement: &Placement) -> f32 {
    let x = attendee.x - placement.x;
    let y = attendee.y - placement.y;

    (x * x + y * y).sqrt()
}

fn calculate_impact(attendee: &Attendee, instrument: &Instrument, distance: f32) -> i64 {
    let impact =
        1000000_f64 * attendee.tastes[instrument.0 as usize] as f64 / (distance * distance) as f64;

    impact.ceil() as i64
}

fn calculate_attendee_happiness(
    attendee: &Attendee,
    musicians: &[Instrument],
    placements: &[Placement],
) -> i64 {
    let mut happiness = 0;

    for i in 0..musicians.len() {
        let is_blocked = musicians
            .iter()
            .enumerate()
            .any(|(j, _)| j != i && is_sound_blocked(&placements[i], &placements[j], attendee));

        if is_blocked {
            continue;
        }

        let distance = calculate_distance(attendee, &placements[i]);
        happiness += calculate_impact(attendee, &musicians[i], distance);
    }

    happiness
}

fn is_sound_blocked(k: &Placement, k_1: &Placement, attendee: &Attendee) -> bool {
    let r: f32 = 5.0;

    // Step 1: Find the equation of the line: y = mx + c
    let m = (attendee.y - k.y) / (attendee.x - k.x);
    let c = k.y - m * k.x;

    // Step 2: Substitute y in the equation of the circle and get a quadratic equation: Ax^2 + Bx + C = 0
    let a = 1.0 + m.powi(2);
    let b = 2.0 * m * c - 2.0 * k_1.x - 2.0 * k_1.y * m;
    let cc = k_1.x.powi(2) + k_1.y.powi(2) + c.powi(2) - r.powi(2) - 2.0 * c * k_1.y;

    // Step 3: Check if the equation has real roots by computing the discriminant
    let discriminant = b.powi(2) - 4.0 * a * cc;

    // If discriminant < 0, no real roots, so the line doesn't intersect the circle
    if discriminant < 0.0 {
        return false;
    }

    // Otherwise, compute the two intersection points and check if they lie within the segment
    let t1 = (-b - discriminant.sqrt()) / (2.0 * a);
    let t2 = (-b + discriminant.sqrt()) / (2.0 * a);

    let x_min = if k.x <= attendee.x { k.x } else { attendee.x };
    let x_max = if k.x >= attendee.x { k.x } else { attendee.x };

    (x_min <= t1 && t1 <= x_max) || (x_min <= t2 && t2 <= x_max)
}

pub fn score_instrument(
    attendees: &[Attendee],
    placement: &Placement,
    instrument: &Instrument,
) -> Score {
    let mut score = 0;

    for attendee in attendees {
        score += calculate_impact(
            attendee,
            instrument,
            calculate_distance(attendee, placement),
        );
    }

    Score(score)
}

pub fn score(problem: &ProblemDto, solution: &SolutionDto) -> Score {
    let mut score = 0;

    for attendee in &problem.attendees {
        score += calculate_attendee_happiness(attendee, &problem.musicians, &solution.placements);
    }

    Score(score)
}

#[derive(Clone)]
pub struct ImpactMap {
    pub scores: Vec<Score>,
    pub best_score_pos_idx: usize,
    pub best_score: Score,
}

impl ImpactMap {
    pub fn new(instrument: &Instrument, attendees: &[Attendee], grid: &[Position]) -> Self {
        let mut scores = vec![];
        for pos in grid {
            let score = score_instrument(&attendees, &pos.p, instrument);
            scores.push(score);
        }

        let (best_score_pos_idx, best_score) = Self::get_best_score(&scores, grid);

        ImpactMap {
            scores,
            best_score_pos_idx,
            best_score,
        }
    }

    fn get_best_score(scores: &[Score], grid: &[Position]) -> (usize, Score) {
        let best = scores
            .iter()
            .zip(grid)
            .enumerate()
            .filter(|(_idx, (_s, p))| !p.taken)
            .max_by_key(|(_idx, (s, _p))| s.0)
            .unwrap();
        (best.0, *best.1 .0)
    }

    pub fn calculate_blocked_positions(
        new_pos: &Placement,
        attendees: &[Attendee],
        grid: &[Position],
    ) -> Vec<(usize, usize)> {
        grid.par_iter()
            .enumerate()
            // We don't care for those anymore, so can keep them invalid
            .filter(|(_idx, pos)| !pos.taken)
            .flat_map(|(idx, pos)| {
                let mut result = vec![];
                for (idx_attendee, attendee) in attendees.iter().enumerate() {
                    if is_sound_blocked(&pos.p, new_pos, attendee) {
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
        blocked_positions: &[(usize, usize)],
        grid: &[Position],
    ) {
        let mut needs_best_score_update = false;
        for (idx, idx_attendee) in blocked_positions {
            let pos = &grid[*idx];
            let attendee = &attendees[*idx_attendee];
            self.scores[*idx].0 -=
                calculate_impact(attendee, instrument, calculate_distance(attendee, &pos.p));
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

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use crate::dto::{Attendee, Placement};

//     #[test]
//     fn test_calculate_attendee_happiness_no_blocked() {
//         let attendees = vec![
//             Attendee {
//                 x: 1.0,
//                 y: 1.0,
//                 tastes: vec![100.0, 200.0, 300.0],
//             },
//             Attendee {
//                 x: 2.0,
//                 y: 2.0,
//                 tastes: vec![150.0, 250.0, 350.0],
//             },
//         ];

//         let musicians = vec![Instrument(0), Instrument(1), Instrument(2)];

//         let placements = vec![
//             Placement { x: 0.0, y: 0.0 },
//             Placement { x: 1.0, y: 1.0 },
//             Placement { x: 2.0, y: 2.0 },
//         ];

//         let happiness = calculate_attendee_happiness(&attendees[0], &musicians, &placements);

//         assert_eq!(happiness, 1499998.0);
//     }

//     #[test]
//     fn test_calculate_attendee_happiness_with_blocked() {
//         let attendees = vec![
//             Attendee {
//                 x: 1.0,
//                 y: 1.0,
//                 tastes: vec![100.0, 200.0, 300.0],
//             },
//             Attendee {
//                 x: 2.0,
//                 y: 2.0,
//                 tastes: vec![150.0, 250.0, 350.0],
//             },
//         ];

//         let musicians = vec![Instrument(0), Instrument(1), Instrument(2)];

//         let placements = vec![
//             Placement { x: 0.0, y: 0.0 },
//             Placement { x: 1.0, y: 1.0 },
//             Placement { x: 1.5, y: 1.5 }, // This musician is blocked by the previous one
//         ];

//         let happiness = calculate_attendee_happiness(&attendees[0], &musicians, &placements);

//         assert_eq!(happiness, 1500000.0); // The blocked musician's impact is skipped
//     }

//     #[test]
//     fn test_calculate_attendee_happiness_empty_musicians() {
//         let attendees = vec![
//             Attendee {
//                 x: 1.0,
//                 y: 1.0,
//                 tastes: vec![100.0, 200.0, 300.0],
//             },
//             Attendee {
//                 x: 2.0,
//                 y: 2.0,
//                 tastes: vec![150.0, 250.0, 350.0],
//             },
//         ];

//         let musicians = vec![];

//         let placements = vec![
//             Placement { x: 0.0, y: 0.0 },
//             Placement { x: 1.0, y: 1.0 },
//             Placement { x: 2.0, y: 2.0 },
//         ];

//         let happiness = calculate_attendee_happiness(&attendees[0], &musicians, &placements);

//         assert_eq!(happiness, 0.0); // No musicians, so happiness is 0
//     }
// }
