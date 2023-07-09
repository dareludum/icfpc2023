use std::collections::HashSet;

use log::debug;
use rand::Rng;
use rayon::prelude::{IntoParallelRefIterator, ParallelIterator};

use crate::{
    dto::{Attendee, PillarDto, Point2D, ProblemDto},
    geometry::{distance2, line_circle_intersection, Coords2D},
    solvers::Problem,
};

#[derive(Clone, Copy)]
pub struct Position {
    pub p: Point2D,
    pub taken: bool,
}

impl Coords2D for Position {
    fn x(&self) -> f32 {
        self.p.x
    }

    fn y(&self) -> f32 {
        self.p.y
    }
}

// Rasterized stage to work with a grid of points instead of an infinite surface
#[derive(Default, Clone)]
pub struct Grid {
    pub positions: Vec<Position>,
}

impl Grid {
    pub fn new(problem: &Problem) -> Self {
        let data = &problem.data;
        let x = data.stage_bottom_left.0 + 10.0;
        let y = data.stage_bottom_left.1 + 10.0;
        let until_x = data.stage_bottom_left.0 + data.stage_width - 10.0;
        let until_y = data.stage_bottom_left.1 + data.stage_height - 10.0;

        debug!(
            "common({}): {} total musicians",
            problem.id,
            data.musicians.len()
        );
        let max_instrument = data.musicians.iter().map(|i| i.0).max().unwrap();
        debug!(
            "common({}): {} total instruments",
            problem.id, max_instrument
        );

        let weight_factor = data.musicians.len() as f32 / max_instrument as f32;
        let max_position_count = 1000000.0 / max_instrument as f32 / weight_factor;
        let min_position_count = data.musicians.len() as f32 * 4.0;
        const MIN_DELTA: f32 = 0.5;
        let mut delta = MIN_DELTA;
        loop {
            let position_count = (((until_x - x) / delta) + 1.0) * (((until_y - y) / delta) + 1.0);
            if position_count < min_position_count {
                if delta > MIN_DELTA {
                    delta /= 1.1;
                }
                break;
            }
            if position_count < max_position_count {
                break;
            }
            delta *= 1.01;
        }

        debug!("common({}): delta = {}", problem.id, delta);

        let mut positions = vec![];
        let mut curr_y = y;
        while curr_y <= until_y {
            let mut curr_x = x;
            while curr_x <= until_x {
                positions.push(Position {
                    p: Point2D {
                        x: curr_x,
                        y: curr_y,
                    },
                    taken: false,
                });
                curr_x += delta;
            }
            curr_y += delta;
        }

        debug!(
            "common({}): {} total positions",
            problem.id,
            positions.len()
        );

        Grid { positions }
    }

    pub fn recalculate_taken(&mut self, placements: &[Point2D]) {
        for pos in &mut self.positions {
            pos.taken = false;
        }

        for placement in placements {
            if placement.x.is_nan() {
                continue;
            }
            for pos in self.positions.iter_mut() {
                let x = pos.p.x - placement.x;
                let y = pos.p.y - placement.y;
                let dist = (x * x + y * y).sqrt();
                if dist <= 10.0 {
                    pos.taken = true;
                }
            }
        }
    }
}

pub fn calculate_invalid_positions(positions: &[Point2D], problem: &ProblemDto) -> HashSet<usize> {
    let min_x = problem.stage_bottom_left.x() + 10.0;
    let min_y = problem.stage_bottom_left.y() + 10.0;
    let max_x = problem.stage_bottom_left.x() + problem.stage_width - 10.0;
    let max_y = problem.stage_bottom_left.y() + problem.stage_height - 10.0;

    let mut result = HashSet::new();
    for i in 0..positions.len() {
        let pos0 = &positions[i];
        if pos0.x.is_nan() {
            continue;
        }
        if pos0.x < min_x || pos0.x > max_x || pos0.y < min_y || pos0.y > max_y {
            result.insert(i);
        }
        #[allow(clippy::needless_range_loop)]
        for j in 0..i {
            let pos1 = &positions[j];
            if pos1.x.is_nan() {
                continue;
            }
            if distance2(pos0, pos1) < 100.0 {
                result.insert(i);
                result.insert(j);
            }
        }
    }
    result
}

