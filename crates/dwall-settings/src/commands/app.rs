//! Application metadata commands.

use dwall::{DWALL_CACHE_DIR, DWALL_CONFIG_DIR, DWALL_LOG_DIR};

/// Basic application and environment information for the UI.
#[cfg_attr(feature = "typegen", derive(ts_rs::TS))]
#[cfg_attr(feature = "typegen", ts(export))]
#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppInfo {
    pub version: String,
    pub config_dir: String,
    pub log_dir: String,
    pub cache_dir: String,
}

#[tauri::command]
pub fn get_app_info() -> AppInfo {
    AppInfo {
        version: env!("CARGO_PKG_VERSION").to_string(),
        config_dir: DWALL_CONFIG_DIR.display().to_string(),
        log_dir: DWALL_LOG_DIR.display().to_string(),
        cache_dir: DWALL_CACHE_DIR.display().to_string(),
    }
}
