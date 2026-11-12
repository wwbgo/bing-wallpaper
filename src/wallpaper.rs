use std::iter;
use std::os::windows::ffi::OsStrExt;
use std::path::Path;
use std::time::Duration;

use anyhow::{Context, Result};
use windows::Storage::StorageFile;
use windows::System::UserProfile::LockScreen;
use windows::Win32::System::Com::{
    CLSCTX_LOCAL_SERVER, COINIT_APARTMENTTHREADED, CoCreateInstance, CoInitializeEx,
};
use windows::Win32::System::WinRT::{RO_INIT_MULTITHREADED, RoInitialize, RoUninitialize};
use windows::Win32::UI::Shell::{
    DESKTOP_WALLPAPER_POSITION, DWPOS_CENTER, DWPOS_FILL, DWPOS_FIT, DWPOS_SPAN, DWPOS_STRETCH,
    DWPOS_TILE, DesktopWallpaper, IDesktopWallpaper,
};
use windows::Win32::UI::WindowsAndMessaging::{
    SPI_SETDESKWALLPAPER, SPIF_SENDCHANGE, SPIF_UPDATEINIFILE, SystemParametersInfoW,
};
use windows::core::{HSTRING, PCWSTR};
use winreg::RegKey;
use winreg::enums::HKEY_CURRENT_USER;

use crate::cli::WallpaperStyle;

pub fn set_wallpaper(path: &Path, style: WallpaperStyle) -> Result<()> {
    let position = wallpaper_position(style);

    unsafe {
        CoInitializeEx(None, COINIT_APARTMENTTHREADED)
            .ok()
            .context("failed to initialize COM")?;

        let desktop: IDesktopWallpaper =
            CoCreateInstance(&DesktopWallpaper, None, CLSCTX_LOCAL_SERVER)
                .context("failed to create IDesktopWallpaper COM instance")?;

        let wide_path: Vec<u16> = path
            .as_os_str()
            .encode_wide()
            .chain(iter::once(0))
            .collect();

        desktop
            .SetWallpaper(None, PCWSTR(wide_path.as_ptr()))
            .context("failed to set desktop wallpaper")?;
        desktop
            .SetPosition(position)
            .context("failed to set desktop wallpaper position")?;
    }

    persist_wallpaper_registry(path, style)?;
    Ok(())
}

const LOCK_SCREEN_TIMEOUT: Duration = Duration::from_secs(30);

pub fn set_lock_screen(path: &Path) -> Result<()> {
    let path = path.to_path_buf();
    // Desktop wallpaper uses STA; wait for WinRT async operations on a separate MTA thread.
    run_lock_screen_task(LOCK_SCREEN_TIMEOUT, move || -> Result<()> {
        // SAFETY: this fresh thread initializes its own MTA; RuntimeGuard balances
        // each successful initialization on the same thread after WinRT values drop.
        unsafe { RoInitialize(RO_INIT_MULTITHREADED) }
            .context("failed to initialize Windows Runtime for lock screen")?;
        let _runtime = RuntimeGuard;

        let file = StorageFile::GetFileFromPathAsync(&HSTRING::from(path.as_os_str()))
            .context("failed to open lock screen image")?
            .join()
            .context("failed to load lock screen image")?;

        LockScreen::SetImageFileAsync(&file)
            .context("failed to start setting lock screen image")?
            .join()
            .context("failed to set lock screen image")?;

        Ok(())
    })
}

fn run_lock_screen_task(
    timeout: Duration,
    task: impl FnOnce() -> Result<()> + Send + 'static,
) -> Result<()> {
    let (sender, receiver) = std::sync::mpsc::sync_channel(1);
    std::thread::Builder::new()
        .name("lock-screen-update".into())
        .spawn(move || {
            let _ = sender.send(task());
        })
        .context("failed to start lock screen update thread")?;

    // Do not join a stuck WinRT thread, including one stuck in activation or
    // teardown. The CLI can report failure, release its mutex, and exit; Windows
    // reclaims the background thread when the process exits.
    match receiver.recv_timeout(timeout) {
        Ok(result) => result,
        Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
            anyhow::bail!("lock screen update timed out after {timeout:?}")
        }
        Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
            anyhow::bail!("lock screen update thread exited without a result")
        }
    }
}

struct RuntimeGuard;

impl Drop for RuntimeGuard {
    fn drop(&mut self) {
        // SAFETY: the guard stays on the initializing thread and is constructed
        // only after RoInitialize succeeds.
        unsafe { RoUninitialize() };
    }
}

