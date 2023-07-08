use std::cmp::{max, min};

use crate::kdtree::{Point2, Vector2, AABB};

/// A 3D ray
pub struct Ray {
    /// The origin of the ray
    origin: Vector2,
    /// The inverse of the direction of the ray (1 / direction)
    inv_direction: Vector2,
    /// The sign of the direction of the ray (0 if negative, 1 if positive)
    sign: [bool; 2],
}

impl Ray {
    pub fn new(origin: &Vector2, direction: &Vector2) -> Self {
        let inv_direction = Vector2::new(1. / direction.x, 1. / direction.y);
        let sign = [direction.x < 0., direction.y < 0.];

        Self {
            origin: *origin,
            inv_direction,
            sign,
        }
    }

    fn get_aabb_sign(aabb: &AABB, sign: bool) -> Point2 {
        if sign {
            aabb.max
        } else {
            aabb.min
        }
    }

    pub fn intersect(&self, b: &AABB) -> bool {
        let mut t1 = (b.min[0] - self.origin[0]) * self.inv_direction[0];
        let mut t2 = (b.max[0] - self.origin[0]) * self.inv_direction[0];

        let mut tmin = t1.min(t2);
        let mut tmax = t1.max(t2);

        t1 = (b.min[1] - self.origin[1]) * self.inv_direction[1];
        t2 = (b.max[1] - self.origin[1]) * self.inv_direction[1];

        tmin = tmin.max(t1.min(t2).min(tmax));
        tmax = tmax.min(t1.max(t2).max(tmin));

        tmax > tmin.max(0.0)
    }
}
