use std::ops::{Index, IndexMut};

#[derive(Debug, Clone, Copy, Default)]
pub struct GridSize {
    half_width: usize,
    half_height: usize,
}

impl GridSize {
    pub fn width(&self) -> usize {
        self.half_width * 2
    }

    pub fn height(&self) -> usize {
        self.half_width * 2
    }

    pub fn all_grid_coordinates(&self) -> Vec<GridCoord> {
        let mut res = vec![];
        for y in 0..self.width() {
            for x in 0..self.height() {
                let coord = GridCoord::new(x as isize, y as isize);
                if classify(&coord).is_some() {
                    res.push(coord)
                }
            }
        }
        res
    }
}

#[derive(Debug, Clone, Default)]
pub struct GridTransform {
    pub cell_width: f32,
    pub cell_height: f32,
    pub x_offset: f32,
    pub y_offset: f32,
}

impl GridTransform {
    pub fn apply(&self, coord: &GridCoord) -> (f32, f32) {
        (
            self.x_offset + self.cell_width * coord.x as f32,
            self.y_offset + self.cell_height * coord.y as f32,
        )
    }
}

pub fn fit_circles_grid(
    bottom_left: (f32, f32),
    width: f32,
    height: f32,
    radius: f32,
) -> (GridSize, GridTransform) {
    // the coarseness of the grid is the space between two even rows
    // diag = sqrt(2) * side
    // diag = 2 * radius
    // min_coarseness = 2 * side
    // min_coarseness = 2 * diag / sqrt(2)
    // min_coarseness = 4 * radius / sqrt(2)
    // min_coarseness = radius * (4 / sqrt(2))
    let min_coarseness = radius * 2.82842712474619f32;
    let min_x = bottom_left.0 + radius;
    let min_y = bottom_left.1 + radius;
    let width = width - radius * 2f32;
    let height = height - radius * 2f32;
    fit_grid(min_x, min_y, width, height, min_coarseness)
}

pub fn fit_grid(
    min_x: f32,
    min_y: f32,
    width: f32,
    height: f32,
    min_coarseness: f32,
) -> (GridSize, GridTransform) {
    let size_x = width.div_euclid(min_coarseness);
    let size_y = height.div_euclid(min_coarseness);
    let x_extra = width.rem_euclid(min_coarseness);
    let y_extra = height.rem_euclid(min_coarseness);
    let x_cell_size = min_coarseness + x_extra / size_x;
    let y_cell_size = min_coarseness + y_extra / size_y;
    let grid_size = GridSize {
        half_width: size_x as usize,
        half_height: size_y as usize,
    };
    // the transform
    let grid_transform = GridTransform {
        cell_width: x_cell_size / 2f32,
        cell_height: y_cell_size / 2f32,
        x_offset: min_x,
        y_offset: min_y,
    };
    (grid_size, grid_transform)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GridCoord {
    pub x: isize,
    pub y: isize,
}

impl GridCoord {
    pub fn new(x: isize, y: isize) -> Self {
        GridCoord { x, y }
    }
}

enum CoordSpace {
    Even,
    Odd,
}

fn classify(coord: &GridCoord) -> Option<CoordSpace> {
    match (coord.x & 1, coord.y & 1) {
        (0, 0) => Some(CoordSpace::Even),
        (1, 1) => Some(CoordSpace::Odd),
        _ => None,
    }
}

///    +---------+---------+
///    |(0,2)    |(2,2)    |(4,2)
///    |    +    |    +    |
///    |  (1,1)  |  (3,1)  |
///    +---------+---------+
///     (0,0)     (2,0)     (4,0)
#[derive(Clone, Default)]
pub struct DiamondGrid<T: Clone> {
    even_nodes: Vec<T>,
    odd_nodes: Vec<T>,
    pub size: GridSize,
}

impl<T: Clone> DiamondGrid<T> {
    pub fn new(size: GridSize, init: impl Fn(&GridCoord) -> T) -> DiamondGrid<T> {
        let mut even_nodes = vec![];
        for y in (0..size.half_height).step_by(2) {
            for x in (0..size.half_width).step_by(2) {
                even_nodes.push(init(&GridCoord::new(x as isize, y as isize)));
            }
        }

        let mut odd_nodes = vec![];
        for y in (1..size.half_height).step_by(2) {
            for x in (1..size.half_width).step_by(2) {
                odd_nodes.push(init(&GridCoord::new(x as isize, y as isize)));
            }
        }

        DiamondGrid {
            even_nodes,
            odd_nodes,
            size,
        }
    }
}

impl<T: Copy> Index<&GridCoord> for DiamondGrid<T> {
    type Output = T;

    fn index(&self, index: &GridCoord) -> &Self::Output {
        let x = index.x as usize >> 1;
        let y = index.y as usize >> 1;
        match classify(index).unwrap() {
            CoordSpace::Even => return &self.even_nodes[y * self.size.half_width + x],
            CoordSpace::Odd => return &self.odd_nodes[y * (self.size.half_width - 1) + x],
        }
    }
}

impl<T: Copy> IndexMut<&GridCoord> for DiamondGrid<T> {
    fn index_mut(&mut self, index: &GridCoord) -> &mut Self::Output {
        let x = index.x as usize >> 1;
        let y = index.y as usize >> 1;
        match classify(index).unwrap() {
            CoordSpace::Even => return &mut self.even_nodes[y * self.size.half_width + x],
            CoordSpace::Odd => return &mut self.odd_nodes[y * (self.size.half_width - 1) + x],
        }
    }
}
