use std::error::Error;

use tauri::{WebviewUrl, WebviewWindowBuilder};
use windows::Win32::{
    Foundation::HWND,
    Graphics::Dwm::{DwmSetWindowAttribute, DWMWA_CAPTION_COLOR},
};

use crate::error::DwallSettingsResult;

pub fn new_main_window(app: &tauri::AppHandle) -> Result<(), Box<dyn Error>> {
    trace!("Entering new_main_window function");

    trace!(
        "Creating window with parameters: title='Dwall Settings', transparent=false, resizable=false, maximizable=false, visible=false, inner_size=(660, 600)"
    );
    let win_builder = WebviewWindowBuilder::new(app, "main", WebviewUrl::default())
        .title("Dwall Settings")
        .resizable(false)
        .maximizable(false)
        .visible(false)
        .inner_size(660., 600.);

    trace!("WebviewWindowBuilder configured");

    let window = win_builder.build().map_err(|e| {
        error!("Failed to build main window: {}", e);
        e
    })?;

    trace!("Main window successfully built");

    let raw_handle = window.hwnd().map_err(|e| {
        error!("Failed to get window handle: {}", e);
        e
    })?;

    trace!("Retrieved window handle: {:?}", raw_handle);

    set_titlebar_color(raw_handle, 0xFAFAFA).map_err(|e| {
        error!("Failed to set titlebar color: {}", e);
        e
    })?;

    trace!("Exiting new_main_window function");
    Ok(())
}

/// Set the titlebar color for a given window handle
///
/// # Arguments
/// * `hwnd` - The window handle to set the titlebar color for
/// * `color` - The color to set, in BGR format (0xBBGGRR)
///
/// # Notes
/// - Color is in COLORREF format (0xBBGGRR)
/// - Transparent colors cannot be set
/// - Errors in setting the color are logged but do not halt execution
pub fn set_titlebar_color(hwnd: HWND, color: u32) -> DwallSettingsResult<()> {
    trace!("Entering set_titlebar_color function");
    trace!(
        "Setting titlebar color for HWND {:?} to 0x{:X}",
        hwnd,
        color
    );
    let result = unsafe {
        DwmSetWindowAttribute(
            hwnd,
            DWMWA_CAPTION_COLOR,
            &color as *const u32 as *const std::ffi::c_void,
            std::mem::size_of::<u32>() as u32,
        )
    };

    match result {
        Ok(_) => {
            trace!("Titlebar color set successfully");
            Ok(())
        }
        Err(e) => {
            error!("Failed to set titlebar color: {}", e);
            trace!("Titlebar color setting error details: {:?}", e);
            Err(e.into())
        }
    }
}
