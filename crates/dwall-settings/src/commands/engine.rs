//! Engine (daemon process) lifecycle commands.

use crate::daemon::{DaemonLauncher, daemon_path, is_daemon_running};
use crate::error::DwallSettingsResult;
use crate::process::stop_daemon;

/// Whether the daemon process is currently running.
///
/// Runs off the main thread: the underlying check is a single `OpenMutex`, but the
/// command still must not touch the window's event loop.
#[tauri::command]
pub async fn get_engine_status() -> DwallSettingsResult<bool> {
    tauri::async_runtime::spawn_blocking(is_daemon_running)
        .await
        .map_err(|e| crate::error::DwallSettingsError::Other(e.to_string()))?
}

/// The daemon's most recent error line, when it has one.
///
/// A running daemon no longer means a working one: a failed cycle is retried with a
/// backoff instead of ending the process, so the status area needs the reason to be
/// able to say anything useful.
#[tauri::command]
pub async fn get_daemon_last_error() -> DwallSettingsResult<Option<String>> {
    tauri::async_runtime::spawn_blocking(|| {
        crate::daemon::get_last_daemon_error(&dwall::DWALL_LOG_DIR)
    })
    .await
    .map_err(|e| crate::error::DwallSettingsError::Other(e.to_string()))
}

/// Start the daemon process
#[tauri::command]
pub async fn start_engine() -> DwallSettingsResult<()> {
    daemon_path()?;
    tauri::async_runtime::spawn_blocking(DaemonLauncher::launch)
        .await
        .map_err(|e| crate::error::DwallSettingsError::Other(e.to_string()))?
}

/// Stop the daemon process. Returns whether a process was actually terminated.
///
/// The daemon is located by executable name and session, so this also stops a
/// portable or headless copy that was started outside this settings app — and it
/// waits for the process to really exit, because its mutex is what the next start
/// has to claim.
#[tauri::command]
pub async fn stop_engine() -> DwallSettingsResult<bool> {
    tauri::async_runtime::spawn_blocking(|| -> DwallSettingsResult<bool> {
        Ok(stop_daemon()?.is_some())
    })
    .await
    .map_err(|e| crate::error::DwallSettingsError::Other(e.to_string()))?
}