pub fn generate_random_placement(problem: &ProblemDto, placements: &[Point2D]) -> Point2D {
    let mut placement = get_random_coords(problem);
    let mut correct_placed = false;

    while !correct_placed {
        correct_placed = true;

        for other_placement in placements {
            if placement.distance(other_placement) < 10.0 {
                placement = get_random_coords(problem);
                correct_placed = false;
                break;
            }
        }
    }
    placement
}

pub fn get_random_coords(problem: &ProblemDto) -> Point2D {
    let mut rng = rand::thread_rng();

    Point2D {
        x: rng.gen_range(
            (problem.stage_bottom_left.0 + 10.0)
                ..problem.stage_bottom_left.0 + problem.stage_width - 10.0,
        ),
        y: rng.gen_range(
            (problem.stage_bottom_left.1 + 10.0)
                ..problem.stage_bottom_left.1 + problem.stage_height - 10.0,
        ),
    }
}

pub fn prune_attendees_and_pillars(problem: &ProblemDto) -> (Vec<Attendee>, Vec<PillarDto>) {
    debug!(
        "prune: trying to prune {} attendees ({} pillars)",
        problem.attendees.len(),
        problem.pillars.len()
    );
    let mut stage_edge_points = vec![];
    const DELTA: f32 = 0.001;

    // NOTE: 10.0 is the musician radius
    let min_x = problem.stage_bottom_left.x() + 10.0;
    let min_y = problem.stage_bottom_left.y() + 10.0;
    let max_x = problem.stage_bottom_left.x() + problem.stage_width - 10.0;
    let max_y = problem.stage_bottom_left.y() + problem.stage_height - 10.0;

    let mut x = min_x;
    while x <= max_x {
        stage_edge_points.push((x, min_y));
        stage_edge_points.push((x, max_y));
        x += DELTA;
    }
    let mut y = min_y;
    while y <= max_y {
        stage_edge_points.push((min_x, y));
        stage_edge_points.push((max_x, y));
        y += DELTA;
    }

    let pruned_attendees = problem
        .attendees
        .par_iter()
        .filter(|a| {
            let is_stage_visible = stage_edge_points.iter().any(|point| {
                let is_point_obstructed = problem
                    .pillars
                    .iter()
                    .any(|p| line_circle_intersection(*a, point, &p.center, p.radius));
                !is_point_obstructed
            });
            is_stage_visible
        })
        .cloned()
        .collect::<Vec<_>>();

    if pruned_attendees.len() < problem.attendees.len() {
        debug!(
            "prune: {} => {} attendees (-{}%)",
            problem.attendees.len(),
            pruned_attendees.len(),
            (((problem.attendees.len() as f32 - pruned_attendees.len() as f32)
                / (problem.attendees.len() as f32))
                * 100.0) as i32
        );
    } else {
        debug!(
            "prune: pruned 0 attendees ({} total)",
            problem.attendees.len()
        );
    }

    let pruned_pillars = problem
        .pillars
        .par_iter()
        .filter(|p| {
            let is_obscuring_vision_of_anyone = pruned_attendees.iter().any(|a| {
                let is_obscuring_vision_of_this_attendee = stage_edge_points
                    .iter()
                    .any(|point| line_circle_intersection(a, point, &p.center, p.radius));
                is_obscuring_vision_of_this_attendee
            });
            is_obscuring_vision_of_anyone
        })
        .cloned()
        .collect::<Vec<_>>();

    if pruned_pillars.len() < problem.attendees.len() {
        debug!(
            "prune: {} => {} pillars (-{}%)",
            problem.pillars.len(),
            pruned_pillars.len(),
            (((problem.pillars.len() as f32 - pruned_pillars.len() as f32)
                / (problem.pillars.len() as f32))
                * 100.0) as i32
        );
    } else {
        debug!("prune: pruned 0 pillars ({} total)", problem.pillars.len());
    }

    (pruned_attendees, pruned_pillars)
}
