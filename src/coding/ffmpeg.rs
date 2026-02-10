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
        .stdout(Stdio::null()) // Show ffmpeg logs in console (optional)
        .stderr(Stdio::null()) // Show errors in console
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

use std::io::Read;

pub fn load_video(input_path: &Path) -> Result<(Vec<RgbImage>, u32), Box<dyn Error>> {
    // 1. Get Video Metadata (Width, Height, FPS) using ffprobe
    // We need these to calculate buffer size and return the correct FPS.
    // Command: ffprobe -v error -select_streams v:0 -show_entries stream=width,height,r_frame_rate -of csv=p=0 input.mp4
    let probe_output = Command::new("ffprobe")
        .args([
            "-v",
            "error",
            "-select_streams",
            "v:0",
            "-show_entries",
            "stream=width,height,r_frame_rate",
            "-of",
            "csv=p=0",
            input_path.to_str().ok_or("Invalid path")?,
        ])
        .output()?;

    if !probe_output.status.success() {
        return Err("Failed to probe video file (is ffprobe installed?)".into());
    }

    let output_str = String::from_utf8(probe_output.stdout)?;
    let parts: Vec<&str> = output_str.trim().split(',').collect();

    if parts.len() != 3 {
        return Err("Failed to parse ffprobe output".into());
    }

    let width: u32 = parts[0].parse()?;
    let height: u32 = parts[1].parse()?;

    // FPS often comes as "30/1" or "30000/1001". We parse and round to nearest u32.
    let fps_parts: Vec<&str> = parts[2].split('/').collect();
    let fps_num: f64 = fps_parts[0].parse()?;
    let fps_den: f64 = fps_parts.get(1).unwrap_or(&"1").parse()?;
    let fps = (fps_num / fps_den).round() as u32;

    // 2. Launch ffmpeg to decode video to raw RGB24
    let mut child = Command::new("ffmpeg")
        .args([
            "-i",
            input_path.to_str().ok_or("Invalid path")?,
            "-f",
            "rawvideo",
            "-pix_fmt",
            "rgb24",
            "-", // Output to stdout
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::null()) // Hide logs
        .spawn()?;

    let mut stdout = child.stdout.take().ok_or("Failed to open stdout")?;

    // 3. Read frames from stdout
    let mut frames = Vec::new();
    // RGB24 means 3 bytes per pixel
    let frame_size = (width * height * 3) as usize;
    let mut buffer = vec![0u8; frame_size];

    // Read exact number of bytes for one frame repeatedly until EOF
    while stdout.read_exact(&mut buffer).is_ok() {
        // Create RgbImage from the raw buffer
        if let Some(img) = RgbImage::from_raw(width, height, buffer.clone()) {
            frames.push(img);
        }
    }

    // Wait for ffmpeg to finish cleanly
    let status = child.wait()?;
    if !status.success() {
        // You might choose to ignore this if you successfully read frames,
        // but it's good practice to check.
        return Err("ffmpeg process finished with error".into());
    }

    Ok((frames, fps))
}
