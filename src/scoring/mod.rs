use dyn_clone::DynClone;

use crate::{
    dto::{Point2D, ProblemDto},
    solvers::Score,
};

use self::new_scorer::NewScorer;

pub mod approximate;
pub mod impact_map;
pub mod new_scorer;
pub mod scorer;

pub trait Scorer: DynClone + Sync + Send {
    fn score(
        &self,
        problem: &ProblemDto,
        placements: &[Point2D],
        volumes: Option<&Vec<f32>>,
    ) -> Score;
}

impl Default for Box<dyn Scorer> {
    fn default() -> Self {
        Box::<NewScorer>::default()
    }
}

dyn_clone::clone_trait_object!(Scorer);
