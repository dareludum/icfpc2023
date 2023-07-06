use super::{Problem, Solver};

#[derive(Clone)]
pub struct NoOp {}

impl Solver for NoOp {
    fn name(&self) -> &'static str {
        "no_op"
    }

    fn solve_core(&self, _problem: &Problem) -> () {
        ()
    }
}
