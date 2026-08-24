use std::fs;
use std::iter;
use std::os::windows::ffi::OsStrExt;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use windows::Win32::Storage::FileSystem::{
    MOVEFILE_REPLACE_EXISTING, MOVEFILE_WRITE_THROUGH, MoveFileExW,
};
use windows::core::PCWSTR;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AppState {
    pub last_startdate: String,
    pub last_image: PathBuf,
    pub copyright: String,
    pub updated_at: String,
    #[serde(default)]
    pub last_local_run_date: String,
}

impl AppState {
    pub fn load(path: &Path) -> Result<Option<Self>> {
        if !path.exists() {
            return Ok(None);
        }

        let raw = fs::read_to_string(path)
            .with_context(|| format!("failed to read state file {}", path.display()))?;
        let state = serde_json::from_str(&raw)
            .with_context(|| format!("failed to parse state file {}", path.display()))?;
        Ok(Some(state))
    }

    pub fn save(&self, path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("failed to create directory {}", parent.display()))?;
        }

        let tmp = path.with_extension("json.tmp");
        let data = serde_json::to_vec_pretty(self)?;
        fs::write(&tmp, data)
            .with_context(|| format!("failed to write state file {}", tmp.display()))?;
        atomic_replace(&tmp, path)?;
        Ok(())
    }
}

fn atomic_replace(tmp: &Path, destination: &Path) -> Result<()> {
    let tmp_wide = wide_path(tmp);
    let destination_wide = wide_path(destination);

    unsafe {
        MoveFileExW(
            PCWSTR(tmp_wide.as_ptr()),
            PCWSTR(destination_wide.as_ptr()),
            MOVEFILE_REPLACE_EXISTING | MOVEFILE_WRITE_THROUGH,
        )
        .with_context(|| {
            format!(
                "failed to replace state file {} with {}",
                destination.display(),
                tmp.display()
            )
        })?;
    }

    Ok(())
}

fn wide_path(path: &Path) -> Vec<u16> {
    path.as_os_str()
        .encode_wide()
        .chain(iter::once(0))
        .collect()
}
