use std::{
    path::Path,
    sync::{
        Arc, RwLock,
        mpsc::{self, Sender},
    },
};

use chrono::{DateTime, Local};
use tracing::info;

use crate::{
    bus::Event, capturer::CapturedFrame, coding::ffmpeg::save_video, hits::LaserInfo,
    recorder::Recorder, targets::TargetInfo, vision::laser::find_red_laser,
};

pub enum HitDetectorCommand {
    NewFrame(Arc<CapturedFrame>),
}

pub fn start_hit_detector(
    bus: Sender<Event>,
    laser_info: Arc<RwLock<Option<LaserInfo>>>,
    target_info: Arc<RwLock<Option<TargetInfo>>>,
    recorder: Arc<Recorder>,
) -> Sender<HitDetectorCommand> {
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        let mut clip: Vec<Arc<CapturedFrame>> = Vec::with_capacity(60);
        let mut recording = false;
        let mut recording_target_info = None;
        let mut last_laser_at = Local::now();
        for msg in rx {
            match msg {
                HitDetectorCommand::NewFrame(frame) => {
                    if let Some(laser_flash) = find_red_laser(&frame.image) {
                        info!("Laser: {:?}", laser_flash);
                        if !recording {
                            *laser_info.write().unwrap() = Some(LaserInfo { pos: laser_flash });
                            clip = recorder.frames();
                            clip.retain(|f| f.timestamp > last_laser_at);
                            recording = true;
                            recording_target_info =
                                Some(target_info.read().unwrap().clone().unwrap());
                        } else {
                            clip.push(frame);
                        }
                    } else if recording {
                        // let clip_path = "data/hello.mp4";
                        let v: Vec<_> = clip.iter().map(|c| c.image.clone()).collect();
                        // save_video(&v, 20, Path::new("data/hello.mp4"))
                        //     .expect("failed to save clip");
                        bus.send(Event::NewHit {
                            timestamp: frame.timestamp.into(),
                            clip: (v, 20),
                            target_info: recording_target_info.take().unwrap(),
                        })
                        .expect("Failed to send hit event");
                        // info!("Saved clip in {clip_path}");
                        clip.clear();
                        recording = false;
                        last_laser_at = Local::now();
                    }
                }
            }
        }
    });
    tx
}
