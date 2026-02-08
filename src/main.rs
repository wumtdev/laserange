slint::include_modules!();

use slint::ComponentHandle;
use tracing::info;

mod bus;
mod capturer;
mod coding;
mod hits;
mod recorder;
mod targets;
mod util;
mod vision;

fn main() {
    tracing_subscriber::fmt().init();

    let ui = MainWindow::new().unwrap();
    let ui_weak = ui.as_weak();

    // Start app event loop
    let (bus_tx, bus_rx) = crate::bus::start();

    // UI event handling thread
    std::thread::spawn(move || {
        for msg in bus_rx.iter() {
            match msg {
                crate::bus::AppMessage::FrameReady {
                    camera_frame,
                    target_frame,
                } => {
                    let ui = ui_weak.clone();
                    slint::invoke_from_event_loop(move || {
                        // Create camera image and set it
                        let buffer = slint::SharedPixelBuffer::<slint::Rgb8Pixel>::clone_from_slice(
                            camera_frame.as_raw(),
                            camera_frame.width(),
                            camera_frame.height(),
                        );
                        let image = slint::Image::from_rgb8(buffer);
                        let ui = ui.upgrade().unwrap();
                        ui.set_camera_frame(image);

                        // Set target frame if available
                        if let Some(target_img) = &target_frame {
                            let buffer =
                                slint::SharedPixelBuffer::<slint::Rgb8Pixel>::clone_from_slice(
                                    target_img.as_raw(),
                                    target_img.width(),
                                    target_img.height(),
                                );
                            let image = slint::Image::from_rgb8(buffer);
                            ui.set_target_frame(image);
                        } else {
                            ui.set_target_frame(slint::Image::default());
                        }
                    })
                    .ok();
                }
            }
        }
    });

    // UI stencil change handler
    ui.global::<TargetStencil>()
        .on_change(move |start_x, start_y, end_x, end_y| {
            info!("changed {start_x}, {start_y}, {end_x}, {end_y}");
            bus_tx
                .send(crate::bus::AppCommand::NewStencil((
                    start_x, start_y, end_x, end_y,
                )))
                .unwrap();
        });

    ui.run().unwrap();
}
