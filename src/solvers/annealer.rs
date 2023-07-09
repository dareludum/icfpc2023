use log::debug;
use rand::seq::SliceRandom;

use crate::{
    common::Grid,
    dto::{Point2D, SolutionDto},
    geometry::distance2,
};

use super::{Problem, Score, Solver};

#[derive(Default, Clone)]
pub struct Annealer {
    problem: Problem,
    grid: Grid,
    placements: Vec<Point2D>,
    curr_score: Score,
    pub temperature: f32,
    pub cooling_rate: f32,
}

impl Solver for Annealer {
    fn name(&self) -> String {
        "expand".to_owned()
    }

    fn get_grid(&self) -> Option<&Grid> {
        Some(&self.grid)
    }

    fn initialize(&mut self, problem: &Problem, solution: SolutionDto) {
        // NOTE: This can be changed
        assert!(
            solution.placements.is_empty(),
            "expand: must be the start of the chain"
        );
        self.problem = problem.clone();
        self.grid = Grid::new(&self.problem);

        let stride = self.grid.positions.len() / self.problem.data.musicians.len();
        for (idx, _) in (0..self.grid.positions.len())
            .step_by(stride)
            .zip(0..self.problem.data.musicians.len())
        {
            self.placements.push(self.grid.positions[idx].p);
        }

        for placement in &self.placements {
            for pos in self.grid.positions.iter_mut() {
                let dist2 = distance2(pos, placement);
                if dist2 <= 100.0 {
                    pos.taken = true;
                }
            }
        }

        self.curr_score = crate::scorer::score(&self.problem.data, &self.placements);
        debug!("expand({}): initialized", self.problem.id);
    }

    fn solve_step(&mut self) -> (SolutionDto, bool) {
        loop {
            if rand::random::<u8>() % 10 > 3 {
                // Try expand - move musicians to new positions

                let group_size = rand::random::<usize>() % 3 + 1;

                let mut placement_indices = (0..self.placements.len()).collect::<Vec<_>>();
                let (placement_indices_slice, _) =
                    placement_indices.partial_shuffle(&mut rand::thread_rng(), group_size);

                // Take out the musicians
                let old_placements = self.placements.clone();
                for idx in placement_indices_slice.iter() {
                    self.placements[*idx] = Point2D {
                        x: f32::NAN,
                        y: f32::NAN,
                    };
                }

                self.grid.recalculate_taken(&self.placements);

                let mut not_taken_positions = self
                    .grid
                    .positions
                    .iter()
                    .filter(|p| !p.taken)
                    .collect::<Vec<_>>();
                let (not_taken_slice, _) =
                    not_taken_positions.partial_shuffle(&mut rand::thread_rng(), group_size);

                let mut new_placements = old_placements.clone();
                for (idx, pos) in placement_indices_slice.iter().zip(not_taken_slice.iter()) {
                    new_placements[*idx] = pos.p;
                }

                let new_score = crate::scorer::score(&self.problem.data, &new_placements);
                let diff = new_score.0 - self.curr_score.0;

                if diff > 0 {
                    self.placements = new_placements;
                    self.curr_score = new_score;

                    self.grid.recalculate_taken(&self.placements);

                    debug!(
                        "expand({}) : won (group size {})",
                        self.problem.id, group_size
                    );
                    break;
                } else {
                    self.placements = old_placements;

                    self.grid.recalculate_taken(&self.placements);
                }
            } else {
                // Try shuffle - swap musician positions

                let group_size = rand::random::<usize>() % 3 + 1;

                let mut placement_indices = (0..self.placements.len()).collect::<Vec<_>>();
                let (placement_indices_slice, _) =
                    placement_indices.partial_shuffle(&mut rand::thread_rng(), group_size * 2);

                let (group_0, group_1) = placement_indices_slice.split_at_mut(group_size);

                let rng = &mut rand::thread_rng();
                group_0.shuffle(rng);

                let mut new_placements = self.placements.clone();
                for (idx0, idx1) in group_0.iter().zip(group_1.iter()) {
                    new_placements.swap(*idx0, *idx1);
                }

                let new_score = crate::scorer::score(&self.problem.data, &new_placements);
                let diff = new_score.0 - self.curr_score.0;

                if diff > 0 {
                    self.placements = new_placements;
                    self.curr_score = new_score;

                    debug!(
                        "shuffle({}): won (group size {})",
                        self.problem.id, group_size
                    );
                    break;
                }
            }
        }

        (
            SolutionDto {
                placements: self.placements.clone(),
            },
            false,
        )
    }
}
