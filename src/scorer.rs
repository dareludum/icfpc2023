use crate::{
    dto::{Attendee, Placement, ProblemDto, SolutionDto},
    solvers::Score,
};

fn calculate_distance(attendee: &Attendee, placement: &Placement) -> f32 {
    let x = attendee.x - placement.x;
    let y = attendee.y - placement.y;

    (x * x + y * y).sqrt()
}

fn calculate_impact(attendee: &Attendee, instrument: i32, distance: f32) -> f64 {
    1000000 as f64 * attendee.tastes[instrument as usize] as f64 / (distance * distance) as f64
}

fn calculate_attendee_happiness(
    attendee: &Attendee,
    musicians: &[i32],
    placements: &[Placement],
) -> f64 {
    let mut happiness = 0.0;

    for musician in musicians {
        let is_blocked = musicians.iter().any(|m| {
            *m != *musician
                && is_sound_blocked(
                    &placements[*musician as usize],
                    &placements[*m as usize],
                    attendee,
                )
        });

        if is_blocked {
            continue;
        }

        let distance = calculate_distance(attendee, &placements[*musician as usize]);
        happiness += calculate_impact(attendee, *musician, distance);
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

pub fn score_musician(attendees: &[Attendee], placement: &Placement, instrument: i32) -> Score {
    let mut score = 0.0;

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
    let mut score = 0.0;

    for attendee in &problem.attendees {
        score += calculate_attendee_happiness(attendee, &problem.musicians, &solution.placements);
    }

    Score(score)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dto::{Attendee, Placement};
    use crate::solvers::Score;

    #[test]
    fn test_calculate_attendee_happiness_no_blocked() {
        let attendees = vec![
            Attendee {
                x: 1.0,
                y: 1.0,
                tastes: vec![100, 200, 300],
            },
            Attendee {
                x: 2.0,
                y: 2.0,
                tastes: vec![150, 250, 350],
            },
        ];

        let musicians = vec![0, 1, 2];

        let placements = vec![
            Placement { x: 0.0, y: 0.0 },
            Placement { x: 1.0, y: 1.0 },
            Placement { x: 2.0, y: 2.0 },
        ];

        let happiness = calculate_attendee_happiness(&attendees[0], &musicians, &placements);

        assert_eq!(happiness, 1499998.0);
    }

    #[test]
    fn test_calculate_attendee_happiness_with_blocked() {
        let attendees = vec![
            Attendee {
                x: 1.0,
                y: 1.0,
                tastes: vec![100, 200, 300],
            },
            Attendee {
                x: 2.0,
                y: 2.0,
                tastes: vec![150, 250, 350],
            },
        ];

        let musicians = vec![0, 1, 2];

        let placements = vec![
            Placement { x: 0.0, y: 0.0 },
            Placement { x: 1.0, y: 1.0 },
            Placement { x: 1.5, y: 1.5 }, // This musician is blocked by the previous one
        ];

        let happiness = calculate_attendee_happiness(&attendees[0], &musicians, &placements);

        assert_eq!(happiness, 1500000.0); // The blocked musician's impact is skipped
    }

    #[test]
    fn test_calculate_attendee_happiness_empty_musicians() {
        let attendees = vec![
            Attendee {
                x: 1.0,
                y: 1.0,
                tastes: vec![100, 200, 300],
            },
            Attendee {
                x: 2.0,
                y: 2.0,
                tastes: vec![150, 250, 350],
            },
        ];

        let musicians = vec![];

        let placements = vec![
            Placement { x: 0.0, y: 0.0 },
            Placement { x: 1.0, y: 1.0 },
            Placement { x: 2.0, y: 2.0 },
        ];

        let happiness = calculate_attendee_happiness(&attendees[0], &musicians, &placements);

        assert_eq!(happiness, 0.0); // No musicians, so happiness is 0
    }
}
