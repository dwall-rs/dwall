//! Application commands module
//!
//! This module contains Tauri command handlers that coordinate between the frontend and backend.

use std::{
    borrow::Cow,
    collections::HashMap,
    path::{Path, PathBuf},
};

use dwall::{
    ColorScheme, DWALL_CONFIG_DIR, DWALL_LOG_DIR, DisplayMonitor, config::Config,
    domain::geography::check_location_permission, read_config_file as dwall_read_config,
    write_config_file as dwall_write_config,
};
use tauri::{AppHandle, Manager, WebviewWindow};

use crate::{
    domain::{monitor::get_monitors, theme::validate_solar_theme},
    error::{DwallSettingsError, DwallSettingsResult},
    i18n::get_translations,
    infrastructure::{
        filesystem::move_themes_directory, network::download::ThemeDownloader,
        process::kill_daemon, registry::AutoStartManager, window::set_window_color_mode,
    },
    services::{
        cache::{clear_thumbnail_cache, get_or_save_cached_thumbnails},
        download_service::download_theme_and_extract,
        theme_service::{apply_theme, get_applied_theme_id},
    },
};

#[tauri::command]
pub fn show_window(app: AppHandle, label: &str) -> DwallSettingsResult<()> {
    if let Some(window) = app.get_webview_window(label) {
        window.show()?;
        window.set_focus()?;
    }

    Ok(())
}

#[tauri::command]
pub async fn read_config_file() -> DwallSettingsResult<Config> {
    dwall_read_config().map_err(Into::into)
}

#[tauri::command]
pub async fn write_config_file(config: Config) -> DwallSettingsResult<()> {
    dwall_write_config(&config).map_err(Into::into)
}

#[tauri::command]
pub async fn open_dir(dir_path: Cow<'_, Path>) -> DwallSettingsResult<()> {
    open::that(dir_path.as_os_str()).map_err(|e| e.into())
}

#[tauri::command]
pub async fn open_config_dir() -> DwallSettingsResult<()> {
    open::that(DWALL_CONFIG_DIR.as_os_str()).map_err(|e| e.into())
}

#[tauri::command]
pub async fn open_log_dir() -> DwallSettingsResult<()> {
    open::that(DWALL_LOG_DIR.as_os_str()).map_err(|e| e.into())
}

#[tauri::command]
pub async fn set_titlebar_color_mode(
    window: WebviewWindow,
    color_mode: ColorScheme,
) -> DwallSettingsResult<()> {
    let hwnd = window.hwnd()?;

    set_window_color_mode(hwnd, color_mode)?;
    Ok(())
}

#[tauri::command]
pub async fn get_monitors_cmd() -> DwallSettingsResult<HashMap<String, DisplayMonitor>> {
    get_monitors().map_err(Into::into)
}

#[tauri::command]
pub async fn open_privacy_location_settings() -> DwallSettingsResult<()> {
    open::that("ms-settings:privacy-location").map_err(Into::into)
}

#[tauri::command]
pub async fn validate_theme_cmd(
    themes_directory: &Path,
    theme_id: &str,
) -> DwallSettingsResult<()> {
    validate_solar_theme(themes_directory, theme_id).map_err(Into::into)
}

#[tauri::command]
pub async fn get_applied_theme_id_cmd(monitor_id: &str) -> DwallSettingsResult<Option<String>> {
    get_applied_theme_id(monitor_id)
}

#[tauri::command]
pub async fn apply_theme_cmd(config: Config) -> DwallSettingsResult<()> {
    apply_theme(config).await
}

#[tauri::command]
pub fn check_auto_start() -> DwallSettingsResult<bool> {
    AutoStartManager::check_auto_start()
}

#[tauri::command]
pub fn disable_auto_start() -> DwallSettingsResult<()> {
    AutoStartManager::disable_auto_start()
}

#[tauri::command]
pub fn enable_auto_start() -> DwallSettingsResult<()> {
    AutoStartManager::enable_auto_start()
}

#[tauri::command]
pub async fn download_theme_cmd<R: tauri::Runtime>(
    window: tauri::WebviewWindow<R>,
    downloader: tauri::State<'_, ThemeDownloader>,
    config: Config,
    theme_id: &str,
) -> DwallSettingsResult<()> {
    download_theme_and_extract(window, downloader, config, theme_id).await
}

#[tauri::command]
pub async fn cancel_theme_download_cmd(
    downloader: tauri::State<'_, ThemeDownloader>,
    theme_id: String,
) -> DwallSettingsResult<()> {
    downloader.cancel_theme_download(&theme_id).await;
    Ok(())
}

#[tauri::command]
pub fn request_location_permission() -> DwallSettingsResult<()> {
    check_location_permission().map_err(Into::into)
}

#[tauri::command]
pub async fn move_themes_directory_cmd(
    config: Config,
    dir_path: PathBuf,
) -> DwallSettingsResult<()> {
    move_themes_directory(config, dir_path).await
}

#[tauri::command]
pub fn kill_daemon_cmd() -> DwallSettingsResult<()> {
    kill_daemon()
}

#[tauri::command]
pub async fn get_or_save_cached_thumbnails_cmd(
    theme_id: &str,
    serial_number: u8,
    url: &str,
) -> DwallSettingsResult<PathBuf> {
    get_or_save_cached_thumbnails(theme_id, serial_number, url).await
}

#[tauri::command]
pub async fn clear_thumbnail_cache_cmd() -> DwallSettingsResult<u64> {
    clear_thumbnail_cache().await
}

#[tauri::command]
pub fn get_translations_cmd() -> DwallSettingsResult<serde_json::Value> {
    let translations = get_translations();
    serde_json::to_value(translations)
        .map_err(|_| DwallSettingsError::Other("Failed to serialize translations".to_string()))
}
