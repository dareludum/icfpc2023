use crate::kdtree::Point2;

/// Axis-aligned bounding box is defined by two positions.
///
/// **Note**: The first position is expected to be the minimum bound and the second
/// the maximum bound.
///
/// The animated GIF below shows a graphic example of an AABB that adapts its
/// size to fit the rotating entity. The box constantly changes dimensions to
/// snugly fit the entity contained inside.
///
/// ![Gif describing an AABB](https://media.prod.mdn.mozit.cloud/attachments/2015/10/16/11799/57dfaf5508784d6b9c5fe77c0df49a54/rotating_knot.gif)
#[derive(Clone, Debug)]
pub struct AABB {
    /// Minimum position
    pub min: Point2,
    /// Maximum position
    pub max: Point2,
}

impl Default for AABB {
    fn default() -> Self {
        Self::empty()
    }
}

impl AABB {
    /// Create an new AABB from two points.
    pub fn new(min: Point2, max: Point2) -> Self {
        Self { min, max }
    }

    /// Create an empty AABB.
    pub fn empty() -> Self {
        Self::new(
            Point2::new(f32::INFINITY, f32::INFINITY),
            Point2::new(f32::NEG_INFINITY, f32::NEG_INFINITY),
        )
    }

    /// Compute AABB area
    pub fn area(&self) -> f32 {
        (self.max.x - self.min.x) * (self.max.y - self.min.y)
    }

    /// Compute AABB surface
    pub fn surface(&self) -> f32 {
        let dx = self.max.x - self.min.x;
        let dy = self.max.y - self.min.y;
        2.0 * (dx * dy)
    }

    /// Merge another AABB into this one.
    pub fn merge(&mut self, other: &Self) {
        self.min = Point2::new(self.min.x.min(other.min.x), self.min.y.min(other.min.y));
        self.max = Point2::new(self.max.x.max(other.max.x), self.max.y.max(other.max.y));
    }
}

/// Your shapes needs to implement `Bounded` trait to build a KD-tree around it.
pub trait Bounded {
    /// This function return the **Axis-aligned bounding boxes**
    /// (`AABB`) of the object.
    ///
    /// For more information check [AABB](type.AABB.html).
    fn bound(&self) -> AABB;
}
