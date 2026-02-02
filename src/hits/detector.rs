use std::sync::{
    Arc, RwLock,
    mpsc::{self, Sender},
};

use tracing::info;

use crate::{
    bus::Event, capturer::CapturedFrame, hits::LaserInfo, recorder::Recorder,
    vision::laser::find_red_laser,
};

pub enum HitDetectorCommand {
    NewFrame(Arc<CapturedFrame>),
}

pub fn start_hit_detector(
    bus: Sender<Event>,
    laser_info: Arc<RwLock<Option<LaserInfo>>>,
    recorder: Arc<Recorder>,
) -> Sender<HitDetectorCommand> {
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        let recording: Option<Vec<CapturedFrame>> = None;
        let mut recorded = false;
        for msg in rx {
            match msg {
                HitDetectorCommand::NewFrame(frame) => {
                    if let Some(laser_flash) = find_red_laser(&frame.image) {
                        println!("Laser: {:?}", laser_flash);
                        if !recorded {
                            *laser_info.write().unwrap() = Some(LaserInfo { pos: laser_flash });
                            recorded = true;
                        }
                    } else {
                        recorded = false;
                    }
                }
            }
        }
    });
    tx
}
