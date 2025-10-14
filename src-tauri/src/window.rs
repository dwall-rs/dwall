use core::ffi;
use std::error::Error;

use dwall::ColorScheme;
use tauri::{WebviewUrl, WebviewWindowBuilder};
use windows::{
    Wdk::System::SystemServices::RtlGetVersion,
    Win32::{
        Foundation::{HWND, STATUS_SUCCESS},
        Graphics::Dwm::{
            DwmSetWindowAttribute, DWMWA_CAPTION_COLOR, DWMWA_USE_IMMERSIVE_DARK_MODE,
        },
        System::SystemInformation::OSVERSIONINFOW,
    },
};

use crate::error::DwallSettingsResult;

/// Creates the main application window with predefined settings
///
/// # Arguments
/// * `app` - The Tauri application handle
///
/// # Returns
/// A result indicating successful window creation or an error
pub fn create_main_window(app: &tauri::AppHandle) -> Result<(), Box<dyn Error>> {
    trace!("Initializing main application window");

    // Define window configuration parameters
    const WINDOW_TITLE: &str = "Dwall Settings";
    const WINDOW_WIDTH: f64 = 660.0;
    const WINDOW_HEIGHT: f64 = 600.0;

    debug!(
        title = WINDOW_TITLE,
        width = WINDOW_WIDTH,
        height = WINDOW_HEIGHT,
        "Configuring window"
    );

    let window_builder = WebviewWindowBuilder::new(app, "main", WebviewUrl::default())
        .title(WINDOW_TITLE)
        .resizable(false)
        .maximizable(false)
        .visible(cfg!(debug_assertions))
        .inner_size(WINDOW_WIDTH, WINDOW_HEIGHT);

    // Attempt to build the window
    match window_builder.build() {
        Ok(_) => {
            info!("Main application window created successfully");
            Ok(())
        }
        Err(build_error) => {
            error!(error = ?build_error, "Failed to create main window");
            Err(build_error.into())
        }
    }
}

/// Sets the window's titlebar color mode based on the system version
///
/// # Arguments
/// * `window_handle` - The window handle
/// * `color_mode` - The desired color mode (Dark or Light)
///
/// # Returns
/// A result indicating successful color mode change or an error
pub fn set_window_color_mode(
    window_handle: HWND,
    color_mode: ColorScheme,
) -> DwallSettingsResult<()> {
    trace!(mode = ?color_mode, "Attempting to set window color mode");

    // Determine color based on Windows version
    let result = if is_windows_11_or_newer() {
        set_windows_11_caption_color(window_handle, &color_mode)
    } else {
        set_legacy_dark_mode(window_handle, &color_mode)
    };

    match result {
        Ok(_) => {
            info!(mode = ?color_mode, "Window color mode set successfully");
            Ok(())
        }
        Err(error) => {
            error!(mode = ?color_mode, error = %error, "Failed to set window color mode");
            Err(error)
        }
    }
}

/// Sets caption color for Windows 11 and newer
///
/// # Arguments
/// * `window_handle` - The window handle
/// * `color_mode` - The desired color mode
///
/// # Returns
/// A result of the color setting operation
fn set_windows_11_caption_color(
    window_handle: HWND,
    color_mode: &ColorScheme,
) -> DwallSettingsResult<()> {
    // Predefined color values for dark and light modes
    const DARK_CAPTION_COLOR: u32 = 0x1F1F1F; // Dark gray
    const LIGHT_CAPTION_COLOR: u32 = 0xFAFAFA; // Light gray

    let caption_color = match color_mode {
        ColorScheme::Dark => DARK_CAPTION_COLOR,
        ColorScheme::Light => LIGHT_CAPTION_COLOR,
    };

    debug!(mode = ?color_mode, color=caption_color, "Setting Windows 11+ caption color");

    unsafe {
        DwmSetWindowAttribute(
            window_handle,
            DWMWA_CAPTION_COLOR,
            &caption_color as *const u32 as *const std::ffi::c_void,
            std::mem::size_of::<u32>() as u32,
        )
        .map_err(Into::into)
    }
}

/// Sets dark mode for legacy Windows versions
///
/// # Arguments
/// * `window_handle` - The window handle
/// * `color_mode` - The desired color mode
///
/// # Returns
/// A result of the dark mode setting operation
fn set_legacy_dark_mode(window_handle: HWND, color_mode: &ColorScheme) -> DwallSettingsResult<()> {
    let dark_mode_value: u32 = match color_mode {
        ColorScheme::Dark => 1,
        ColorScheme::Light => 0,
    };

    debug!(mode = ?color_mode, value = dark_mode_value, "Setting legacy dark mode");

    unsafe {
        DwmSetWindowAttribute(
            window_handle,
            DWMWA_USE_IMMERSIVE_DARK_MODE,
            &dark_mode_value as *const _ as *const ffi::c_void,
            std::mem::size_of::<u32>() as u32,
        )
        .map_err(|e| {
            error!(error = %e, "Failed to set legacy dark mode");
            e.into()
        })
    }
}

/// Determines if the current Windows version is Windows 11 or newer
///
/// # Returns
/// A boolean indicating whether the system is Windows 11 or newer
fn is_windows_11_or_newer() -> bool {
    let mut os_version_info = OSVERSIONINFOW {
        dwOSVersionInfoSize: std::mem::size_of::<OSVERSIONINFOW>() as u32,
        ..Default::default()
    };

    let version_check_status = unsafe { RtlGetVersion(&mut os_version_info) };

    if version_check_status != STATUS_SUCCESS {
        warn!(status_code = ?version_check_status, "Failed to retrieve Windows version");
        return false;
    }

    debug!(
        build_number = os_version_info.dwBuildNumber,
        "Windows version detected"
    );

    os_version_info.dwBuildNumber > 22000
}
