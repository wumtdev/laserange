use imageproc::point::Point;

pub mod detector;
pub mod manager;
pub mod processor;
pub mod storage;

pub struct LaserInfo {
    pub pos: Point<f32>,
}
