use std::sync::{
    Arc,
    mpsc::{self, Receiver, Sender},
};

use chrono::{DateTime, Local};
use image::RgbImage;

use crate::{capturer::CapturedFrame, targets::TargetInfo};

pub enum Event {
    NewFrame(Arc<CapturedFrame>),
    NewStencil((f32, f32, f32, f32)),
    HitProcessorReady,
    ProcessHit {
        timestamp: DateTime<Local>,
        clip: (Vec<RgbImage>, u32),
        target_info: TargetInfo,
    },
}

pub enum AppCommand {}
pub enum AppMessage {}

pub fn start() -> (Sender<AppCommand>, Receiver<AppMessage>) {
    let (app_tx, owner_rx) = mpsc::channel();
    let (owner_tx, app_rx) = mpsc::channel();
    todo!();
    (app_tx, app_rx)
}
