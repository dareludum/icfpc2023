use crate::dto::{Point2D, ProblemDto};
use nalgebra::{Point2, Vector2};
use parry2d::{
    bounding_volume::Aabb,
    partitioning::{Qbvh, QbvhUpdateWorkspace},
    query::{visitors::RayIntersectionsVisitor, Ray},
};

pub struct Collider<'sol, 'pro> {
    placements: &'sol [Point2D],
    problem: &'pro ProblemDto,
    qbvh: Qbvh<usize>,
    workspace: QbvhUpdateWorkspace,
}

#[derive(PartialEq, Eq, Debug, Copy, Clone)]
pub enum Obstacle {
    Musician(usize),
    Pillar(usize),
}

fn lookup_obstacle(pillar_count: usize, index: usize) -> Obstacle {
    if index < pillar_count {
        return Obstacle::Pillar(index);
    }
    Obstacle::Musician(index - pillar_count)
}

fn get_circle(
    placements: &[Point2D],
    problem: &ProblemDto,
    obstacle: Obstacle,
) -> (Vector2<f32>, f32) {
    match obstacle {
        Obstacle::Musician(i) => (placements[i].as_vec(), 5.0),
        Obstacle::Pillar(i) => {
            let pillar = &problem.pillars[i];
            (pillar.as_vec(), pillar.radius)
        }
    }
}

impl<'sol, 'pro> Collider<'sol, 'pro> {
    fn pillar_count(&self) -> usize {
        self.problem.pillars.len()
    }

    /// Re-fetches and updates the position of nodes
    fn refit(&mut self) {
        let margin = 0.002f32;
        let pillar_count = self.pillar_count();
        self.qbvh.refit(margin, &mut self.workspace, |index| {
            let obstacle = lookup_obstacle(pillar_count, *index);
            let (center, radius) = get_circle(self.placements, self.problem, obstacle);
            circle_bounds(&center, radius)
        });
        self.qbvh.rebalance(margin, &mut self.workspace);
    }

    pub fn new(problem: &'pro ProblemDto, placements: &'sol [Point2D]) -> Collider<'sol, 'pro> {
        let node_count = problem.musicians.len() + problem.pillars.len();

        let mut qbvh: Qbvh<usize> = Qbvh::new();
        let workspace = QbvhUpdateWorkspace::default();

        for i in 0..node_count {
            qbvh.pre_update_or_insert(i);
        }

        let mut collider = Collider {
            problem,
            placements,
            qbvh,
            workspace,
        };
        collider.refit();
        collider
    }

    pub fn is_hidden(&self, attendee_i: usize, musician_i: usize) -> bool {
        let attendee_location = &self.problem.attendees[attendee_i].as_vec();
        let musician_location = self.placements[musician_i].as_vec();
        let dir = (musician_location - attendee_location).normalize();
        let ray = Ray::new(Point2::new(attendee_location.x, attendee_location.y), dir);

        let mut callback = |node_index: &usize| {
            // TODO: perform intersection
            // skip collision check with the target musician
            let obstacle = lookup_obstacle(self.pillar_count(), *node_index);
            if obstacle == Obstacle::Musician(musician_i) {
                return true;
            }

            let (center, radius) = get_circle(self.placements, self.problem, obstacle);
            !crate::geometry::line_circle_intersection(
                attendee_location,
                &musician_location,
                &center,
                radius,
            )
        };

        let mut visitor = RayIntersectionsVisitor::new(&ray, std::f32::INFINITY, &mut callback);

        !self.qbvh.traverse_depth_first(&mut visitor)
    }
}

fn circle_bounds(position: &Vector2<f32>, radius: f32) -> Aabb {
    let min = Point2::new(position.x - radius, position.y - radius);
    let max = Point2::new(position.x + radius, position.y + radius);
    Aabb::new(min, max)
}
