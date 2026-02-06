slint::include_modules!();

use image::{GrayImage, Rgb, buffer::ConvertBuffer, imageops::grayscale};
use imageproc::{contours, edges::canny, rect::Rect};
use slint::Weak;
use std::sync::{Arc, Mutex, RwLock, mpsc};
use tracing::info;

use crate::{
    bus::Event,
    capturer::CapturedFrame,
    hits::detector::{HitDetectorCommand, start_hit_detector},
    recorder::Recorder,
    targets::recognizer::start_target_recognizer,
    vision::project::unwarp_rectangle,
};

mod bus;
mod capturer;
mod coding;
mod hits;
mod recorder;
mod targets;
mod vision;

fn main() {
    tracing_subscriber::fmt().init();

    let ui = MainWindow::new().unwrap();

    let ui_weak = ui.as_weak();

    std::thread::spawn(move || {
        let (tx, rx) = mpsc::channel();
        let target_info = Arc::new(RwLock::new(None));
        let laser_info = Arc::new(RwLock::new(None));
        let recorder = Arc::new(Recorder::new());
        let capturer = crate::capturer::start_capturer(tx.clone());
        let last_camera_frame = Arc::new(RwLock::new(None));
        let target_recognizer =
            start_target_recognizer(target_info.clone(), last_camera_frame.clone());
        let hit_detector = start_hit_detector(tx.clone(), laser_info.clone(), recorder.clone());

        for event in rx.iter() {
            match event {
                Event::NewFrame(captured_frame) => {
                    // info!("Received new frame");

                    *last_camera_frame.write().unwrap() = Some(captured_frame.clone());

                    let mut ui_camera_frame = captured_frame.image.clone();
                    let mut ui_target_frame = None;

                    if let Some(target_info) = &*target_info.read().unwrap() {
                        if let Some(mut frame) =
                            unwarp_rectangle(&ui_camera_frame, &target_info.rect, 600, 800)
                        {
                            let captured_frame = Arc::new(CapturedFrame {
                                image: frame.clone(),
                                timestamp: captured_frame.timestamp,
                            });
                            recorder.push_frame(captured_frame.clone());
                            hit_detector
                                .send(HitDetectorCommand::NewFrame(captured_frame))
                                .unwrap();
                            if let Some(laser_info) = &*laser_info.read().unwrap() {
                                imageproc::drawing::draw_cross_mut(
                                    &mut frame,
                                    Rgb([255, 0, 0]),
                                    laser_info.pos.x as i32,
                                    laser_info.pos.y as i32,
                                );
                                imageproc::drawing::draw_hollow_rect_mut(
                                    &mut frame,
                                    Rect::at(
                                        laser_info.pos.x as i32 - 10,
                                        laser_info.pos.y as i32 - 10,
                                    )
                                    .of_size(20, 20),
                                    Rgb([255, 0, 0]),
                                );
                            }
                            ui_target_frame = Some(frame);
                        }
                        imageproc::drawing::draw_hollow_polygon_mut(
                            &mut ui_camera_frame,
                            &target_info.rect,
                            Rgb([0, 255, 0]),
                        );
                    }

                    let ui = ui_weak.clone();
                    slint::invoke_from_event_loop(move || {
                        // Create image in UI thread
                        let buffer = slint::SharedPixelBuffer::<slint::Rgb8Pixel>::clone_from_slice(
                            ui_camera_frame.as_raw(),
                            ui_camera_frame.width(),
                            ui_camera_frame.height(),
                        );
                        let image = slint::Image::from_rgb8(buffer);
                        let ui = ui.upgrade().unwrap();
                        ui.set_camera_frame(image);
                        if let Some(frame) = ui_target_frame {
                            let buffer =
                                slint::SharedPixelBuffer::<slint::Rgb8Pixel>::clone_from_slice(
                                    frame.as_raw(),
                                    frame.width(),
                                    frame.height(),
                                );
                            let image = slint::Image::from_rgb8(buffer);
                            ui.set_target_frame(image);
                        } else {
                            ui.set_target_frame(slint::Image::default());
                        }
                    })
                    .ok();
                }
            }
        }
    });

    ui.run().unwrap();
}
