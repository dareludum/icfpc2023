use serde::{Deserialize, Serialize};


#[derive(Serialize, Deserialize, Debug)]
pub struct Attendee {
    pub x: f32,
    pub y: f32,
    pub tastes: Vec<f32>,
}


#[derive(Serialize, Deserialize, Debug)]
pub struct ProblemDto {
    pub room_width: f32,
    pub room_height: f32,
    pub stage_width: f32,
    pub stage_height: f32,
    pub stage_bottom_left: (f32, f32),
    pub musicians: Vec<i32>,
    pub attendees: Vec<Attendee>,
}


#[derive(Serialize, Deserialize, Debug)]
pub struct SolutionDto {
    pub placements: Vec<Placement>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Placement {
    pub x: f32,
    pub y: f32,
}


#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct SolutionMetaDto {
    pub solver_name: String,
    pub score: f64,
}

impl SolutionMetaDto {
    pub fn not_solved() -> Self {
        SolutionMetaDto {
            solver_name: "err_not_solved".to_string(),
            score: 0.0,
        }
    }
}
