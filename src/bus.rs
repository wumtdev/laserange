use std::{
    collections::HashMap,
    sync::{
        Arc, RwLock,
        mpsc::{self, Receiver, Sender},
    },
};

use chrono::{DateTime, Local};
use image::{GrayImage, Rgb, RgbImage, buffer::ConvertBuffer};
use imageproc::{
    drawing::draw_hollow_polygon_mut, edges::canny, filter::gaussian_blur_f32, point::Point,
    rect::Rect,
};

use crate::{
    capturer::CapturedFrame,
    hits::{
        detector::{HitDetectorCommand, start_hit_detector},
        manager::HitManagerCommand,
        processor::{HitProcessResult, HitProcessorCommand},
        storage::HitData,
    },
    recorder::Recorder,
    targets::{TargetInfo, recognizer::start_target_recognizer},
    vision::{crop::crop_image, project::unwarp_rectangle, stencil::Stencil, zones::ZoneMap},
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
    LoadedHits {
        hits: HashMap<DateTime<Local>, HitData>,
    },
    LoadedHitClip {
        timestamp: DateTime<Local>,
        clip: (Vec<RgbImage>, u32),
    },
}

pub enum AppCommand {
    NewStencil((f32, f32, f32, f32)),
    RequestHitClip { timestamp: DateTime<Local> },
}

pub enum AppMessage {
    FrameReady {
        camera_frame: Arc<RgbImage>,
        target_frame: Option<Arc<RgbImage>>,
    },
    LoadedHits {
        hits: HashMap<DateTime<Local>, HitData>,
    },
    LoadedHitClip {
        timestamp: DateTime<Local>,
        clip: (Vec<RgbImage>, u32),
    },
    NewHit {
        timestamp: DateTime<Local>,
        clip: (Vec<RgbImage>, u32),
        target_info: TargetInfo,
    },
}

