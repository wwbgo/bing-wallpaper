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
use windows::core::PCWSTR;

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
