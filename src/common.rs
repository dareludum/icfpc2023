use log::debug;

use crate::dto::{Point2D, ProblemDto};

#[derive(Clone, Copy)]
pub struct Position {
    pub p: Point2D,
    pub taken: bool,
}

// Rasterized stage to work with a grid of points instead of an infinite surface
#[derive(Default, Clone)]
pub struct Grid {
    pub positions: Vec<Position>,
}

impl Grid {
    pub fn new(problem: &ProblemDto) -> Self {
        let x = problem.stage_bottom_left.0 + 10.0;
        let y = problem.stage_bottom_left.1 + 10.0;
        let until_x = problem.stage_bottom_left.0 + problem.stage_width - 10.0;
        let until_y = problem.stage_bottom_left.1 + problem.stage_height - 10.0;

        debug!("common: {} total musicians", problem.musicians.len());
        let max_instrument = problem.musicians.iter().map(|i| i.0).max().unwrap();
        debug!("common: {} total instruments", max_instrument);

        let weight_factor = problem.musicians.len() as f32 / max_instrument as f32;
        let max_position_count = 1000000.0 / max_instrument as f32 / weight_factor;
        let min_position_count = problem.musicians.len() as f32 * 4.0;
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

        debug!("common: delta = {}", delta);

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

        debug!("common: {} total positions", positions.len());

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
