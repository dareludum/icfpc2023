pub trait Coords2D {
    fn x(&self) -> f32;
    fn y(&self) -> f32;
}

pub fn distance2(c0: &impl Coords2D, c1: &impl Coords2D) -> f32 {
    let x = c0.x() - c1.x();
    let y = c0.y() - c1.y();
    x * x + y * y
}
