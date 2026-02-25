use std::{
    collections::HashMap,
    fs,
    io::BufReader,
    path::{Path, PathBuf},
    sync::{Arc, RwLock, Weak},
};

use anyhow::Result;
use image::{GrayImage, ImageReader, RgbImage};
use serde::Deserialize;
use tracing::error;

use crate::vision::zones::ZoneMap;

const TARGETS_PATH: &str = "targets";
const TARGET_PREVIEW_PATH: &str = "preview.png";
const TARGET_ZONEMAP_PATH: &str = "zonemap.png";
const TARGET_DATA_PATH: &str = "data.json";

#[derive(Deserialize)]
pub struct TargetData {
    pub zone_scores: HashMap<u8, u32>,
    pub name: String,
}

pub struct Target {
    id: String,
    name: String,
    zone_scores: HashMap<u8, u32>,
    loaded_zonemap: RwLock<Weak<ZoneMap>>,
}

impl Target {
    pub fn load_from_dir(dir: impl AsRef<Path>, id: String) -> Result<Self> {
        let data: TargetData = serde_json::from_reader(BufReader::new(fs::File::open(
            dir.as_ref().join(TARGET_DATA_PATH),
        )?))?;

        Ok(Self {
            id,
            name: data.name,
            zone_scores: data.zone_scores,
            loaded_zonemap: RwLock::new(Weak::new()),
        })
    }

    pub fn load_zonemap(&self) -> Result<Arc<ZoneMap>> {
        let mut loaded_zonemap = self
            .loaded_zonemap
            .write()
            .map_err(|_| anyhow::anyhow!("failed to lock loaded_zonemap"))?;

        if let Some(loaded_zonemap) = loaded_zonemap.upgrade() {
            return Ok(loaded_zonemap);
        }

        let dir = self.dir();
        let zonemap_img: GrayImage = ImageReader::open(dir)?.decode()?.into();
        let zonemap = Arc::new(ZoneMap::load(zonemap_img));

        *loaded_zonemap = Arc::<ZoneMap>::downgrade(&zonemap);

        Ok(zonemap)
    }

    pub fn dir(&self) -> PathBuf {
        Path::new(TARGETS_PATH).join(&self.id)
    }

    pub fn name(&self) -> &String {
        &self.name
    }

    pub fn id(&self) -> &String {
        &self.id
    }

    pub fn preview_path(&self) -> PathBuf {
        Path::new(TARGETS_PATH)
            .join(&self.id)
            .join(TARGET_PREVIEW_PATH)
    }
}

pub fn load_targets() -> HashMap<String, Target> {
    match fs::read_dir(TARGETS_PATH) {
        Err(e) => {
            error!("failed to load targets: {e:?}");
            return HashMap::new();
        }
        Ok(i) => i
            .filter_map(|target_dir| {
                let target_dir = target_dir
                    .inspect_err(|e| error!("failed to explore target dir: {e}"))
                    .ok()?;

                if !target_dir
                    .file_type()
                    .inspect_err(|e| {
                        error!("failed to check target dir '{target_dir:?}' type: {e:?}")
                    })
                    .ok()?
                    .is_dir()
                {
                    return None;
                }

                let id = target_dir
                    .file_name()
                    .into_string()
                    .inspect_err(|e| {
                        error!("failed to decode target dir '{target_dir:?}' path: {e:?}")
                    })
                    .ok()?;

                let target = Target::load_from_dir(target_dir.path(), id.clone())
                    .inspect_err(|e| {
                        error!("failed to load target '{id}' from dir '{target_dir:?}': {e:?}")
                    })
                    .ok()?;

                Some((id, target))
            })
            .collect(),
    }
}
