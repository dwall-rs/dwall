//! Monitor commands

use std::collections::HashMap;

use dwall::{DisplayMonitor, DisplayMonitorProvider, MonitorProvider};

use crate::error::DwallSettingsResult;
use crate::monitor_wallpapers::{MonitorThemeChange, MonitorThemes, resolve};

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

/// The effective theme per monitor, optionally turning an apply/stop into the mode to
/// write. Infallible: it only maps the values the renderer already holds.
#[tauri::command]
pub fn resolve_monitor_themes_cmd(
    mode: dwall::config::WallpaperMode,
    monitors: Vec<String>,
    change: Option<MonitorThemeChange>,
) -> MonitorThemes {
    resolve(&mode, &monitors, change.as_ref())
}
