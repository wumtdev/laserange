use std::sync::{
    Arc, RwLock,
    mpsc::{self, Sender},
};

use tracing::info;

use crate::{bus::Event, capturer::CapturedFrame, hits::LaserInfo, vision::laser::find_red_laser};

pub enum HitDetectorCommand {
    NewFrame(Arc<CapturedFrame>),
}

pub fn start_hit_detector(
    bus: Sender<Event>,
    laser_info: Arc<RwLock<Option<LaserInfo>>>,
) -> Sender<HitDetectorCommand> {
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        for msg in rx {
            match msg {
                HitDetectorCommand::NewFrame(frame) => {
                    if let Some(laser_flash) = find_red_laser(&frame.image) {
                        println!("Laser: {:?}", laser_flash);
                        *laser_info.write().unwrap() = Some(LaserInfo { pos: laser_flash });
                    }
                }
            }
        }
    });
    tx
}
