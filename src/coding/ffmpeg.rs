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

use image::ImageBuffer;

pub fn load_video(input_path: &Path) -> Result<(Vec<RgbImage>, u32), Box<dyn Error>> {
    // 1. Get video dimensions, frame count, and fps using ffprobe
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
        return Err("ffprobe failed to read video metadata".into());
    }

    let probe_str = String::from_utf8(probe_output.stdout)?;
    let parts: Vec<&str> = probe_str.trim().split(',').collect();

    if parts.len() < 3 {
        return Err("Failed to parse video metadata".into());
    }

    let width: u32 = parts[0].parse()?;
    let height: u32 = parts[1].parse()?;

    // Parse fps (format is typically "30/1" or "24000/1001")
    let fps_parts: Vec<&str> = parts[2].split('/').collect();
    let fps = if fps_parts.len() == 2 {
        let numerator: f64 = fps_parts[0].parse()?;
        let denominator: f64 = fps_parts[1].parse()?;
        (numerator / denominator).round() as u32
    } else {
        parts[2].parse()?
    };

    // 2. Launch ffmpeg to decode video to raw RGB24 frames
    let mut child = Command::new("ffmpeg")
        .args([
            "-i",
            input_path.to_str().ok_or("Invalid path")?,
            "-f",
            "rawvideo", // Output format is raw video
            "-pixel_format",
            "rgb24", // Output pixel format matching RgbImage
            "-",     // Write output to stdout
        ])
        .stdout(Stdio::piped()) // Capture ffmpeg's stdout
        .stderr(Stdio::null()) // Show errors in console
        .spawn()?;

    // 3. Read raw RGB24 data from ffmpeg stdout
    let mut stdout = child.stdout.take().ok_or("Failed to open stdout")?;
    let mut buffer = Vec::new();
    stdout.read_to_end(&mut buffer)?;

    // 4. Wait for ffmpeg to finish
    let status = child.wait()?;
    if !status.success() {
        return Err(format!("FFmpeg exited with error code: {}", status).into());
    }

    // 5. Convert raw bytes into RgbImage frames
    let frame_size = (width * height * 3) as usize; // 3 bytes per pixel (RGB)
    let mut frames = Vec::new();

    for chunk in buffer.chunks_exact(frame_size) {
        let img = ImageBuffer::from_raw(width, height, chunk.to_vec())
            .ok_or("Failed to create image from raw data")?;
        frames.push(img);
    }

    Ok((frames, fps))
}
