//! Application commands module
//!
//! This module contains Tauri command handlers that coordinate between the frontend and backend.

use std::{
    borrow::Cow,
    collections::HashMap,
    path::{Path, PathBuf},
};

use dwall::{
    ColorScheme, DWALL_CONFIG_DIR, DWALL_LOG_DIR, DisplayMonitor, config::Network,
    domain::geography::check_location_permission, read_config_file as dwall_read_config,
    write_config_file as dwall_write_config,
};
use serde::Serialize;
use tauri::{AppHandle, Manager, ResourceId, Runtime, Url, Webview, WebviewWindow};
use tauri_plugin_updater::UpdaterExt;

use crate::{
    domain::{monitor::get_monitors, settings::Config, theme::validate_solar_theme},
    error::DwallSettingsResult,
    infrastructure::{
        filesystem::move_themes_directory, network::download::ThemeDownloader,
        process::kill_daemon, registry::AutoStartManager, window::set_window_color_mode,
    },
    services::{
        cache::{ThumbnailCache, clear_thumbnail_cache, get_or_save_cached_thumbnails},
        download_service::download_theme_and_extract,
        theme_service::{apply_theme, get_applied_theme_id},
    },
    utils::helpers::resolve_github_mirror_url,
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
pub async fn read_config_file() -> DwallSettingsResult<dwall::Config> {
    dwall_read_config().map_err(Into::into)
}

#[tauri::command]
pub async fn write_config_file(config: dwall::Config) -> DwallSettingsResult<()> {
    debug!(config = ?config, "Writing config file");
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
pub async fn apply_theme_cmd(config: dwall::Config) -> DwallSettingsResult<()> {
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
    config: dwall::Config,
    theme_id: &str,
) -> DwallSettingsResult<()> {
    download_theme_and_extract(window, downloader, Config::new(config), theme_id).await
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
    config: dwall::Config,
    dir_path: PathBuf,
) -> DwallSettingsResult<()> {
    move_themes_directory(config, dir_path).await
}

#[tauri::command]
pub fn kill_daemon_cmd() -> DwallSettingsResult<Option<u32>> {
    kill_daemon()
}

#[tauri::command]
pub async fn get_or_save_cached_thumbnails_cmd(
    cache: tauri::State<'_, ThumbnailCache>,
    theme_id: &str,
    serial_number: u8,
    url: &str,
) -> DwallSettingsResult<PathBuf> {
    get_or_save_cached_thumbnails(cache.inner(), theme_id, serial_number, url).await
}

#[tauri::command]
pub async fn clear_thumbnail_cache_cmd() -> DwallSettingsResult<u64> {
    clear_thumbnail_cache().await
}

#[derive(Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Metadata {
    rid: ResourceId,
    current_version: String,
    version: String,
    body: Option<String>,
}

#[tauri::command]
pub async fn check_for_updates_cmd<R: Runtime>(
    webview: Webview<R>,
    network: Option<Network>,
) -> DwallSettingsResult<Option<Metadata>, tauri_plugin_updater::Error> {
    debug!(network = ?network, "Checking for updates");

    let url = resolve_github_mirror_url(
        network.as_ref(),
        "https://github.com/dwall-rs/dwall/releases/latest/download/latest.json",
    )
    .await;
    let endpoint = Url::parse(&url).inspect_err(|&e| {
        error!(error = ?e, "Failed to parse endpoint URL");
    })?;
    debug!("Endpoint URL: {}", endpoint);

    let mut builder = webview.updater_builder().endpoints(vec![endpoint])?;

    if let Some(Network::Socks5 { host, port }) = &network {
        let proxy = Url::parse(&format!("socks5h://{host}:{port}")).inspect_err(|&e| {
            error!(error = ?e, "Failed to parse proxy URL");
        })?;
        debug!("Proxy URL: {}", proxy);
        builder = builder.proxy(proxy);
    }

    match builder.build()?.check().await.inspect_err(|e| {
        error!(error = ?e, "Failed to check update");
    })? {
        None => Ok(None),
        Some(mut update) => {
            let download_url =
                resolve_github_mirror_url(network.as_ref(), update.download_url.as_str()).await;

            update.download_url = download_url.parse().inspect_err(|e| {
                error!(error = ?e, "Failed to parse download URL");
            })?;

            info!(
                version = update.version,
                date = ?update.date,
                url = %update.download_url,
                proxy = update.proxy.as_ref().map(|p| p.as_str()),
                "Update available"
            );
            let metadata = Metadata {
                current_version: update.current_version.clone(),
                version: update.version.clone(),
                body: update.body.clone(),
                rid: webview.resources_table().add(update),
            };

            debug!(rid = metadata.rid, "Update metadata generated");
            Ok(Some(metadata))
        }
    }
}
