use core::time;
use std::sync::{
    Arc,
    mpsc::{self, Sender},
};

use chrono::{DateTime, Local};
use image::RgbImage;
use tracing::{error, info};

use crate::{
    bus::Event,
    hits::{
        processor::HitProcessResult,
        storage::{HitData, HitStorage},
    },
    targets::TargetInfo,
};
use std::collections::VecDeque;

pub enum HitManagerCommand {
    NewHit {
        timestamp: DateTime<Local>,
        clip: (Vec<RgbImage>, u32),
        target_info: TargetInfo,
    },
    HitProcessorReady,
    ProcessedHit {
        timestamp: DateTime<Local>,
        processed: HitProcessResult,
    },
    RequestHitClip {
        timestamp: DateTime<Local>,
    },
}

pub fn start_hit_manager(
    bus_tx: Sender<Event>,
    mut storage: Box<dyn HitStorage>,
) -> Sender<HitManagerCommand> {
    let (tx, rx) = mpsc::channel();

    std::thread::spawn(move || {
        let mut recognizer_ready = true;
        let unprocessed: VecDeque<(DateTime<Local>, HitData)> = {
            let hits = storage
                .get_all_hits()
                .expect("failed to load hits from storage");
            bus_tx
                .send(Event::LoadedHits { hits: hits.clone() })
                .expect("failed to send loaded hits event");
            let mut v: Vec<_> = hits
                .into_iter()
                .filter_map(|(timestamp, hit)| {
                    if hit.processed.is_none() {
                        Some((timestamp, hit))
                    } else {
                        None
                    }
                })
                .collect();
            v.sort_by_key(|v| v.0);
            v.into()
        };
        // let unprocessed = storage
        //     .get_unprocessed_hits_old_sorted()
        //     .expect("failed to get unprocessed hits");
        let mut unprocessed_hits: VecDeque<_> = VecDeque::from(unprocessed);
        for msg in rx {
            match msg {
                HitManagerCommand::NewHit {
                    timestamp,
                    clip,
                    target_info,
                } => {
                    let data = HitData {
                        target_info: target_info.clone(),
                        processed: None,
                    };
                    if let Err(e) = storage.new_hit(timestamp, (&clip.0, clip.1), data.clone()) {
                        error!("failed to create clip in storage: {e:?}");
                        continue;
                    };

                    if recognizer_ready {
                        bus_tx
                            .send(Event::ProcessHit {
                                timestamp,
                                clip,
                                target_info,
                            })
                            .expect("failed to request hit process");
                    } else {
                        unprocessed_hits.push_back((timestamp, data));
                    }
                }
                HitManagerCommand::HitProcessorReady => {
                    recognizer_ready = true;
                    while recognizer_ready {
                        let (timestamp, data) = match unprocessed_hits.pop_front() {
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
                HitManagerCommand::ProcessedHit {
                    timestamp,
                    processed,
                } => {
                    let mut data = match storage.load_data(timestamp) {
                        Ok(v) => v,
                        Err(e) => {
                            error!("failed to load hit {timestamp} from storage: {e:?}");
                            continue;
                        }
                    };
                    data.processed = Some(processed);
                    storage
                        .save_data(timestamp, data)
                        .expect("failed to save hit {timestamp} process result");
                }
                HitManagerCommand::RequestHitClip { timestamp } => {
                    let clip = match storage.load_clip(timestamp) {
                        Ok(v) => v,
                        Err(e) => {
                            error!("failed to load hit {timestamp} clip from storage: {e:?}");
                            continue;
                        }
                    };

                    bus_tx
                        .send(Event::LoadedHitClip { timestamp, clip })
                        .unwrap();
                }
            }
        }
    });

    tx
}
