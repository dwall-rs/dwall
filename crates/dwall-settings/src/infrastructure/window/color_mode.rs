//! Window color mode management (pure technical)

use std::ffi;

use dwall::ColorScheme;
use windows::{
    Wdk::System::SystemServices::RtlGetVersion,
    Win32::{
        Foundation::{HWND, STATUS_SUCCESS},
        Graphics::Dwm::{
            DWMWA_CAPTION_COLOR, DWMWA_USE_IMMERSIVE_DARK_MODE, DwmSetWindowAttribute,
        },
        System::SystemInformation::OSVERSIONINFOW,
    },
};

use crate::error::DwallSettingsResult;

/// Sets the window's titlebar color mode based on the system version
pub fn set_color_mode(window_handle: HWND, color_mode: ColorScheme) -> DwallSettingsResult<()> {
    let result = if is_windows_11_or_newer() {
        set_windows_11_caption_color(window_handle, &color_mode)
    } else {
        set_legacy_dark_mode(window_handle, &color_mode)
    };

    match result {
        Ok(_) => Ok(()),
        Err(error) => Err(error),
    }
}

fn set_windows_11_caption_color(
    window_handle: HWND,
    color_mode: &ColorScheme,
) -> DwallSettingsResult<()> {
    const DARK_CAPTION_COLOR: u32 = 0x171717;
    const LIGHT_CAPTION_COLOR: u32 = 0xFAFAFA;

    let caption_color = match color_mode {
        ColorScheme::Dark => DARK_CAPTION_COLOR,
        ColorScheme::Light => LIGHT_CAPTION_COLOR,
    };

    unsafe {
        DwmSetWindowAttribute(
            window_handle,
            DWMWA_CAPTION_COLOR,
            &caption_color as *const u32 as *const ffi::c_void,
            std::mem::size_of::<u32>() as u32,
        )
        .map_err(Into::into)
    }
}

fn set_legacy_dark_mode(window_handle: HWND, color_mode: &ColorScheme) -> DwallSettingsResult<()> {
    let dark_mode_value: u32 = match color_mode {
        ColorScheme::Dark => 1,
        ColorScheme::Light => 0,
    };

    unsafe {
        DwmSetWindowAttribute(
            window_handle,
            DWMWA_USE_IMMERSIVE_DARK_MODE,
            &dark_mode_value as *const _ as *const ffi::c_void,
            std::mem::size_of::<u32>() as u32,
        )
        .map_err(Into::into)
    }
}

fn is_windows_11_or_newer() -> bool {
    let mut os_version_info = OSVERSIONINFOW {
        dwOSVersionInfoSize: std::mem::size_of::<OSVERSIONINFOW>() as u32,
        ..Default::default()
    };

    let version_check_status = unsafe { RtlGetVersion(&mut os_version_info) };

    if version_check_status != STATUS_SUCCESS {
        return false;
    }

    os_version_info.dwBuildNumber >= 22000
}
