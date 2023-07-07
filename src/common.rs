use crate::dto::Point2D;

#[derive(Clone, Copy)]
pub struct Position {
    pub p: Point2D,
    pub taken: bool,
}
