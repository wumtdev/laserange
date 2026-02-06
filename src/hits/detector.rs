use std::{
    sync::{
        Arc, RwLock,
        mpsc::{self, Sender},
    },
    time::Instant,
};

use image::RgbImage;
use tracing::info;

use crate::{
    bus::Event, capturer::CapturedFrame, coding::ffmpeg::save_video_simple, hits::LaserInfo,
    recorder::Recorder, vision::laser::find_red_laser,
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
        let mut clip: Vec<Arc<CapturedFrame>> = Vec::with_capacity(20);
        let mut recording = false;
        for msg in rx {
            match msg {
                HitDetectorCommand::NewFrame(frame) => {
                    if let Some(laser_flash) = find_red_laser(&frame.image) {
                        println!("Laser: {:?}", laser_flash);
                        if !recording {
                            *laser_info.write().unwrap() = Some(LaserInfo { pos: laser_flash });
                            clip = recorder.frames();
                            recording = true;
                        } else {
                            clip.push(frame);
                        }
                    } else if recording {
                        if clip.len() > 3 {
                            let clip_path = "data/hello.mp4";
                            let v: Vec<_> = clip.iter().map(|c| c.image.clone()).collect();
                            save_video_simple(&v, "data/hello.mp4", 20)
                                .expect("failed to save clip");
                            info!("Saved clip in {clip_path}");
                        }
                        recording = false;
                    }
                }
            }
        }
    });
    tx
}
