pub trait Coords2D {
    fn x(&self) -> f32;
    fn y(&self) -> f32;
}

impl Coords2D for (f32, f32) {
    fn x(&self) -> f32 {
        self.0
    }

    fn y(&self) -> f32 {
        self.1
    }
}

impl Coords2D for nalgebra::Vector2<f32> {
    fn x(&self) -> f32 {
        self.x
    }

    fn y(&self) -> f32 {
        self.y
    }
}

pub fn distance2(c0: &impl Coords2D, c1: &impl Coords2D) -> f32 {
    let x = c0.x() - c1.x();
    let y = c0.y() - c1.y();
    x * x + y * y
}

pub fn line_circle_intersection(
    line_start: &impl Coords2D,
    line_end: &impl Coords2D,
    circle_center: &impl Coords2D,
    radius: f32,
) -> bool {
    // Create vector from the start of the line to the center of the circle
    let start_to_center_x = circle_center.x() - line_start.x();
    let start_to_center_y = circle_center.y() - line_start.y();

    // Create the vector that represents the line
    let line_vector_x = line_end.x() - line_start.x();
    let line_vector_y = line_end.y() - line_start.y();

    // Calculate the squared length of the line
    let line_len_sq = line_vector_x * line_vector_x + line_vector_y * line_vector_y;

    // Calculate the dot product of the start_to_center and the line_vector
    let dot_product = start_to_center_x * line_vector_x + start_to_center_y * line_vector_y;

    // Calculate the closest Placement on the line to the center of the circle
    let t = dot_product / line_len_sq;

    // If the closest Placement is outside the line segment, return false
    if !(0.0..=1.0).contains(&t) {
        return false;
    }

    // Calculate the coordinates of the closest Placement
    let closest_point_x = line_start.x() + t * line_vector_x;
    let closest_point_y = line_start.y() + t * line_vector_y;

    // Calculate the vector from the closest Placement to the center of the circle
    let closest_to_center_x = circle_center.x() - closest_point_x;
    let closest_to_center_y = circle_center.y() - closest_point_y;

    // Calculate the squared length of the vector
    let closest_to_center_len_sq =
        closest_to_center_x * closest_to_center_x + closest_to_center_y * closest_to_center_y;

    // If the squared length is less than r squared, the line intersects the circle
    closest_to_center_len_sq <= radius * radius
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cases() {
        let tests = [
            ((0., 0.), (1., 0.), (0.5, 0.), 0.01, true),
            ((0., 0.), (10., 0.), (5., 5.), 3., false),
            ((0., 0.), (10., 0.), (5., 5.), 5.1, true),
            ((0., 0.), (10., 0.), (5., 5.), 4.9, false),
            // circle alongside the line
            ((-2.42, -3.58), (14.76, 6.64), (7.1, 8.44), 5., false),
            ((-2.42, -3.58), (14.76, 6.64), (7.1, 7.44), 5., true),
            // circle slightly behind the line
            ((-2.42, -3.58), (14.76, 6.64), (17.56, 11.7), 5., false),
            ((1100., 800.), (1100., 150.), (1100., 100.), 5., false),
        ];

        for (a, b, c, r, int) in tests {
            let res = line_circle_intersection(&a, &b, &c, r);
            assert_eq!(
                res, int,
                "intersection of segment from {:?} to {:?} by {:?} r {} should be {}",
                a, b, c, r, int
            );
        }
    }
}
