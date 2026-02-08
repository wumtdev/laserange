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
                    target_info,
                } => {
                    if let Err(e) = storage.new_hit(
                        timestamp,
                        (&clip.0, clip.1),
                        HitData {
                            target_info: target_info.clone(),
                            processed: None,
                        },
                    ) {
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
                        unprocessed_hits.push_back(timestamp);
                    }
                }
                HitManagerCommand::HitProcessorReady => {
                    recognizer_ready = true;
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
            }
        }
    });

    tx
}
