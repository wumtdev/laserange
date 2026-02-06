use std::sync::Arc;

use crate::capturer::CapturedFrame;

pub enum Event {
    NewFrame(Arc<CapturedFrame>),
    NewStencil((f32, f32, f32, f32)),
}
