//! Window management commands

use dwall::ColorScheme;
use tauri::{AppHandle, Manager, WebviewWindow};

use crate::error::DwallSettingsResult;
use crate::infrastructure::window::color_mode::set_color_mode;

#[tauri::command]
pub fn show_window(app: AppHandle, label: &str) -> DwallSettingsResult<()> {
    if let Some(window) = app.get_webview_window(label) {
        window.show()?;
        window.set_focus()?;
    }
    Ok(())
}

#[tauri::command]
pub async fn open_dir(dir_path: std::borrow::Cow<'_, std::path::Path>) -> DwallSettingsResult<()> {
    open::that(dir_path.as_os_str())?;
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

#[tauri::command]
pub async fn set_titlebar_color_mode(
    window: WebviewWindow,
    color_mode: ColorScheme,
) -> DwallSettingsResult<()> {
    let hwnd = window.hwnd()?;
    set_color_mode(hwnd, color_mode)?;
    Ok(())
}
