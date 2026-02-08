use anyhow::Result;
use chrono::{DateTime, Local, TimeZone};
use image::RgbImage;
use std::{fs, io::BufReader, io::BufWriter, path::PathBuf};

use crate::hits::storage::{HitData, HitStorage};

const TIMESTAMP_DIR_FORMAT: &'static str = "%Y-%m-%d_%H-%M-%S.%.3f";

pub struct FileHitStorage {
    base: PathBuf,
}

impl FileHitStorage {
    pub fn new(base: impl Into<PathBuf>) -> Self {
        FileHitStorage { base: base.into() }
    }

    fn dir_for(&self, timestamp: DateTime<Local>) -> PathBuf {
        let name = timestamp.format(TIMESTAMP_DIR_FORMAT).to_string();
        self.base.join(name)
    }
}

impl HitStorage for FileHitStorage {
    fn save_clip(&mut self, timestamp: DateTime<Local>, clip: (&[RgbImage], u32)) -> Result<()> {
        let (frames, fps) = clip;
        let dir = self.dir_for(timestamp);
        fs::create_dir_all(&dir)?;
        let clip_path = dir.join("clip.mp4");
        crate::coding::ffmpeg::save_video(frames, fps, &clip_path)
            .map_err(|e| anyhow::anyhow!(e.to_string()))?;
        Ok(())
    }

    fn load_clip(&mut self, timestamp: DateTime<Local>) -> Result<(Vec<RgbImage>, u32)> {
        let dir = self.dir_for(timestamp);
        let clip_path = dir.join("clip.mp4");
        let (frames, fps) = crate::coding::ffmpeg::load_video(&clip_path)
            .map_err(|e| anyhow::anyhow!(e.to_string()))?;
        Ok((frames, fps))
    }

    fn save_data(&mut self, timestamp: DateTime<Local>, data: HitData) -> Result<()> {
        let dir = self.dir_for(timestamp);
        fs::create_dir_all(&dir)?;
        let data_path = dir.join("data.json");
        let f = fs::File::create(data_path)?;
        let w = BufWriter::new(f);
        serde_json::to_writer_pretty(w, &data)?;
        Ok(())
    }

    fn load_data(&mut self, timestamp: DateTime<Local>) -> Result<HitData> {
        let dir = self.dir_for(timestamp);
        let data_path = dir.join("data.json");
        let f = fs::File::open(data_path)?;
        let r = BufReader::new(f);
        let data = serde_json::from_reader(r)?;
        Ok(data)
    }

    fn new_hit(
        &mut self,
        timestamp: DateTime<Local>,
        clip: (&[RgbImage], u32),
        data: HitData,
    ) -> Result<()> {
        self.save_clip(timestamp, clip)?;
        self.save_data(timestamp, data)?;
        Ok(())
    }

    fn get_unprocessed_hits_old_sorted(&mut self) -> Result<Vec<DateTime<Local>>> {
        let mut out = Vec::new();
        if !self.base.exists() {
            return Ok(out);
        }

        for entry in fs::read_dir(&self.base)? {
            let entry = entry?;
            if !entry.file_type()?.is_dir() {
                continue;
            }

            let name = entry.file_name().to_string_lossy().to_string();
            if let Ok(naive) = chrono::NaiveDateTime::parse_from_str(&name, TIMESTAMP_DIR_FORMAT) {
                if let Some(dt) = Local.from_local_datetime(&naive).single() {
                    // try to read data.json and check processed field
                    let data_path = entry.path().join("data.json");
                    if data_path.exists() {
                        if let Ok(f) = fs::File::open(&data_path) {
                            let res: Result<HitData, _> =
                                serde_json::from_reader(BufReader::new(f));
                            if let Ok(hit) = res {
                                if hit.processed.is_none() {
                                    out.push(dt);
                                }
                            }
                        }
                    }
                }
            }
        }

        // sort oldest -> newest
        out.sort();

        Ok(out)
    }
}
