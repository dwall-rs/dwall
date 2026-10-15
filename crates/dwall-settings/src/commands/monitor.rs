//! Monitor commands

use std::collections::HashMap;

use dwall::{DisplayMonitor, DisplayMonitorProvider, MonitorProvider};

use crate::error::DwallSettingsResult;

#[tauri::command]
pub async fn get_monitors_cmd() -> DwallSettingsResult<HashMap<String, DisplayMonitor>> {
    // Display enumeration talks to the device installer API; keep it off the main
    // thread.
    tauri::async_runtime::spawn_blocking(
        || -> DwallSettingsResult<HashMap<String, DisplayMonitor>> {
            DisplayMonitorProvider::new()
                .refresh_monitors()
                .map_err(Into::into)
        },
    )
    .await
    .map_err(|e| crate::error::DwallSettingsError::Other(e.to_string()))?
}
