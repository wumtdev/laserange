use std::sync::mpsc::Sender;

use serde::{Deserialize, Serialize};

use crate::bus::Event;

#[derive(Serialize, Deserialize, Clone)]
pub struct HitProcessResult {}

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
                } => {}
            }
        }
    });

    tx
}
