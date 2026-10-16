//! System commands (auto-start, updates, location, process management)

use tauri::{Manager, Url};
use tauri_plugin_updater::UpdaterExt;

use crate::autostart::AutoStartProvider;
use crate::error::{DwallSettingsError, DwallSettingsResult};
use crate::filesystem::{move_directory, validate_directory};
use crate::network::{HttpClient, MirrorUrlResolver};
use crate::process::{find_daemon_process, terminate_process};
use crate::update;

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
pub async fn request_location_permission() -> DwallSettingsResult<()> {
    // `RequestAccessAsync` can show the system location prompt, so keep it off the
    // main thread.
    tauri::async_runtime::spawn_blocking(|| -> DwallSettingsResult<()> {
        use dwall::solar::PositionProvider;

        dwall::platform::Positioner::new()
            .check_location_permission()
            .map_err(Into::into)
    })
    .await
    .map_err(|e| crate::error::DwallSettingsError::Other(e.to_string()))?
}

#[tauri::command]
pub async fn move_directory_cmd(
    source: std::path::PathBuf,
    destination: std::path::PathBuf,
) -> DwallSettingsResult<()> {
    validate_directory(&source, "source")?;
    validate_directory(&destination, "destination")?;
    move_directory(source, destination)
        .await
        .map_err(Into::into)
}

#[tauri::command]
pub async fn kill_daemon_cmd() -> DwallSettingsResult<Option<u32>> {
    tauri::async_runtime::spawn_blocking(move || -> DwallSettingsResult<Option<u32>> {
        match find_daemon_process()? {
            Some(pid) => {
                terminate_process(pid)?;
                Ok(Some(pid))
            }
            None => Ok(None),
        }
    })
    .await
    .map_err(|e| crate::error::DwallSettingsError::Other(e.to_string()))?
}

#[tauri::command]
pub async fn open_privacy_location_settings() -> DwallSettingsResult<()> {
    open::that("ms-settings:privacy-location")?;
    Ok(())
}

#[cfg_attr(feature = "typegen", derive(ts_rs::TS))]
#[cfg_attr(feature = "typegen", ts(export))]
#[derive(serde::Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Metadata {
    #[cfg_attr(feature = "typegen", ts(type = "number"))]
    rid: tauri::ResourceId,
    current_version: String,
    version: String,
    body: Option<String>,
}

/// Wraps a URL parse failure, naming what could not be parsed.
fn invalid_url(what: &str, error: &impl std::fmt::Display) -> DwallSettingsError {
    DwallSettingsError::Other(format!("invalid {what} URL: {error}"))
}

#[tauri::command]
pub async fn check_for_updates_cmd<R: tauri::Runtime>(
    webview: tauri::Webview<R>,
    http_client: tauri::State<'_, HttpClient>,
    network: Option<dwall::config::Network>,
) -> DwallSettingsResult<Option<Metadata>> {
    debug!(network = ?network, "Checking for updates");

    // GitHub keeps pre-releases out of `releases/latest`, so a stable build reads that
    // manifest and can only be offered a stable version. A pre-release build compares
    // against the release list instead, which lets it move to a newer pre-release or
    // back onto a released version (see `crate::update`).
    let current_version = webview.package_info().version.clone();
    let endpoint = if current_version.pre.is_empty() {
        update::LATEST_ENDPOINT.to_string()
    } else {
        match update::prerelease_endpoint(&http_client, &current_version).await? {
            Some(endpoint) => endpoint,
            None => return Ok(None),
        }
    };

    let url = MirrorUrlResolver::resolve(network.as_ref(), &endpoint).await;

    // A mirror template the user typed is the only way these can be invalid, so the
    // message keeps the parse error instead of flattening it to "invalid URL".
    let endpoint = Url::parse(&url)
        .inspect_err(|&e| error!(error = ?e, "Failed to parse endpoint URL"))
        .map_err(|e| invalid_url("endpoint", &e))?;

    let mut builder = webview.updater_builder().endpoints(vec![endpoint])?;

    if let Some(dwall::config::Network::Socks5 { host, port }) = &network {
        let proxy = Url::parse(&format!("socks5h://{host}:{port}"))
            .inspect_err(|&e| error!(error = ?e, "Failed to parse proxy URL"))
            .map_err(|e| invalid_url("proxy", &e))?;
        builder = builder.proxy(proxy);
    }

    match builder.build()?.check().await.inspect_err(|e| {
        error!(error = ?e, "Failed to check update");
    })? {
        None => Ok(None),
        Some(mut update) => {
            let download_url =
                MirrorUrlResolver::resolve(network.as_ref(), update.download_url.as_str()).await;

            update.download_url = download_url
                .parse()
                .inspect_err(|e| error!(error = ?e, "Failed to parse download URL"))
                .map_err(|e| invalid_url("download", &e))?;

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
