use crate::dto::Placement;

#[derive(Clone, Copy)]
pub struct Position {
    pub p: Placement,
    pub taken: bool,
}
