use std::{
    error::Error,
    io::Write,
    path::Path,
    process::{Command, Stdio},
};

use image::RgbImage;

pub fn save_video(frames: &[RgbImage], fps: u32, output_path: &Path) -> Result<(), Box<dyn Error>> {
    if frames.is_empty() {
        return Err("No frames to save".into());
    }

    let (width, height) = frames[0].dimensions();

    // 1. Launch the ffmpeg subprocess
    // We direct it to read raw RGB video from stdin and output MP4 (H.264)
    let mut child = Command::new("ffmpeg")
        .args([
            "-y", // Overwrite output file if it exists
            "-f",
            "rawvideo", // Input format is raw video
            "-pixel_format",
            "rgb24", // Input pixel format (RgbImage is rgb24)
            "-video_size",
            &format!("{}x{}", width, height),
            "-framerate",
            &fps.to_string(),
            "-i",
            "-", // Read input from stdin
            "-c:v",
            "libx264", // Encode using H.264 (standard)
            "-pix_fmt",
            "yuv420p", // Ensure compatibility with browsers/players
            "-preset",
            "medium", // Balance between speed and compression
            output_path.to_str().ok_or("Invalid path")?,
        ])
        .stdin(Stdio::piped()) // Pipe our data to ffmpeg's stdin
        .stdout(Stdio::inherit()) // Show ffmpeg logs in console (optional)
        .stderr(Stdio::inherit())
        .spawn()?;

    // 2. Write frames to the ffmpeg stdin
    let mut stdin = child.stdin.take().ok_or("Failed to open stdin")?;

    for frame in frames {
        // Verify dimensions to prevent ffmpeg errors
        if frame.dimensions() != (width, height) {
            return Err("All frames must have the same dimensions".into());
        }

        // Write the raw bytes (R, G, B, R, G, B...)
        stdin.write(frame.as_raw())?;
    }

    // 3. Close stdin to signal EOF and wait for ffmpeg to finish
    drop(stdin); // Explicitly close stdin

    let status = child.wait()?;

    if status.success() {
        Ok(())
    } else {
        Err(format!("FFmpeg exited with error code: {}", status).into())
    }
}
