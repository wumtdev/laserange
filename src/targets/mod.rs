use imageproc::point::Point;
use serde::{Deserialize, Serialize};

use crate::util::point::MyPoint;

pub mod recognizer;

#[derive(Serialize, Deserialize)]
pub struct TargetInfo {
    pub rect: [MyPoint<f32>; 4],
}
