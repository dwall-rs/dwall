//! Window management commands

use dwall::ColorScheme;
use tauri::{AppHandle, Manager, WebviewWindow};

use crate::error::DwallSettingsResult;
use crate::window::set_color_mode;

#[tauri::command]
pub fn show_window(app: AppHandle, label: &str) -> DwallSettingsResult<()> {
    if let Some(window) = app.get_webview_window(label) {
        window.show()?;
        // Windows computes the maximized frame for a *visible* window. One created with
        // `maximized(true)` while hidden carries the state but is shown at its restored
        // size (measured: 1024x640 instead of the 1488x960 work area), so ask once more
        // after showing: `maximize` issues `SW_MAXIMIZE`, which is what sizes the window
        // against the work area with the taskbar on any edge.
        window.maximize()?;
        window.set_focus()?;
    }
    Ok(())
}

/// Opens a directory with the shell.
///
/// Only existing directories are accepted: `open::that` ultimately falls back to
/// `Start-Process`, so an arbitrary path (e.g. an executable) coming from the
/// renderer would otherwise be executed.
#[tauri::command]
pub async fn open_dir(dir_path: std::borrow::Cow<'_, std::path::Path>) -> DwallSettingsResult<()> {
    if !dir_path.is_dir() {
        return Err(crate::error::DwallSettingsError::Other(format!(
            "Not a directory: {}",
            dir_path.display()
        )));
    }

    open::that(dir_path.as_os_str())?;
    Ok(())
}

/// Opens an http(s) URL in the default browser.
///
/// Anything else (local paths, `file:`, custom protocols, `ms-settings:` …) is
/// rejected so the renderer cannot turn this into arbitrary execution.
#[tauri::command]
pub async fn open_url(url: String) -> DwallSettingsResult<()> {
    if !(url.starts_with("https://") || url.starts_with("http://")) {
        return Err(crate::error::DwallSettingsError::Other(format!(
            "Refusing to open non-http(s) URL: {url}"
        )));
    }

    open::that(url)?;
    Ok(())
}

#[tauri::command]
pub async fn open_config_dir() -> DwallSettingsResult<()> {
    open::that(dwall::DWALL_CONFIG_DIR.as_os_str())?;
    Ok(())
}

#[tauri::command]
pub async fn open_log_dir() -> DwallSettingsResult<()> {
    open::that(dwall::DWALL_LOG_DIR.as_os_str())?;
    Ok(())
}

/// Paints the window's title bar for `color_mode`.
///
/// `follow_windows_theme` hands the caption back to Windows instead of matching Dwall's
/// background; see [`set_color_mode`](crate::window::set_color_mode) for what each Windows
/// version supports.
#[tauri::command]
pub async fn set_titlebar_color_mode(
    window: WebviewWindow,
    color_mode: ColorScheme,
    follow_windows_theme: bool,
) -> DwallSettingsResult<()> {
    let hwnd = window.hwnd()?;
    set_color_mode(hwnd, color_mode, follow_windows_theme)
}
