use std::collections::HashMap;

use anyhow::Result;
use chrono::{DateTime, Local, TimeZone};
use image::RgbImage;
use serde::{Deserialize, Serialize};

use crate::{hits::processor::HitProcessResult, targets::TargetInfo};

#[derive(Serialize, Deserialize, Clone)]
pub struct HitData {
    pub target_info: TargetInfo,
    pub processed: Option<HitProcessResult>,
}

pub trait HitStorage: Send {
    fn save_clip(&mut self, timestamp: DateTime<Local>, clip: (&[RgbImage], u32)) -> Result<()>;
    fn load_clip(&mut self, timestamp: DateTime<Local>) -> Result<(Vec<RgbImage>, u32)>;

    fn save_data(&mut self, timestamp: DateTime<Local>, data: HitData) -> Result<()>;
    fn load_data(&mut self, timestamp: DateTime<Local>) -> Result<HitData>;

    fn new_hit(
        &mut self,
        timestamp: DateTime<Local>,
        clip: (&[RgbImage], u32),
        data: HitData,
    ) -> Result<()>;

    fn get_unprocessed_hits_old_sorted(&mut self) -> Result<Vec<DateTime<Local>>>;
    fn get_all_hits(&mut self) -> Result<HashMap<DateTime<Local>, HitData>>;
}

pub mod file;

pub use file::FileHitStorage;
