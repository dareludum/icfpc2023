use crate::{
    dto::{Point2D, ProblemDto},
    kdtree::{KDTree, AABB},
};
use nalgebra::Vector2;

pub struct Collider<'sol, 'pro> {
    placements: &'sol [Point2D],
    problem: &'pro ProblemDto,
    pillar_count: usize,
    kdtree: KDTree,
}

#[derive(PartialEq, Eq, Debug, Copy, Clone)]
pub enum Obstacle {
    Musician(usize),
    Pillar(usize),
}

impl<'sol, 'pro> Collider<'sol, 'pro> {
    fn lookup_obstacle(&self, index: usize) -> Obstacle {
        if index < self.pillar_count {
            return Obstacle::Pillar(index);
        }
        Obstacle::Musician(index - self.pillar_count)
    }

    fn get_circle(&self, obstacle: Obstacle) -> (Vector2<f32>, f32) {
        match obstacle {
            Obstacle::Musician(i) => (self.placements[i].as_vec(), 5.0),
            Obstacle::Pillar(i) => {
                let pillar = &self.problem.pillars[i];
                (pillar.as_vec(), pillar.radius)
            }
        }
    }

    pub fn new(problem: &'pro ProblemDto, placements: &'sol [Point2D]) -> Collider<'sol, 'pro> {
        let mut bbs = vec![];
        bbs.reserve_exact(problem.musicians.len() + problem.pillars.len());

        for pillar in &problem.pillars {
            let pos = Vector2::new(pillar.center.0, pillar.center.1);
            bbs.push(circle_bounds(&pos, pillar.radius));
        }

        for musician_pos in placements {
            bbs.push(circle_bounds(&musician_pos.as_vec(), 5.0));
        }

        Collider {
            problem,
            placements,
            pillar_count: 0usize,
            kdtree: KDTree::build(&bbs),
        }
    }

    fn intersection_candidates(&self, origin: &Vector2<f32>, dir: &Vector2<f32>) -> Vec<Obstacle> {
        self.kdtree
            .intersect(origin, dir)
            .iter()
            .map(|i| self.lookup_obstacle(*i))
            .collect()
    }

    fn filter_candidates(
        &self,
        candidates: Vec<Obstacle>,
        attendee_i: usize,
        musician_i: usize,
    ) -> Vec<Obstacle> {
        let attendee_location = self.problem.attendees[attendee_i].as_vec();
        let musician_location = self.placements[musician_i].as_vec();

        candidates
            .into_iter()
            .filter(|obstacle| {
                // skip collision check with the target musician
                if *obstacle == Obstacle::Musician(musician_i) {
                    false
                } else {
                    let (center, radius) = self.get_circle(*obstacle);
                    crate::geometry::line_circle_intersection(
                        &attendee_location,
                        &musician_location,
                        &center,
                        radius,
                    )
                }
            })
            .collect()
    }

    pub fn intersect(&self, attendee_i: usize, musician_i: usize) -> Vec<Obstacle> {
        let attendee_location = self.problem.attendees[attendee_i].as_vec();
        let musician_location = self.placements[musician_i].as_vec();

        let candidates = self.intersection_candidates(
            &attendee_location,
            &(musician_location - attendee_location).normalize(),
        );
        self.filter_candidates(candidates, attendee_i, musician_i)
    }

    pub fn is_hidden(&self, attendee_i: usize, musician_i: usize) -> bool {
        !self.intersect(attendee_i, musician_i).is_empty()
    }
}

fn circle_bounds(position: &Vector2<f32>, radius: f32) -> AABB {
    let min = Vector2::new(position.x - radius, position.y - radius);
    let max = Vector2::new(position.x + radius, position.y + radius);
    AABB::new(min, max)
}
