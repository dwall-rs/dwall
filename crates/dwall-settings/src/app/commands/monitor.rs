//! Monitor commands

use std::collections::HashMap;

use dwall::DisplayMonitor;

use crate::domain::monitor::MonitorProvider;
use crate::error::DwallSettingsResult;

#[tauri::command]
pub async fn get_monitors_cmd() -> DwallSettingsResult<HashMap<String, DisplayMonitor>> {
    MonitorProvider::get_monitors().map_err(Into::into)
}
