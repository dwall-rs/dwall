//! System commands (auto-start, updates, location, process management)

use tauri::{Manager, Url};
use tauri_plugin_updater::UpdaterExt;

use crate::domain::config::MirrorUrlResolver;
use crate::error::DwallSettingsResult;
use crate::infrastructure::filesystem::directory::move_directory;
use crate::infrastructure::process::finder::find_process_by_path;
use crate::infrastructure::process::lifecycle::terminate_process;
use crate::infrastructure::registry::auto_start::AutoStartProvider;

#[tauri::command]
pub fn check_auto_start() -> DwallSettingsResult<bool> {
    AutoStartProvider::is_enabled()
}

#[tauri::command]
pub fn disable_auto_start() -> DwallSettingsResult<()> {
    AutoStartProvider::disable()
}

#[tauri::command]
pub fn enable_auto_start() -> DwallSettingsResult<()> {
    AutoStartProvider::enable()
}

#[tauri::command]
pub fn request_location_permission() -> DwallSettingsResult<()> {
    use dwall::domain::geography::PositionProvider;
    dwall::infrastructure::platform::windows::geolocation::Positioner::new()
        .check_location_permission()
        .map_err(Into::into)
}

#[tauri::command]
pub async fn move_directory_cmd(
    source: std::path::PathBuf,
    destination: std::path::PathBuf,
) -> DwallSettingsResult<()> {
    move_directory(source, destination)
        .await
        .map_err(Into::into)
}

#[tauri::command]
pub fn kill_daemon_cmd() -> DwallSettingsResult<Option<u32>> {
    let daemon_path = crate::DAEMON_EXE_PATH.get().unwrap();
    match find_process_by_path(daemon_path)? {
        Some(pid) => {
            terminate_process(pid)?;
            Ok(Some(pid))
        }
        None => Ok(None),
    }
}

#[tauri::command]
pub async fn open_privacy_location_settings() -> DwallSettingsResult<()> {
    open::that("ms-settings:privacy-location")?;
    Ok(())
}

#[derive(serde::Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Metadata {
    rid: tauri::ResourceId,
    current_version: String,
    version: String,
    body: Option<String>,
}

#[tauri::command]
pub async fn check_for_updates_cmd<R: tauri::Runtime>(
    webview: tauri::Webview<R>,
    network: Option<dwall::config::Network>,
) -> DwallSettingsResult<Option<Metadata>, tauri_plugin_updater::Error> {
    debug!(network = ?network, "Checking for updates");

    let url = MirrorUrlResolver::resolve(
        network.as_ref(),
        "https://github.com/dwall-rs/dwall/releases/latest/download/latest.json",
    )
    .await;

    let endpoint = Url::parse(&url).inspect_err(|&e| {
        error!(error = ?e, "Failed to parse endpoint URL");
    })?;

    let mut builder = webview.updater_builder().endpoints(vec![endpoint])?;

    if let Some(dwall::config::Network::Socks5 { host, port }) = &network {
        let proxy = Url::parse(&format!("socks5h://{host}:{port}")).inspect_err(|&e| {
            error!(error = ?e, "Failed to parse proxy URL");
        })?;
        builder = builder.proxy(proxy);
    }

    match builder.build()?.check().await.inspect_err(|e| {
        error!(error = ?e, "Failed to check update");
    })? {
        None => Ok(None),
        Some(mut update) => {
            let download_url =
                MirrorUrlResolver::resolve(network.as_ref(), update.download_url.as_str()).await;

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
