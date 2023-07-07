use std::collections::HashSet;

use nalgebra::Vector2;
use rayon::prelude::*;

use crate::{
    common::Grid,
    dto::{Attendee, Instrument, Point2D, ProblemDto, SolutionDto},
    solvers::Score,
};

fn calculate_impact(attendee: &Attendee, instrument: &Instrument, placement: &Point2D) -> i64 {
    let x = attendee.x - placement.x;
    let y = attendee.y - placement.y;

    let distance_square = x * x + y * y;

    let impact =
        1000000_f64 * attendee.tastes[instrument.0 as usize] as f64 / distance_square as f64;

    impact.ceil() as i64
}

fn calculate_attendee_happiness(
    attendee: &Attendee,
    musicians: &[Instrument],
    placements: &[Point2D],
) -> i64 {
    let mut happiness = 0;
    // println!("{:?}", attendee);

    'hap_loop: for i in 0..musicians.len() {
        // println!("musician {:?} instrument {:?}", placements[i], musicians[i]);

        for other_i in 0..musicians.len() {
            if other_i == i {
                continue;
            }

            if is_sound_blocked_2(&placements[i], &placements[other_i], attendee) {
                continue 'hap_loop;
            }
        }

        let impact = calculate_impact(attendee, &musicians[i], &placements[i]);
        happiness += impact;
    }

    happiness
}

fn is_sound_blocked_2(k: &Point2D, k_1: &Point2D, attendee: &Attendee) -> bool {
    let r: f32 = 5.0;
    let circle_center = k_1;
    let line_start = k;
    let line_end = attendee;

    // Create vector from the start of the line to the center of the circle
    let start_to_center = Point2D {
        x: circle_center.x - line_start.x,
        y: circle_center.y - line_start.y,
    };

    // Create the vector that represents the line
    let line_vector = Point2D {
        x: line_end.x - line_start.x,
        y: line_end.y - line_start.y,
    };

    // Calculate the squared length of the line
    let line_len_sq = line_vector.x * line_vector.x + line_vector.y * line_vector.y;

    // Calculate the dot product of the start_to_center and the line_vector
    let dot_product = start_to_center.x * line_vector.x + start_to_center.y * line_vector.y;

    // Calculate the closest Placement on the line to the center of the circle
    let t = dot_product / line_len_sq;

    // If the closest Placement is outside the line segment, return false
    if !(0.0..=1.0).contains(&t) {
        return false;
    }

    // Calculate the coordinates of the closest Placement
    let closest_point = Point2D {
        x: line_start.x + t * line_vector.x,
        y: line_start.y + t * line_vector.y,
    };

    // Calculate the vector from the closest Placement to the center of the circle
    let closest_to_center = Point2D {
        x: circle_center.x - closest_point.x,
        y: circle_center.y - closest_point.y,
    };

    // Calculate the squared length of the vector
    let closest_to_center_len_sq =
        closest_to_center.x * closest_to_center.x + closest_to_center.y * closest_to_center.y;

    // If the squared length is less than r squared, the line intersects the circle
    closest_to_center_len_sq <= r * r
}

fn is_sound_blocked(k: &Point2D, k_1: &Point2D, attendee: &Attendee) -> bool {
    line_circle_intersection(attendee.into(), k.into(), k_1.into(), 5.0)
}

fn line_circle_intersection(
    a: Vector2<f32>,
    b: Vector2<f32>,
    circle_center: Vector2<f32>,
    circle_radius: f32,
) -> bool {
    assert!((a - circle_center).norm() > circle_radius);
    assert!((b - circle_center).norm() > circle_radius);
    let a_b = b - a;
    let a_b_norm = a_b.norm();
    assert!(a_b_norm > circle_radius);
    let a_b_dir = a_b / a_b_norm;
    let a_circle = circle_center - a;
    let projected_len = a_b_dir.dot(&a_circle);
    if projected_len < 0. || projected_len > a_b_norm {
        return false;
    }
    let circle_deviation = (a_circle.norm_squared() - projected_len * projected_len).sqrt();
    circle_deviation < circle_radius
}

#[cfg(test)]
mod tests {
    use nalgebra::Vector2;

    use crate::scorer::line_circle_intersection;

    #[test]
    fn test_cases() {
        let tests = [
            ((0., 0.), (1., 0.), (0.5, 0.), 0.01, true),
            ((0., 0.), (10., 0.), (5., 5.), 3., false),
            ((0., 0.), (10., 0.), (5., 5.), 5.1, true),
            ((0., 0.), (10., 0.), (5., 5.), 4.9, false),
            // circle alongside the line
            ((-2.42, -3.58), (14.76, 6.64), (7.1, 8.44), 5., false),
            ((-2.42, -3.58), (14.76, 6.64), (7.1, 7.44), 5., true),
            // circle slightly behind the line
            ((-2.42, -3.58), (14.76, 6.64), (17.56, 11.7), 5., false),
            ((1100., 800.), (1100., 150.), (1100., 100.), 5., false),
        ];

        for (a, b, c, r, int) in tests {
            let res = line_circle_intersection(
                Vector2::new(a.0, a.1),
                Vector2::new(b.0, b.1),
                Vector2::new(c.0, c.1),
                r,
            );
            assert_eq!(
                res, int,
                "intersection of segment from {:?} to {:?} by {:?} r {} should be {}",
                a, b, c, r, int
            );
        }
    }
}

pub fn score_instrument(
    attendees: &[Attendee],
    placement: &Point2D,
    instrument: &Instrument,
) -> Score {
    let mut score = 0;

    for attendee in attendees {
        score += calculate_impact(attendee, instrument, placement);
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
    pub fn new(instrument: &Instrument, attendees: &[Attendee], grid: &Grid) -> Self {
        let mut scores = vec![];
        for pos in &grid.positions {
            let score = score_instrument(attendees, &pos.p, instrument);
            scores.push(score);
        }

        let (best_score_pos_idx, best_score) = Self::get_best_score(&scores, grid);

        ImpactMap {
            scores,
            best_score_pos_idx,
            best_score,
        }
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
        grid: &Grid,
        new_taken_positions: &HashSet<usize>,
        blocked_positions: &[(usize, usize)],
    ) {
        let mut needs_best_score_update = new_taken_positions.contains(&self.best_score_pos_idx);
        for (idx, idx_attendee) in blocked_positions {
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
