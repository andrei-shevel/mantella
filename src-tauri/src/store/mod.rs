mod model;

pub use model::{FileState, Settings};

use crate::error::{AppError, Result};
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// JSON-backed application store living in the app data directory:
/// `settings.json` for global settings, `files.json` for per-file state.
pub struct Store {
    dir: PathBuf,
    pub settings: Settings,
    pub files: HashMap<String, FileState>,
}

impl Store {
    pub fn load(dir: PathBuf) -> Self {
        let settings = read_json(&dir.join("settings.json")).unwrap_or_default();
        let files = read_json(&dir.join("files.json")).unwrap_or_default();
        Self { dir, settings, files }
    }

    pub fn save_settings(&self) -> Result<()> {
        write_json_atomic(&self.dir.join("settings.json"), &self.settings)
    }

    pub fn save_files(&self) -> Result<()> {
        write_json_atomic(&self.dir.join("files.json"), &self.files)
    }
}

fn read_json<T: DeserializeOwned>(path: &Path) -> Option<T> {
    let bytes = fs::read(path).ok()?;
    serde_json::from_slice(&bytes).ok()
}

fn write_json_atomic<T: Serialize>(path: &Path, value: &T) -> Result<()> {
    let json = serde_json::to_vec_pretty(value).map_err(|e| AppError::Message(e.to_string()))?;
    let tmp = path.with_extension("tmp");
    fs::write(&tmp, json)?;
    fs::rename(&tmp, path)?;
    Ok(())
}
