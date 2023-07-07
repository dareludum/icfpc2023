use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Attendee {
    pub x: f32,
    pub y: f32,
    pub tastes: Vec<f32>,
}

#[derive(Serialize, Deserialize, Debug, Default, PartialEq, Eq, Hash, Clone, Copy)]
pub struct Instrument(pub u32);

// Default is to avoid Option<ProblemDto> in solvers
#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct ProblemDto {
    pub room_width: f32,
    pub room_height: f32,
    pub stage_width: f32,
    pub stage_height: f32,
    pub stage_bottom_left: (f32, f32),
    pub musicians: Vec<Instrument>,
    pub attendees: Vec<Attendee>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SolutionDto {
    pub placements: Vec<Placement>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct Placement {
    pub x: f32,
    pub y: f32,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct SolutionMetaDto {
    pub solver_name: String,
    pub score: i64,
}

impl SolutionMetaDto {
    pub fn not_solved() -> Self {
        SolutionMetaDto {
            solver_name: "err_not_solved".to_string(),
            score: 0,
        }
    }
}
