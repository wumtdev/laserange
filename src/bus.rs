use std::sync::{
    Arc,
    mpsc::{self, Receiver, Sender},
};

use chrono::{DateTime, Local};
use image::{Rgb, RgbImage};
use imageproc::rect::Rect;

use crate::{
    capturer::CapturedFrame,
    hits::{
        detector::{HitDetectorCommand, start_hit_detector},
        manager::HitManagerCommand,
        processor::{HitProcessResult, HitProcessorCommand},
    },
    recorder::Recorder,
    targets::{TargetInfo, recognizer::start_target_recognizer},
    vision::project::unwarp_rectangle,
};

pub enum Event {
    NewFrame(Arc<CapturedFrame>),
    NewStencil((f32, f32, f32, f32)),
    NewHit {
        timestamp: DateTime<Local>,
        clip: (Vec<RgbImage>, u32),
        target_info: TargetInfo,
    },
    HitProcessorReady,
    ProcessHit {
        timestamp: DateTime<Local>,
        clip: (Vec<RgbImage>, u32),
        target_info: TargetInfo,
    },
    ProcessedHit {
        timestamp: DateTime<Local>,
        processed: HitProcessResult,
    },
}

pub enum AppCommand {
    NewFrame(Arc<CapturedFrame>),
    NewStencil((f32, f32, f32, f32)),
}

pub enum AppMessage {
    FrameReady {
        camera_frame: Arc<RgbImage>,
        target_frame: Option<Arc<RgbImage>>,
    },
}

pub fn start() -> (Sender<AppCommand>, Receiver<AppMessage>) {
    let (bus_tx, ui_rx) = mpsc::channel();
    let (ui_tx, bus_rx) = mpsc::channel();
    std::thread::spawn(move || {
        let (bus_tx, bus_rx) = mpsc::channel::<Event>();
        // don't change bus_tx and bus_rx, they are shadowing external
        // bus_tx and bus_rx and it's normal, I want it be so

        let target_info = Arc::new(std::sync::RwLock::new(None));
        let laser_info = Arc::new(std::sync::RwLock::new(None));
        let recorder = Arc::new(Recorder::new());
        let mut _target_stencil = (0f32, 0f32, 1f32, 1f32);

        // Start sub-systems
        let _capturer = crate::capturer::start_capturer(bus_tx.clone());
        let last_camera_frame = Arc::new(std::sync::RwLock::new(None));
        let _target_recognizer =
            start_target_recognizer(target_info.clone(), last_camera_frame.clone());
        let hit_detector = start_hit_detector(
            bus_tx.clone(),
            laser_info.clone(),
            target_info.clone(),
            recorder.clone(),
        );

        let _hit_manager = crate::hits::manager::start_hit_manager(
            bus_tx.clone(),
            Box::new(crate::hits::storage::FileHitStorage::new("data/hits")),
        );

        let _hit_processor = crate::hits::processor::start_hit_processor(bus_tx.clone());

        loop {
            for event in bus_rx.try_iter() {
                match event {
                    Event::NewFrame(captured_frame) => {
                        let mut ui_camera_frame = captured_frame.image.clone();

                        *last_camera_frame.write().unwrap() = Some(captured_frame.clone());

                        let mut ui_target_frame = None;

                        if let Some(target_info) = &*target_info.read().unwrap() {
                            if let Some(mut frame) =
                                unwarp_rectangle(&captured_frame.image, &target_info.rect, 600, 800)
                            {
                                let captured_frame = Arc::new(CapturedFrame {
                                    image: frame.clone(),
                                    timestamp: captured_frame.timestamp,
                                });
                                recorder.push_frame(captured_frame.clone());
                                hit_detector
                                    .send(HitDetectorCommand::NewFrame(captured_frame))
                                    .ok();
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
                                target_info
                                    .rect
                                    .iter()
                                    .map(Into::into)
                                    .collect::<Vec<_>>()
                                    .as_slice(),
                                Rgb([0, 255, 0]),
                            );
                        }

                        // Send frame data back to UI
                        let target_frame_arc = ui_target_frame.map(|f| Arc::new(f));
                        let _ = ui_tx.send(AppMessage::FrameReady {
                            camera_frame: Arc::new(ui_camera_frame),
                            target_frame: target_frame_arc,
                        });
                    }
                    Event::NewStencil(_) => {}
                    Event::HitProcessorReady => {}
                    Event::ProcessHit {
                        timestamp,
                        clip,
                        target_info,
                    } => {
                        _hit_processor
                            .send(HitProcessorCommand::ProcessHit {
                                timestamp,
                                clip,
                                target_info,
                            })
                            .expect("failed to request hit process");
                    }
                    Event::NewHit {
                        timestamp,
                        clip,
                        target_info,
                    } => {
                        _hit_manager
                            .send(HitManagerCommand::NewHit {
                                timestamp,
                                clip,
                                target_info,
                            })
                            .unwrap();
                    }
                    Event::ProcessedHit {
                        timestamp,
                        processed,
                    } => {
                        _hit_manager
                            .send(HitManagerCommand::ProcessedHit {
                                timestamp,
                                processed,
                            })
                            .expect("failed to send hit process result to manager");
                    }
                }
            }
            for cmd in ui_rx.try_iter() {
                match cmd {
                    AppCommand::NewFrame(captured_frame) => {
                        let _ = bus_tx.send(Event::NewFrame(captured_frame));
                    }
                    AppCommand::NewStencil(stencil) => {
                        _target_stencil = stencil;
                        let _ = bus_tx.send(Event::NewStencil(stencil));
                    }
                }
            }
            std::thread::yield_now()
        }
    });
    (bus_tx, bus_rx)
}
