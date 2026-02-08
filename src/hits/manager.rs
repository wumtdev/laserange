use std::sync::{
    Arc,
    mpsc::{self, Sender},
};

use chrono::{DateTime, Local};
use image::RgbImage;
use tracing::{error, info};

use crate::{
    bus::Event,
    hits::storage::{HitData, HitStorage},
    targets::TargetInfo,
};
use std::collections::VecDeque;

pub enum HitManagerCommand {
    NewHit {
        timestamp: DateTime<Local>,
        clip: Vec<RgbImage>,
        fps: u32,
        target_info: TargetInfo,
    },
    HitProcessorReady,
}

pub fn start_hit_manager(
    bus_tx: Sender<Event>,
    mut storage: Box<dyn HitStorage>,
) -> Sender<HitManagerCommand> {
    let (tx, rx) = mpsc::channel();

    std::thread::spawn(move || {
        let mut recognizer_ready = true;
        let unprocessed = storage
            .get_unprocessed_hits_old_sorted()
            .expect("failed to get unprocessed hits");
        let mut unprocessed_hits: VecDeque<_> = VecDeque::from(unprocessed);
        for msg in rx {
            match msg {
                HitManagerCommand::NewHit {
                    timestamp,
                    clip,
                    fps,
                    target_info,
                } => {
                    if let Err(e) = storage.new_hit(
                        timestamp,
                        (&clip, fps),
                        HitData {
                            target_info,
                            processed: None,
                        },
                    ) {
                        error!("failed to create clip in storage: {e:?}");
                        continue;
                    };

                    unprocessed_hits.push_back(timestamp);
                }
                HitManagerCommand::HitProcessorReady => {
                    while recognizer_ready {
                        let timestamp = match unprocessed_hits.pop_front() {
                            Some(t) => t,
                            None => break,
                        };
                        let clip = match storage.load_clip(timestamp) {
                            Ok(v) => v,
                            Err(e) => {
                                error!("failed to load unprocessed hit clip: {e:?}");
                                continue;
                            }
                        };

                        let data = match storage.load_data(timestamp) {
                            Ok(v) => v,
                            Err(e) => {
                                error!("failed to load unprocessed hit data: {e:?}");
                                continue;
                            }
                        };

                        bus_tx
                            .send(Event::ProcessHit {
                                timestamp,
                                clip,
                                target_info: data.target_info,
                            })
                            .expect("failed to request hit process");
                        recognizer_ready = false;
                    }
                }
            }
        }
    });

    tx
}
