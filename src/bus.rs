use std::sync::Arc;

use crate::capturer::CapturedFrame;

pub enum Event {
    NewFrame(Arc<CapturedFrame>),
}
