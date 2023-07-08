use log::debug;
use priority_queue::PriorityQueue;
use rand::seq::SliceRandom;

use crate::{
    common::Grid,
    dto::{Point2D, SolutionDto},
    geometry::distance2,
};

use super::{Problem, Score, Solver};

#[derive(Default, Clone)]
pub struct Expand {
    problem: Problem,
    grid: Grid,
    placements: Vec<Point2D>,
    pq: PriorityQueue<usize, i64>,
    curr_score: Score,
}

impl Solver for Expand {
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

        // let mut positions_by_distance_from_all = self
        //     .grid
        //     .positions
        //     .iter()
        //     .enumerate()
        //     .map(|(idx, pos)| {
        //         let sum_dist2: f32 = self
        //             .problem
        //             .attendees
        //             .iter()
        //             .map(|a| distance2(pos, a))
        //             .sum();
        //         (idx, sum_dist2)
        //     })
        //     .collect::<Vec<_>>();
        // // Sort by max to min distance (squared)
        // positions_by_distance_from_all.sort_by_key(|(_idx, sum_dist2)| -sum_dist2 as i32);

        // let mut idx_pos = 0;
        // for _ in &self.problem.musicians {
        //     let new_pos = loop {
        //         let pos = &mut self.grid.positions[positions_by_distance_from_all[idx_pos].0];
        //         if !pos.taken {
        //             break pos;
        //         }
        //         idx_pos += 1;
        //     };

        //     self.placements.push(new_pos.p);
        //     new_pos.taken = true;
        //     let new_pos = *new_pos;

        //     for pos in self.grid.positions.iter_mut() {
        //         let dist2 = distance2(pos, &new_pos);
        //         if dist2 <= 100.0 {
        //             pos.taken = true;
        //         }
        //     }
        // }

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

        let mut pq = PriorityQueue::new();
        for (idx, _) in self.problem.data.musicians.iter().enumerate() {
            pq.push(idx, 0);
        }

        self.pq = pq;
        self.curr_score = crate::scorer::score(&self.problem.data, &self.placements);

        debug!("expand({}): initialized", self.problem.id);
    }

    fn solve_step(&mut self) -> (SolutionDto, bool) {
        // loop {
        //     let mut group_size = rand::random::<usize>() % (self.problem.musicians.len() / 2);
        //     if group_size == 0 {
        //         group_size = 1;
        //     }
        //     debug!("expand: group size = {}", group_size);

        //     let mut group_0 = vec![];
        //     for _ in 0..group_size {
        //         group_0.push(self.pq.pop().unwrap());
        //     }
        //     let mut group_1 = vec![];
        //     for _ in 0..group_size {
        //         group_1.push(self.pq.pop().unwrap());
        //     }

        //     let rng = &mut rand::thread_rng();
        //     group_0.shuffle(rng);
        //     // group_1.shuffle(rng);

        //     for (idx0, idx1) in group_0.iter().zip(group_1.iter()) {
        //         self.placements.swap(idx0.0, idx1.0);
        //     }

        //     let new_score = crate::scorer::score(&self.problem, &self.placements);
        //     let diff = new_score.0 - self.curr_score.0;

        //     let individual_contribution = diff / ((group_size / 5) as i64 + 1);
        //     if diff < 0 {
        //         for (idx0, idx1) in group_0.iter().zip(group_1.iter()) {
        //             self.placements.swap(idx0.0, idx1.0);
        //         }
        //         for (idx, priority) in group_0.into_iter().chain(group_1.into_iter()) {
        //             self.pq.push(idx, priority);
        //         }
        //     } else {
        //         for (idx, priority) in group_0.into_iter().chain(group_1.into_iter()) {
        //             // Negative is better
        //             self.pq.push(idx, priority - individual_contribution);
        //         }
        //         self.curr_score = new_score;
        //         if diff > 0 {
        //             break;
        //         }
        //     }
        // }

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
