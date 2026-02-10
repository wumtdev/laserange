use std::sync::mpsc::Sender;

use serde::{Deserialize, Serialize};
use tracing::info;

use crate::{bus::Event, util::point::MyPoint, vision::laser::find_red_laser};

#[derive(Serialize, Deserialize, Clone)]
pub struct HitProcessResult {
    pub score: f32,
    pub hit_pos: Option<MyPoint<f32>>,
}

pub enum HitProcessorCommand {
    ProcessHit {
        timestamp: chrono::DateTime<chrono::Local>,
        clip: (Vec<image::RgbImage>, u32),
        target_info: crate::targets::TargetInfo,
    },
}

pub fn start_hit_processor(bus_tx: Sender<Event>) -> Sender<HitProcessorCommand> {
    let (tx, rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        bus_tx
            .send(Event::HitProcessorReady)
            .expect("failed to send hit processor ready event");
        for msg in rx {
            match msg {
                HitProcessorCommand::ProcessHit {
                    timestamp,
                    clip,
                    target_info,
                } => {
                    info!("Processing {timestamp:?}");
                    let mut hit_pos = None;
                    for frame in clip.0 {
                        if let Some(pos) = find_red_laser(&frame) {
                            hit_pos = Some(MyPoint::from(pos));
                            break;
                        }
                    }

                    let res = HitProcessResult {
                        score: 0.0,
                        hit_pos: hit_pos,
                    };

                    bus_tx
                        .send(Event::ProcessedHit {
                            timestamp,
                            processed: res,
                        })
                        .unwrap();
                    bus_tx.send(Event::HitProcessorReady).unwrap();
                }
            }
        }
    });

    tx
}
