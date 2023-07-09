use std::ops::{Index, IndexMut};

use rand::Rng;

#[derive(Debug, Clone, Copy, Default)]
pub struct GridSize {
    pub cells_x: usize,
    pub cellx_y: usize,
}

impl GridSize {
    pub fn width(&self) -> usize {
        self.cells_x * 2 + 1
    }

    pub fn height(&self) -> usize {
        self.cellx_y * 2 + 1
    }

    pub fn all_grid_coordinates(&self) -> Vec<GridCoord> {
        let mut res = vec![];
        for x in 0..self.width() {
            for y in 0..self.height() {
                let coord = GridCoord::new(x as isize, y as isize);
                if classify(&coord).is_some() {
                    assert!(self.check(&coord));
                    res.push(coord)
                }
            }
        }
        res
    }

    pub fn check(&self, point: &GridCoord) -> bool {
        let width = self.width() as isize;
        let height = self.height() as isize;
        point.x >= 0
            && point.y >= 0
            && point.x < width
            && point.y < height
            && classify(point).is_some()
    }

    pub fn displace(&self, point: &GridCoord, displacement: (isize, isize)) -> GridCoord {
        let width = self.width() as isize;
        let height = self.height() as isize;
        let (mut off_x, mut off_y) = displacement;
        if height == 1 {
            off_y = 0;
            off_x &= !1;
        }
        if width == 1 {
            off_x = 0;
            off_y &= !1;
        }
        let mut new_x = point.x + off_x;
        let mut new_y = point.y + off_y;

        loop {
            let mut changed = false;
            if new_x < 0 {
                new_x = -new_x;
                changed = true;
            }

            if new_y < 0 {
                new_y = -new_y;
                changed = true;
            }

            let excess_x = new_x - width + 1;
            if excess_x > 0 {
                new_x -= excess_x * 2;
                changed = true;
            }

            let excess_y = new_y - height + 1;
            if excess_y > 0 {
                new_y -= excess_y * 2;
                changed = true;
            }

            if !changed {
                break;
            }
        }
        let res = GridCoord::new(new_x, new_y);
        assert!(
            self.check(&res),
            "{:?} {:?} w {} h {} class {:?}",
            self,
            &res,
            self.width(),
            self.height(),
            classify(&res)
        );
        res
    }
}

#[derive(Debug, Clone, Default)]
pub struct GridTransform {
    pub size: GridSize,
    pub cell_width: f32,
    pub cell_height: f32,
    pub x_min: f32,
    pub y_min: f32,
    pub x_max: f32,
    pub y_max: f32,
}

impl GridTransform {
    pub fn apply(&self, coord: &GridCoord) -> (f32, f32) {
        assert!(self.size.check(coord));
        let x = self.x_min + self.cell_width * coord.x as f32;
        let y = self.y_min + self.cell_height * coord.y as f32;
        assert!(x <= self.x_max + 0.0001, "{:?} {:?}", self, coord);
        assert!(y <= self.y_max + 0.0001, "{:?} {:?}", self, coord);
        (x, y)
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
    let min_coarseness = radius * 2.828_427_f32;
    let min_x = bottom_left.0 + radius;
    let min_y = bottom_left.1 + radius;
    let width = (width - radius * 2f32).max(0.);
    let height = (height - radius * 2f32).max(0.);
    fit_grid(min_x, min_y, width, height, min_coarseness)
}

pub fn fit_grid(
    x_min: f32,
    y_min: f32,
    width: f32,
    height: f32,
    min_coarseness: f32,
) -> (GridSize, GridTransform) {
    let grid_cells_x = width.div_euclid(min_coarseness);
    let grid_cellx_y = height.div_euclid(min_coarseness);
    // a cell is 1 unit on the grid
    let x_cell_size = if grid_cells_x == 0. {
        0.
    } else {
        width / (grid_cells_x * 2.)
    };
    let y_cell_size = if grid_cellx_y == 0. {
        0.
    } else {
        height / (grid_cellx_y * 2.)
    };
    let grid_size = GridSize {
        cells_x: grid_cells_x as usize,
        cellx_y: grid_cellx_y as usize,
    };
    // the transform
    let grid_transform = GridTransform {
        size: grid_size,
        cell_width: x_cell_size,
        cell_height: y_cell_size,
        x_min,
        y_min,
        x_max: x_min + width,
        y_max: y_min + height,
    };
    (grid_size, grid_transform)
}

#[cfg(test)]
mod tests {

    use super::fit_grid;

    #[test]
    fn test_fit_grid() {
        let (_size, _transform) = fit_grid(5., 10., 3.5, 3.5, 1.);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GridCoord {
    pub x: isize,
    pub y: isize,
}

impl GridCoord {
    /// Not public on purpose, to ensure all coordinates are generated in this file.
    /// Not all coordinates are valid.
    fn new(x: isize, y: isize) -> Self {
        GridCoord { x, y }
    }

    pub fn random_displacement(&self, rng: &mut impl Rng, nudge_size: usize) -> (isize, isize) {
        let x_nudge = rng.gen_range(1..=nudge_size);
        let mut y_nudge = rng.gen_range(1..=nudge_size);
        y_nudge &= !1usize;
        y_nudge |= x_nudge & 1;
        let x_nudge = x_nudge as isize;
        let y_nudge = y_nudge as isize;

        let signs: u32 = rng.gen_range(0..4);
        let x_nudge = if signs & 1 == 0 { x_nudge } else { -x_nudge };
        let y_nudge = if signs & 2 == 0 { y_nudge } else { -y_nudge };
        (x_nudge, y_nudge)
    }
}

#[derive(Debug)]
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
        for y in (0..size.height()).step_by(2) {
            for x in (0..size.width()).step_by(2) {
                even_nodes.push(init(&GridCoord::new(x as isize, y as isize)));
            }
        }

        let mut odd_nodes = vec![];
        for y in (1..size.height()).step_by(2) {
            for x in (1..size.width()).step_by(2) {
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
            CoordSpace::Even => &self.even_nodes[y * self.size.cells_x + x],
            CoordSpace::Odd => &self.odd_nodes[y * (self.size.cells_x - 1) + x],
        }
    }
}

impl<T: Copy> IndexMut<&GridCoord> for DiamondGrid<T> {
    fn index_mut(&mut self, index: &GridCoord) -> &mut Self::Output {
        let x = index.x as usize >> 1;
        let y = index.y as usize >> 1;
        match classify(index).unwrap() {
            CoordSpace::Even => &mut self.even_nodes[y * self.size.cells_x + x],
            CoordSpace::Odd => &mut self.odd_nodes[y * (self.size.cells_x - 1) + x],
        }
    }
}
