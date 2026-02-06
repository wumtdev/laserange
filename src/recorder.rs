use std::{
    collections::VecDeque,
    sync::{
        Arc, Mutex,
        mpsc::{self, Sender},
    },
    thread::yield_now,
    time::Duration,
};

use tracing::{error, info};

use crate::{bus::Event, capturer::CapturedFrame};

const FRAME_EXPIRE: Duration = Duration::from_secs(3);

pub struct Recorder {
    frames: Mutex<VecDeque<Arc<CapturedFrame>>>,
}

impl Recorder {
    pub fn new() -> Self {
        Self {
            frames: Mutex::new(VecDeque::new()),
        }
    }

    pub fn push_frame(&self, frame: Arc<CapturedFrame>) {
        let mut frames = match self.frames.lock() {
            Ok(v) => v,
            Err(e) => {
                error!("Failed to lock frames queue to push frame: {e:?}");
                return;
            }
        };

        frames.retain(|c| c.timestamp.elapsed() > FRAME_EXPIRE);

        frames.push_back(frame);
    }

    pub fn last_frame(&self) -> Option<Arc<CapturedFrame>> {
        let frames = match self.frames.lock() {
            Ok(v) => v,
            Err(e) => {
                error!("Failed to lock frames queue to push frame: {e:?}");
                return None;
            }
        };

        frames.back().cloned()
    }

    pub fn frames(&self) -> Vec<Arc<CapturedFrame>> {
        self.frames.lock().unwrap().iter().cloned().collect()
    }
}
