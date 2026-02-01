use std::{
    sync::{
        Arc,
        mpsc::{self, Sender},
    },
    time::Instant,
};

use image::RgbImage;
use nokhwa::{
    Camera,
    pixel_format::RgbFormat,
    utils::{CameraIndex, RequestedFormat, RequestedFormatType},
};
use tracing::info;

use crate::bus::Event;

pub struct CapturedFrame {
    pub timestamp: Instant,
    pub image: RgbImage,
}

#[derive(Debug)]
pub enum CapturerCommand {}

pub fn start_capturer(app_tx: Sender<Event>) -> Sender<CapturerCommand> {
    let (tx, rx) = mpsc::channel();

    std::thread::spawn(move || {
        let requested_format =
            RequestedFormat::new::<RgbFormat>(RequestedFormatType::AbsoluteHighestResolution);
        let mut cam =
            Camera::new(CameraIndex::Index(0), requested_format).expect("failed to open camera");
        cam.open_stream().expect("failed to open stream");

        info!(
            "{}x{} {}fps",
            cam.resolution().height(),
            cam.resolution().width(),
            cam.frame_rate()
        );

        loop {
            for cmd in rx.try_iter() {
                info!("Received command: {:?}", cmd);
            }

            let frame = cam.frame().expect("failed to get next frame");

            let frame: RgbImage = frame
                .decode_image::<RgbFormat>()
                .expect("failed to decode frame image");

            let frame = CapturedFrame {
                timestamp: Instant::now(),
                image: frame,
            };

            app_tx
                .send(Event::NewFrame(Arc::new(frame)))
                .expect("failed to send frame to bus");
        }
    });

    tx
}
