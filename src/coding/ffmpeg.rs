use image::RgbImage;
use std::io::Write;
use std::process::{Command, Stdio};

pub fn save_video_simple(
    frames: &[RgbImage],
    output_path: &str,
    fps: u32,
) -> Result<(), Box<dyn std::error::Error>> {
    if frames.is_empty() {
        return Err("No frames to save".into());
    }

    let (width, height) = frames[0].dimensions();

    let mut child = Command::new("ffmpeg")
        .args([
            "-y",
            "-f",
            "rawvideo",
            "-pixel_format",
            "rgb24",
            "-video_size",
            &format!("{}x{}", width, height),
            "-framerate",
            &fps.to_string(),
            "-i",
            "pipe:0",
            "-c:v",
            "libx264",
            "-pix_fmt",
            "yuv420p",
            output_path,
        ])
        .stdin(Stdio::piped())
        .spawn()?;

    let stdin = child.stdin.as_mut().ok_or("Failed to open stdin")?;

    for frame in frames {
        stdin.write_all(frame.as_raw())?;
    }

    child.wait()?;
    Ok(())
}
