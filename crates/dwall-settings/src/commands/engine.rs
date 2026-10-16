//! Engine (daemon process) lifecycle commands.

use crate::daemon::{DaemonLauncher, daemon_path, is_daemon_running};
use crate::error::DwallSettingsResult;
use crate::process::{find_daemon_process, terminate_process};

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
/// The daemon is located by executable name, so this also stops a portable or
/// headless copy that was started outside this settings app.
#[tauri::command]
pub async fn stop_engine() -> DwallSettingsResult<bool> {
    tauri::async_runtime::spawn_blocking(move || -> DwallSettingsResult<bool> {
        match find_daemon_process()? {
            Some(pid) => {
                terminate_process(pid)?;
                Ok(true)
            }
            None => Ok(false),
        }
    })
    .await
    .map_err(|e| crate::error::DwallSettingsError::Other(e.to_string()))?
}
