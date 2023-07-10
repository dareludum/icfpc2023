use std::{fs::File, hash::Hash, hash::Hasher, io::BufReader, path::Path};

use nalgebra::Vector2;
use serde::{Deserialize, Serialize};

use crate::geometry::Coords2D;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Attendee {
    pub x: f32,
    pub y: f32,
    pub tastes: Vec<f32>,
}

impl PartialEq for Attendee {
    fn eq(&self, other: &Self) -> bool {
        self.x == other.x && self.y == other.y
    }
}

impl Eq for Attendee {
    fn assert_receiver_is_total_eq(&self) {}
}

impl Hash for Attendee {
    fn hash<H: Hasher>(&self, state: &mut H) {
        (self.x as i32).hash(state);
        (self.y as i32).hash(state);
    }
}

impl Coords2D for Attendee {
    fn x(&self) -> f32 {
        self.x
    }

    fn y(&self) -> f32 {
        self.y
    }
}

#[derive(
    Serialize, Deserialize, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord, Clone, Copy,
)]
pub struct Instrument(pub u32);

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct PillarDto {
    pub center: (f32, f32),
    pub radius: f32,
}

impl Eq for PillarDto {}

impl Hash for PillarDto {
    fn hash<H: Hasher>(&self, state: &mut H) {
        (self.center.0 as i32).hash(state);
        (self.center.1 as i32).hash(state);
    }
}

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
    #[serde(default)]
    pub pillars: Vec<PillarDto>,
}

#[derive(Serialize, Deserialize, Default, Debug, Clone)]
pub struct SolutionDto {
    pub placements: Vec<Point2D>,
    pub volumes: Option<Vec<f32>>,
}

impl SolutionDto {
    pub fn load(path: &Path) -> std::io::Result<Self> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        Ok(serde_json::from_reader(reader)?)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Default)]
pub struct Point2D {
    pub x: f32,
    pub y: f32,
}

impl Coords2D for Point2D {
    fn x(&self) -> f32 {
        self.x
    }

    fn y(&self) -> f32 {
        self.y
    }
}

impl Eq for Point2D {}

impl Hash for Point2D {
    fn hash<H: Hasher>(&self, state: &mut H) {
        (self.x as i32, self.y as i32).hash(state);
    }
}

impl Point2D {
    pub fn as_vec(&self) -> Vector2<f32> {
        self.into()
    }

    pub fn distance(&self, other: &Point2D) -> f32 {
        self.as_vec().metric_distance(&other.as_vec())
    }
}

impl PillarDto {
    pub fn as_vec(&self) -> Vector2<f32> {
        Vector2::new(self.center.0, self.center.1)
    }
}

impl Attendee {
    pub fn as_vec(&self) -> Vector2<f32> {
        Vector2::new(self.x, self.y)
    }
}

impl From<&Point2D> for Vector2<f32> {
    fn from(val: &Point2D) -> Self {
        Vector2::new(val.x, val.y)
    }
}

impl From<&Attendee> for Vector2<f32> {
    fn from(val: &Attendee) -> Self {
        Vector2::new(val.x, val.y)
    }
}

impl From<Vector2<f32>> for Point2D {
    fn from(value: Vector2<f32>) -> Self {
        Point2D {
            x: value.x,
            y: value.y,
        }
    }
}

impl From<(f32, f32)> for Point2D {
    fn from(value: (f32, f32)) -> Self {
        Point2D {
            x: value.0,
            y: value.1,
        }
    }
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
