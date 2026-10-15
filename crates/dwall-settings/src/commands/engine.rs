//! Engine (daemon process) lifecycle commands.

use crate::daemon::{DaemonLauncher, daemon_path};
use crate::error::DwallSettingsResult;
use crate::process::{find_process_by_path, terminate_process};

/// Whether the daemon process is currently running.
///
/// Runs off the main thread: a synchronous command would block the window while
/// the whole process list is enumerated.
#[tauri::command]
pub async fn get_engine_status() -> DwallSettingsResult<bool> {
    let path = daemon_path()?;
    Ok(
        tauri::async_runtime::spawn_blocking(move || find_process_by_path(path))
            .await
            .map_err(|e| crate::error::DwallSettingsError::Other(e.to_string()))??
            .is_some(),
    )
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
#[tauri::command]
pub async fn stop_engine() -> DwallSettingsResult<bool> {
    let path = daemon_path()?;
    tauri::async_runtime::spawn_blocking(move || -> DwallSettingsResult<bool> {
        match find_process_by_path(path)? {
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
