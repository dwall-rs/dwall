//! Engine (daemon process) lifecycle commands.

use crate::DAEMON_EXE_PATH;
use crate::daemon::DaemonLauncher;
use crate::error::{DwallSettingsError, DwallSettingsResult};
use crate::process::{find_process_by_path, terminate_process};

fn daemon_path() -> DwallSettingsResult<&'static std::path::PathBuf> {
    DAEMON_EXE_PATH
        .get()
        .ok_or_else(|| DwallSettingsError::Daemon("Daemon executable path not configured".into()))
}

/// Whether the daemon process is currently running
#[tauri::command]
pub fn get_engine_status() -> DwallSettingsResult<bool> {
    Ok(find_process_by_path(daemon_path()?)?.is_some())
}

/// Start the daemon process
#[tauri::command]
pub fn start_engine() -> DwallSettingsResult<()> {
    DaemonLauncher::launch()
}

/// Stop the daemon process. Returns whether a process was actually terminated.
#[tauri::command]
pub fn stop_engine() -> DwallSettingsResult<bool> {
    match find_process_by_path(daemon_path()?)? {
        Some(pid) => {
            terminate_process(pid)?;
            Ok(true)
        }
        None => Ok(false),
    }
}
