//! Window creation and titlebar color mode.

use std::error::Error;
use std::ffi;

use dwall::ColorScheme;
use tauri::{WebviewUrl, WebviewWindowBuilder};
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

/// Creates a window with the given parameters
pub fn build_window(
    app: &tauri::AppHandle,
    label: &str,
    title: &str,
) -> Result<(), Box<dyn Error>> {
    let window_builder = WebviewWindowBuilder::new(app, label, WebviewUrl::default())
        .title(title)
        .resizable(false)
        .maximizable(true)
        .transparent(true)
        .visible(cfg!(debug_assertions))
        .min_inner_size(1024., 640.)
        .maximized(true);

    match window_builder.build() {
        Ok(_) => Ok(()),
        Err(build_error) => Err(build_error.into()),
    }
}

/// Sets the window's titlebar color mode based on the system version
pub fn set_color_mode(window_handle: HWND, color_mode: ColorScheme) -> DwallSettingsResult<()> {
    if is_windows_11_or_newer() {
        set_windows_11_caption_color(window_handle, &color_mode)
    } else {
        set_legacy_dark_mode(window_handle, &color_mode)
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