pub fn start() -> (Sender<AppCommand>, Receiver<AppMessage>) {
    let (bus_tx, ui_rx) = mpsc::channel();
    let (ui_tx, bus_rx) = mpsc::channel();
    std::thread::spawn(move || {
        let (bus_tx, bus_rx) = mpsc::channel::<Event>();

        let target_settings = crate::targets::settings::load_targets();

        let target_info = Arc::new(RwLock::new(None));
        let zone_map = Arc::new(RwLock::new(None));
        let zone_scores: Arc<RwLock<Vec<u32>>> = Arc::new(RwLock::new(Vec::new()));
        let laser_info = Arc::new(RwLock::new(None));
        let recorder = Arc::new(Recorder::new());
        let mut target_stencil = Stencil::default();

        // Start sub-systems
        let capturer = crate::capturer::start_capturer(bus_tx.clone());
        let last_camera_frame = Arc::new(RwLock::new(None));
        let target_recognizer = start_target_recognizer(
            target_info.clone(),
            zone_map.clone(),
            last_camera_frame.clone(),
        );
        let hit_detector = start_hit_detector(
            bus_tx.clone(),
            laser_info.clone(),
            target_info.clone(),
            recorder.clone(),
        );

        let hit_manager = crate::hits::manager::start_hit_manager(
            bus_tx.clone(),
            Box::new(crate::hits::storage::FileHitStorage::new("data/hits")),
        );

        let hit_processor = crate::hits::processor::start_hit_processor(bus_tx.clone());

        loop {
            for event in bus_rx.try_iter() {
                match event {
                    Event::NewFrame(captured_frame) => {
                        let mut camera_frame = captured_frame.image.clone();

                        let mut target_frame = target_stencil.crop(&camera_frame).to_image();
                        *last_camera_frame.write().unwrap() = Some(Arc::new(CapturedFrame {
                            image: target_frame.clone(),
                            timestamp: captured_frame.timestamp,
                        }));

                        if let Some(target_info) = &*target_info.read().unwrap() {
                            if let Some(frame) =
                                unwarp_rectangle(&target_frame, &target_info.rect, 600, 800)
                            {
                                target_frame = frame;
                            }
                            let r =
                                target_stencil.rect(camera_frame.width(), camera_frame.height());
                            let a: Vec<Point<f32>> = target_info
                                .rect
                                .iter()
                                .map(|p| Point::new(r.x as f32 + p.x, r.y as f32 + p.y))
                                .collect();
                            draw_hollow_polygon_mut(&mut camera_frame, &a, Rgb([0, 255, 0]));
                        }

                        // *zone_map.write().unwrap() =
                        //     Some(ZoneMap::recognize(&target_frame.convert()));

                        let captured_target_frame = Arc::new(CapturedFrame {
                            image: target_frame.clone(),
                            timestamp: captured_frame.timestamp,
                        });
                        recorder.push_frame(captured_target_frame.clone());
                        // hit_detector
                        //     .send(HitDetectorCommand::NewFrame(captured_target_frame.clone()))
                        //     .unwrap();

                        // if let Some(target_info) = &*target_info.read().unwrap() {
                        //     if let Some(mut frame) =
                        //         unwarp_rectangle(&captured_frame.image, &target_info.rect, 600, 800)
                        //     {
                        //         let captured_frame = Arc::new(CapturedFrame {
                        //             image: frame.clone(),
                        //             timestamp: captured_frame.timestamp,
                        //         });
                        //         recorder.push_frame(captured_frame.clone());
                        //         hit_detector
                        //             .send(HitDetectorCommand::NewFrame(captured_frame))
                        //             .unwrap();
                        //         if let Some(laser_info) = &*laser_info.read().unwrap() {
                        //             imageproc::drawing::draw_cross_mut(
                        //                 &mut frame,
                        //                 Rgb([255, 0, 0]),
                        //                 laser_info.pos.x as i32,
                        //                 laser_info.pos.y as i32,
                        //             );
                        //             imageproc::drawing::draw_hollow_rect_mut(
                        //                 &mut frame,
                        //                 Rect::at(
                        //                     laser_info.pos.x as i32 - 10,
                        //                     laser_info.pos.y as i32 - 10,
                        //                 )
                        //                 .of_size(20, 20),
                        //                 Rgb([255, 0, 0]),
                        //             );
                        //         }
                        //         ui_target_frame = Some(frame);
                        //     }
                        //     imageproc::drawing::draw_hollow_polygon_mut(
                        //         &mut ui_camera_frame,
                        //         target_info
                        //             .rect
                        //             .iter()
                        //             .map(Into::into)
                        //             .collect::<Vec<_>>()
                        //             .as_slice(),
                        //         Rgb([0, 255, 0]),
                        //     );
                        // }

                        // Send frame data back to UI

                        let mut target_frame_arc = Some(
                            {
                                let img: GrayImage = target_frame.convert();
                                let img = gaussian_blur_f32(&img, 1.5).convert();
                                // canny(&img, 10.0, 10.0).convert()
                                img
                            }
                            .clone(),
                        )
                        .map(|f| Arc::new(f));

                        // if let Some(zone_map) = &*zone_map.read().unwrap() {
                        //     target_frame_arc = Some(Arc::new(zone_map.map().convert()));
                        // }

                        ui_tx
                            .send(AppMessage::FrameReady {
                                camera_frame: Arc::new(camera_frame),
                                target_frame: target_frame_arc,
                            })
                            .unwrap();
                    }
                    Event::NewStencil(_) => {}
                    Event::HitProcessorReady => hit_manager
                        .send(HitManagerCommand::HitProcessorReady)
                        .unwrap(),
                    Event::ProcessHit {
                        timestamp,
                        clip,
                        target_info,
                    } => {
                        hit_processor
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
                        hit_manager
                            .send(HitManagerCommand::NewHit {
                                timestamp,
                                clip: clip.clone(),
                                target_info: target_info.clone(),
                            })
                            .unwrap();
                        ui_tx
                            .send(AppMessage::NewHit {
                                timestamp,
                                clip,
                                target_info,
                            })
                            .unwrap();
                    }
                    Event::ProcessedHit {
                        timestamp,
                        processed,
                    } => hit_manager
                        .send(HitManagerCommand::ProcessedHit {
                            timestamp,
                            processed,
                        })
                        .expect("failed to send hit process result to manager"),
                    Event::LoadedHits { hits } => ui_tx
                        .send(AppMessage::LoadedHits { hits })
                        .expect("failed to send loaded hits to ui"),
                    Event::LoadedHitClip { timestamp, clip } => ui_tx
                        .send(AppMessage::LoadedHitClip { timestamp, clip })
                        .unwrap(),
                }
            }

            for cmd in ui_rx.try_iter() {
                match cmd {
                    AppCommand::NewStencil(stencil) => {
                        target_stencil = stencil.into();
                        *target_info.write().unwrap() = None;
                        bus_tx.send(Event::NewStencil(stencil)).unwrap();
                    }
                    AppCommand::RequestHitClip { timestamp } => hit_manager
                        .send(HitManagerCommand::RequestHitClip { timestamp })
                        .unwrap(),
                }
            }

            std::thread::yield_now();
        }
    });
    (bus_tx, bus_rx)
}
