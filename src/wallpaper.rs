use std::iter;
use std::os::windows::ffi::OsStrExt;
use std::path::Path;

use anyhow::{Context, Result};
use windows::Win32::System::Com::{
    CLSCTX_LOCAL_SERVER, COINIT_APARTMENTTHREADED, CoCreateInstance, CoInitializeEx,
};
use windows::Win32::UI::Shell::{
    DESKTOP_WALLPAPER_POSITION, DWPOS_CENTER, DWPOS_FILL, DWPOS_FIT, DWPOS_SPAN, DWPOS_STRETCH,
    DWPOS_TILE, DesktopWallpaper, IDesktopWallpaper,
};
use windows::Win32::UI::WindowsAndMessaging::{
    SPI_SETDESKWALLPAPER, SPIF_SENDCHANGE, SPIF_UPDATEINIFILE, SystemParametersInfoW,
};
use windows::core::PCWSTR;
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
