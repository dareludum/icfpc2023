use crate::{
    dto::{Attendee, ProblemDto},
    solvers::Problem,
};

use super::Scorer;

#[derive(Clone)]
pub struct ApproximateScorer {
    approximate_problem: ProblemDto,
}

impl Scorer for ApproximateScorer {
    fn score(
        &self,
        _problem: &crate::dto::ProblemDto,
        placements: &[crate::dto::Point2D],
        volumes: Option<&Vec<f32>>,
    ) -> crate::solvers::Score {
        // NOTE: Same as the existing scorer, but less data (at least for now)
        super::new_scorer::NewScorer.score(&self.approximate_problem, placements, volumes)
    }
}

impl ApproximateScorer {
    pub fn new(problem: &Problem) -> Self {
        // let mut unified_attendees = vec![];
        // let mut unified_attendee = problem.data.attendees[0]
        let mut unified_attendee = problem
            .data
            .attendees
            .iter()
            .cloned()
            .reduce(|a, b| Attendee {
                x: a.x + b.x,
                y: a.y + b.y,
                tastes: a
                    .tastes
                    .iter()
                    .zip(b.tastes.iter())
                    .map(|(a, b)| a + b)
                    .collect(),
            })
            .unwrap();
        unified_attendee.x /= problem.data.attendees.len() as f32;
        unified_attendee.y /= problem.data.attendees.len() as f32;

        ApproximateScorer {
            approximate_problem: ProblemDto {
                attendees: vec![unified_attendee],
                ..problem.data.clone()
            },
        }
    }
}
