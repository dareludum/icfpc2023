use enum_map::{Enum, EnumMap};

use crate::kdtree::AABB;

/// 2D dimensions enum.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Enum)]
pub enum Dimension {
    X,
    Y,
}

impl Dimension {
    pub fn get_map<T: Clone>(value: T) -> EnumMap<Dimension, T> {
        enum_map! {
            Dimension::X => value.clone(),
            Dimension::Y => value.clone(),
        }
    }
}

/// 3D plane.
#[derive(Clone, Debug)]
pub struct Plane {
    pub dimension: Dimension,
    pub pos: f32,
}

impl Plane {
    /// Create a new plane.
    pub fn new(dimension: Dimension, pos: f32) -> Self {
        Plane { dimension, pos }
    }

    /// Create a new plane on the X axis.
    pub fn new_x(pos: f32) -> Self {
        Plane::new(Dimension::X, pos)
    }

    /// Create a new plane on the Y axis.
    pub fn new_y(pos: f32) -> Self {
        Plane::new(Dimension::Y, pos)
    }

    /// Check if the plane is cutting the given space.
    pub fn is_cutting(&self, space: &AABB) -> bool {
        match self.dimension {
            Dimension::X => self.pos > space.min.x && self.pos < space.max.x,
            Dimension::Y => self.pos > space.min.y && self.pos < space.max.y,
        }
    }
}
