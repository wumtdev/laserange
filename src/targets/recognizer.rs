use std::{
    ptr::addr_eq,
    sync::{
        Arc, RwLock,
        mpsc::{self, Sender},
    },
    time::{Duration, Instant},
};

use image::buffer::ConvertBuffer;
use imageproc::edges::canny;
use tracing::info;

use crate::{
    capturer::CapturedFrame, recorder::Recorder, targets::TargetInfo,
    vision::frame::find_rectangle_vertices,
};

pub enum TargetRecognizerCommand {}

pub fn start_target_recognizer(
    recorder: Arc<Recorder>,
    target_info_share: Arc<RwLock<Option<TargetInfo>>>,
) -> Sender<TargetRecognizerCommand> {
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        let mut last_recognition_at = Instant::now();
        let recognition_interval = Duration::from_secs(1);
        let mut processed_frame: Option<Arc<CapturedFrame>> = None;
        loop {
            while last_recognition_at.elapsed() < recognition_interval {
                if let Ok(msg) =
                    rx.recv_timeout(recognition_interval - last_recognition_at.elapsed())
                {
                    info!("Command received");
                }
            }

            let frame = match recorder.last_frame() {
                Some(v) => v,
                None => continue,
            };

            if let Some(processed_frame) = &mut processed_frame
                && addr_eq(&*processed_frame, &*frame)
            {
                continue;
            }
            processed_frame = Some(frame.clone());

            // if let Some(laser_flash) = find_red_laser(&frame) {
            //     println!("Laser: {:?}", laser_flash);
            //     println!(
            //         "Color: {:?}",
            //         frame.get_pixel(laser_flash.x as u32, laser_flash.y as u32)
            //     );
            //     imageproc::drawing::draw_cross_mut(
            //         &mut frame,
            //         Rgb([0, 255, 0]),
            //         laser_flash.x as i32,
            //         laser_flash.y as i32,
            //     );
            //     imageproc::drawing::draw_hollow_rect_mut(
            //         &mut frame,
            //         Rect::at(laser_flash.x as i32 - 10, laser_flash.y as i32 - 10).of_size(20, 20),
            //         Rgb([0, 255, 0]),
            //     );
            // }

            let gray = frame.image.convert();
            let edges = canny(&gray, 50.0, 100.0);
            let contours = imageproc::contours::find_contours::<u32>(&edges);

            if let Some(rect) = find_rectangle_vertices(&contours) {
                *target_info_share
                    .write()
                    .expect("failed to lock target info share") = Some(TargetInfo { rect });
            }

            last_recognition_at = Instant::now();
        }
    });
    tx
}
