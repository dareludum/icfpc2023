use serde::{Deserialize, Serialize};

// TODO: Add problem DTOs here here

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct SolvedSolutionDto {
    pub solver_name: String,
    pub score: u64,
}

impl SolvedSolutionDto {
    pub fn not_solved() -> Self {
        SolvedSolutionDto {
            solver_name: "err_not_solved".to_string(),
            score: 0,
        }
    }
}