fn wallpaper_position(style: WallpaperStyle) -> DESKTOP_WALLPAPER_POSITION {
    match style {
        WallpaperStyle::Fill => DWPOS_FILL,
        WallpaperStyle::Fit => DWPOS_FIT,
        WallpaperStyle::Stretch => DWPOS_STRETCH,
        WallpaperStyle::Center => DWPOS_CENTER,
        WallpaperStyle::Span => DWPOS_SPAN,
        WallpaperStyle::Tile => DWPOS_TILE,
    }
}

fn persist_wallpaper_registry(path: &Path, style: WallpaperStyle) -> Result<()> {
    force_picture_mode_registry()?;

    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let (desktop, _) = hkcu
        .create_subkey(r"Control Panel\Desktop")
        .context("failed to open desktop wallpaper registry key")?;

    let (tile, wallpaper_style) = registry_style_values(style);
    desktop
        .set_value("TileWallpaper", &tile)
        .context("failed to update TileWallpaper registry value")?;
    desktop
        .set_value("WallpaperStyle", &wallpaper_style)
        .context("failed to update WallpaperStyle registry value")?;
    desktop
        .set_value("WallPaper", &path.as_os_str())
        .context("failed to update WallPaper registry value")?;

    let mut wide_path: Vec<u16> = path
        .as_os_str()
        .encode_wide()
        .chain(iter::once(0))
        .collect();

    unsafe {
        SystemParametersInfoW(
            SPI_SETDESKWALLPAPER,
            0,
            Some(wide_path.as_mut_ptr() as *mut _),
            SPIF_UPDATEINIFILE | SPIF_SENDCHANGE,
        )
        .context("failed to refresh desktop wallpaper settings")?;
    }

    Ok(())
}

fn registry_style_values(style: WallpaperStyle) -> (&'static str, &'static str) {
    match style {
        WallpaperStyle::Center => ("0", "0"),
        WallpaperStyle::Tile => ("1", "0"),
        WallpaperStyle::Fit => ("0", "6"),
        WallpaperStyle::Stretch => ("0", "2"),
        WallpaperStyle::Span => ("0", "22"),
        WallpaperStyle::Fill => ("0", "10"),
    }
}

fn force_picture_mode_registry() -> Result<()> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);

    let (wallpapers, _) = hkcu
        .create_subkey(r"Software\Microsoft\Windows\CurrentVersion\Explorer\Wallpapers")
        .context("failed to open Explorer Wallpapers registry key")?;
    wallpapers
        .set_value("BackgroundType", &0u32)
        .context("failed to set BackgroundType registry value")?;
    wallpapers
        .set_value("SlideshowEnabled", &0u32)
        .context("failed to set SlideshowEnabled registry value")?;

    let (spotlight, _) = hkcu
        .create_subkey(r"Software\Microsoft\Windows\CurrentVersion\DesktopSpotlight\Settings")
        .context("failed to open DesktopSpotlight Settings registry key")?;
    spotlight
        .set_value("EnabledState", &0u32)
        .context("failed to set DesktopSpotlight EnabledState registry value")?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lock_screen_task_times_out_without_waiting_for_the_blocked_thread() {
        let (release_tx, release_rx) = std::sync::mpsc::channel();
        let (finished_tx, finished_rx) = std::sync::mpsc::channel();
        let releaser = std::thread::spawn(move || {
            // Also release on the broken implementation so a failed test cannot hang.
            let _ = finished_rx.recv_timeout(Duration::from_millis(500));
            let _ = release_tx.send(());
        });
        let result = run_lock_screen_task(Duration::from_millis(20), move || {
            release_rx.recv().unwrap();
            Ok(())
        });
        let _ = finished_tx.send(());
        releaser.join().unwrap();
        assert!(result.is_err(), "blocked lock screen task must time out");
        assert!(result.unwrap_err().to_string().contains("timed out"));
    }

    #[test]
    fn lock_screen_task_preserves_success_and_errors() {
        run_lock_screen_task(Duration::from_secs(2), || Ok(())).unwrap();
        let error = run_lock_screen_task(Duration::from_secs(2), || {
            anyhow::bail!("lock screen denied")
        })
        .unwrap_err();
        assert_eq!(error.to_string(), "lock screen denied");
    }

    #[test]
    fn missing_lock_screen_image_reports_file_error_from_uninitialized_caller() {
        let dir =
            std::env::temp_dir().join(format!("bing-wallpaper-missing-{}", std::process::id()));
        std::fs::create_dir(&dir).unwrap();
        let result = set_lock_screen(&dir.join("missing.jpg"));
        std::fs::remove_dir(&dir).unwrap();
        let error = result.unwrap_err();
        let windows_error = error.downcast_ref::<windows::core::Error>().unwrap();
        assert_eq!(
            windows_error.code(),
            windows::core::HRESULT::from_win32(2),
            "expected missing image, got {error:#}"
        );
    }
}
