slint::include_modules!();

use image::{GrayImage, Rgb, buffer::ConvertBuffer, imageops::grayscale};
use imageproc::{contours, edges::canny, rect::Rect};
use slint::Weak;
use std::sync::{Arc, Mutex, RwLock, mpsc};
use tracing::info;

use crate::{
    bus::Event,
    recorder::Recorder,
    targets::{TargetInfo, recognizer::start_target_recognizer},
    vision::{frame::find_rectangle_vertices, laser::find_red_laser, project::unwarp_rectangle},
};

mod bus;
mod capturer;
mod recorder;
mod targets;
mod vision;

fn main() {
    tracing_subscriber::fmt().init();

    let ui = MainWindow::new().unwrap();
    let threshold = Arc::new(Mutex::new(127i32));

    let threshold_clone = threshold.clone();
    ui.on_threshold_changed(move |val| {
        *threshold_clone.lock().unwrap() = val;
    });

    let ui_weak = ui.as_weak();

    std::thread::spawn(move || {
        let (tx, rx) = mpsc::channel();
        let target_info = Arc::new(RwLock::new(None));
        let recorder = Arc::new(Recorder::new());
        let capturer_tx = crate::capturer::start_capturer(tx.clone());
        let target_recognizer = start_target_recognizer(recorder.clone(), target_info.clone());

        for event in rx.iter() {
            match event {
                Event::NewFrame(frame) => {
                    // info!("Received new frame");
                    recorder.push_frame(frame.clone());

                    let mut frame = frame.image.clone();
                    let mut target_frame = None;

                    if let Some(target_info) = &*target_info.read().unwrap() {
                        target_frame = unwarp_rectangle(&frame, &target_info.rect, 600, 800);
                        imageproc::drawing::draw_hollow_polygon_mut(
                            &mut frame,
                            &target_info.rect,
                            Rgb([0, 255, 0]),
                        );
                    }

                    let ui = ui_weak.clone();
                    slint::invoke_from_event_loop(move || {
                        // Create image in UI thread
                        let buffer = slint::SharedPixelBuffer::<slint::Rgb8Pixel>::clone_from_slice(
                            frame.as_raw(),
                            frame.width(),
                            frame.height(),
                        );
                        let image = slint::Image::from_rgb8(buffer);
                        let ui = ui.upgrade().unwrap();
                        ui.set_camera_frame(image);
                        if let Some(frame) = target_frame {
                            let buffer =
                                slint::SharedPixelBuffer::<slint::Rgb8Pixel>::clone_from_slice(
                                    frame.as_raw(),
                                    frame.width(),
                                    frame.height(),
                                );
                            let image = slint::Image::from_rgb8(buffer);
                            ui.set_target_frame(image);
                        }
                    })
                    .ok();
                }
            }
        }
    });

    ui.run().unwrap();
}
