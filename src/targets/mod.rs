use std::sync::Arc;

use image::GrayImage;
use imageproc::point::Point;
use serde::{Deserialize, Serialize};

use crate::util::point::MyPoint;

pub mod recognizer;
pub mod settings;

#[derive(Serialize, Deserialize, Clone)]
pub struct TargetInfo {
    pub rect: [MyPoint<f32>; 4],
}
