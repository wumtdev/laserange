slint::include_modules!();

use chrono::Local;
use slint::{ComponentHandle, Model, ModelExt};
use tracing::info;

use crate::bus::AppMessage;

const TIMESTAMP_UI_FORMAT: &'static str = "%Y-%m-%d_%H-%M-%S%.3f";

mod bus;
mod capturer;
mod coding;
mod hits;
mod recorder;
mod targets;
mod util;
mod vision;

fn main() {
    tracing_subscriber::fmt().init();

    let ui = MainWindow::new().unwrap();
    let ui_weak = ui.as_weak();

    // Start app event loop
    let (bus_tx, bus_rx) = crate::bus::start();

    // UI event handling thread
    std::thread::spawn(move || {
        for msg in bus_rx.iter() {
            match msg {
                AppMessage::FrameReady {
                    camera_frame,
                    target_frame,
                } => {
                    let ui = ui_weak.clone();
                    slint::invoke_from_event_loop(move || {
                        // Create camera image and set it
                        let buffer = slint::SharedPixelBuffer::<slint::Rgb8Pixel>::clone_from_slice(
                            camera_frame.as_raw(),
                            camera_frame.width(),
                            camera_frame.height(),
                        );
                        let image = slint::Image::from_rgb8(buffer);
                        let ui = ui.upgrade().unwrap();
                        ui.set_camera_frame(image);

                        // Set target frame if available
                        if let Some(target_img) = &target_frame {
                            let buffer =
                                slint::SharedPixelBuffer::<slint::Rgb8Pixel>::clone_from_slice(
                                    target_img.as_raw(),
                                    target_img.width(),
                                    target_img.height(),
                                );
                            let image = slint::Image::from_rgb8(buffer);
                            ui.set_target_frame(image);
                        } else {
                            ui.set_target_frame(slint::Image::default());
                        }
                    })
                    .ok();
                }
                AppMessage::LoadedHits { hits } => {
                    let ui = ui_weak.clone();
                    slint::invoke_from_event_loop(move || {
                        let ui = ui.upgrade().unwrap();
                        let hits: Vec<HitInfo> = hits
                            .into_iter()
                            .map(|(timestamp, data)| {
                                HitInfo {
                                    timestamp: timestamp
                                        .format(TIMESTAMP_UI_FORMAT)
                                        .to_string()
                                        .into(),
                                    // target_info: data.target_info,
                                    is_processed: data.processed.is_some(),
                                    ..Default::default()
                                }
                            })
                            .collect();
                        ui.global::<HitManagerState>()
                            .set_hits(hits.as_slice().into());
                    })
                    .ok();
                }
                AppMessage::LoadedHitClip { timestamp, clip } => {
                    let ui = ui_weak.clone();
                    slint::invoke_from_event_loop(move || {
                        let ui = ui.upgrade().unwrap();
                        let selected_timestamp =
                            ui.global::<HitManagerState>().get_selected_hit().timestamp;
                        let current_timestamp = timestamp.format(TIMESTAMP_UI_FORMAT).to_string();

                        // Only display if this is for the currently selected hit
                        if selected_timestamp == current_timestamp {
                            let (frames, _fps) = clip;
                            if !frames.is_empty() {
                                // Display first frame of the clip
                                let frame = &frames[0];
                                let buffer =
                                    slint::SharedPixelBuffer::<slint::Rgb8Pixel>::clone_from_slice(
                                        frame.as_raw(),
                                        frame.width(),
                                        frame.height(),
                                    );
                                let image = slint::Image::from_rgb8(buffer);
                                ui.set_target_frame(image);
                            }
                        }
                    })
                    .ok();
                }
                AppMessage::NewHit {
                    timestamp,
                    clip,
                    target_info,
                } => {
                    let ui = ui_weak.clone();
                    slint::invoke_from_event_loop(move || {
                        let ui = ui.upgrade().unwrap();

                        // Create HitInfo for the new hit
                        let new_hit = HitInfo {
                            timestamp: timestamp.format(TIMESTAMP_UI_FORMAT).to_string().into(),
                            is_processed: false,
                            ..Default::default()
                        };

                        // Get current hits and add the new one
                        let current_hits = ui.global::<HitManagerState>().get_hits();
                        let mut hits = Vec::new();

                        for i in 0..current_hits.row_count() {
                            if let Some(hit) = current_hits.row_data(i) {
                                hits.push(hit);
                            }
                        }
                        hits.push(new_hit);

                        // Update UI with the new hits list
                        ui.global::<HitManagerState>()
                            .set_hits(hits.as_slice().into());
                    })
                    .ok();
                }
            }
        }
    });

    // UI stencil change handler
    {
        let bus_tx = bus_tx.clone();
        ui.global::<TargetStencil>()
            .on_change(move |start_x, start_y, end_x, end_y| {
                info!("changed {start_x}, {start_y}, {end_x}, {end_y}");
                bus_tx
                    .send(crate::bus::AppCommand::NewStencil((
                        start_x, start_y, end_x, end_y,
                    )))
                    .unwrap();
            });
    }

    ui.global::<HitManagerState>()
        .on_request_hit_clip(move |timestamp| {
            bus_tx
                .send(crate::bus::AppCommand::RequestHitClip {
                    timestamp: chrono::NaiveDateTime::parse_from_str(
                        &timestamp,
                        TIMESTAMP_UI_FORMAT,
                    )
                    .expect("failed to parse timestamp")
                    .and_local_timezone(Local)
                    .single()
                    .expect("failed to convert to local timezone"),
                })
                .unwrap();
        });

    ui.run().unwrap();
}
