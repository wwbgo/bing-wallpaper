use std::mem::size_of;

use anyhow::{Result, bail};
use windows::Win32::Foundation::LPARAM;
use windows::Win32::Graphics::Gdi::{
    EnumDisplayMonitors, GetMonitorInfoW, HDC, HMONITOR, MONITORINFO,
};
use windows::core::BOOL;

use crate::cli::Resolution;

pub fn choose_auto_resolution(width: i32, height: i32) -> Resolution {
    if width >= 2560 || height >= 1440 {
        Resolution::Uhd
    } else {
        Resolution::Fhd
    }
}

pub fn auto_resolution() -> Result<Resolution> {
    let mut bounds = MonitorBounds::default();

    let result = unsafe {
        EnumDisplayMonitors(
            None,
            None,
            Some(monitor_enum_proc),
            LPARAM(&mut bounds as *mut MonitorBounds as isize),
        )
    };

    if !result.as_bool() {
        bail!("failed to enumerate display monitors");
    }

    if bounds.max_width == 0 && bounds.max_height == 0 {
        bail!("no display monitors were found");
    }

    Ok(choose_auto_resolution(bounds.max_width, bounds.max_height))
}

#[derive(Default)]
struct MonitorBounds {
    max_width: i32,
    max_height: i32,
}

unsafe extern "system" fn monitor_enum_proc(
    monitor: HMONITOR,
    _hdc: HDC,
    _rect: *mut windows::Win32::Foundation::RECT,
    data: LPARAM,
) -> BOOL {
    let bounds = unsafe { &mut *(data.0 as *mut MonitorBounds) };

    let mut info = MONITORINFO {
        cbSize: size_of::<MONITORINFO>() as u32,
        ..MONITORINFO::default()
    };

    if unsafe { GetMonitorInfoW(monitor, &mut info) }.as_bool() {
        let width = info.rcMonitor.right - info.rcMonitor.left;
        let height = info.rcMonitor.bottom - info.rcMonitor.top;
        bounds.max_width = bounds.max_width.max(width);
        bounds.max_height = bounds.max_height.max(height);
    }

    true.into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn auto_uses_uhd_for_qhd_and_above() {
        assert_eq!(choose_auto_resolution(2560, 1440), Resolution::Uhd);
        assert_eq!(choose_auto_resolution(1920, 1440), Resolution::Uhd);
        assert_eq!(choose_auto_resolution(2560, 1080), Resolution::Uhd);
    }

    #[test]
    fn auto_uses_fhd_below_qhd() {
        assert_eq!(choose_auto_resolution(1920, 1080), Resolution::Fhd);
        assert_eq!(choose_auto_resolution(1366, 768), Resolution::Fhd);
    }
}
