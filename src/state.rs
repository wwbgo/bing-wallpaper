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
    #[serde(default)]
    pub last_resolution: Option<String>,
    #[serde(default)]
    pub lock_screen_synced: bool,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn old_state_without_last_resolution_deserializes() {
        let state: AppState = serde_json::from_str(
            r#"{"last_startdate":"20260903","last_image":"C:/img.jpg","copyright":"test","updated_at":"2026-09-03 09:00:00","last_local_run_date":"20260903"}"#,
        )
        .unwrap();

        assert_eq!(state.last_resolution, None);
        assert!(!state.lock_screen_synced);
    }

    #[test]
    fn previous_resolution_state_requires_lock_screen_migration() {
        let state: AppState = serde_json::from_str(
            r#"{"last_startdate":"20260903","last_image":"C:/img.jpg","copyright":"test","updated_at":"2026-09-03 09:00:00","last_local_run_date":"20260903","last_resolution":"uhd"}"#,
        )
        .unwrap();

        assert_eq!(state.last_resolution.as_deref(), Some("uhd"));
        assert!(!state.lock_screen_synced);
    }

    #[test]
    fn lock_screen_sync_status_survives_atomic_state_save() {
        let dir = std::env::temp_dir().join(format!("bing-wallpaper-state-{}", std::process::id()));
        let path = dir.join("state.json");
        let mut state = AppState {
            lock_screen_synced: true,
            ..AppState::default()
        };
        state.save(&path).unwrap();
        assert!(AppState::load(&path).unwrap().unwrap().lock_screen_synced);

        state.lock_screen_synced = false;
        state.save(&path).unwrap();
        assert!(!AppState::load(&path).unwrap().unwrap().lock_screen_synced);
        fs::remove_dir_all(dir).unwrap();
    }
}
