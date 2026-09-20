//! Monitor commands

use std::collections::HashMap;

use dwall::{DisplayMonitor, DisplayMonitorProvider, MonitorProvider};

use crate::error::DwallSettingsResult;

#[tauri::command]
pub async fn get_monitors_cmd() -> DwallSettingsResult<HashMap<String, DisplayMonitor>> {
    DisplayMonitorProvider::new()
        .refresh_monitors()
        .map_err(Into::into)
}
