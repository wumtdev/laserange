use imageproc::point::Point;

pub mod recognizer;

pub struct TargetInfo {
    pub rect: [Point<f32>; 4],
}
