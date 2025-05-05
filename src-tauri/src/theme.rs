use std::{
    fs::File,
    io::{BufReader, Read},
    os::windows::process::CommandExt,
    path::Path,
    process::Command,
};

use dwall::{
    config::{write_config_file as dwall_write_config, Config},
    ThemeValidator,
};
use serde::Deserialize;
use serde_json::Value;
use windows::Win32::System::Threading::CREATE_NO_WINDOW;

use crate::{
    error::{DwallSettingsError, DwallSettingsResult},
    process_manager::{find_daemon_process, kill_daemon},
    DAEMON_EXE_PATH, DWALL_CONFIG_DIR,
};

#[derive(Deserialize)]
struct DaemonErrorLog {
    fields: Value,
}

pub fn read_daemon_error_log() -> Option<String> {
    let log_file_path = DWALL_CONFIG_DIR.join("dwall.log");
    if !log_file_path.exists() {
        debug!("Daemon log file does not exist yet");
        return None;
    }

    let file = match File::open(&log_file_path) {
        Ok(file) => file,
        Err(e) => {
            warn!(error = ?e, "Failed to open daemon log file");
            return None;
        }
    };

    let mut content = String::new();
    let mut reader = BufReader::new(file);
    if reader.read_to_string(&mut content).is_err() {
        warn!("Failed to read daemon log file");
        return None;
    }

    // Split by lines and collect into a vector
    let lines: Vec<&str> = content.lines().collect();

    // Search backwards for the first line containing error message
    // Usually the last line is empty, so start from the second last line
    for line in lines.iter().rev() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        if line.to_lowercase().contains("error") {
            match serde_json::from_str::<DaemonErrorLog>(line) {
                Ok(log_line) => {
                    debug!("Found last error log entry");
                    return Some(log_line.fields.to_string());
                }
                Err(e) => {
                    warn!(error = ?e, "Failed to parse log line as JSON");
                }
            }
        }
    }

    debug!("No error logs found in daemon log file");
    None
}

pub fn spawn_apply_daemon() -> DwallSettingsResult<()> {
    let daemon_path = match DAEMON_EXE_PATH.get() {
        Some(path) => path.as_os_str(),
        None => {
            error!("DAEMON_EXE_PATH is not set");
            return Err(DwallSettingsError::Daemon(
                "DAEMON_EXE_PATH is not set".into(),
            ));
        }
    };

    let mut cmd = Command::new(daemon_path);

    if cfg!(debug_assertions) {
        use std::process::Stdio;

        cmd.creation_flags(CREATE_NO_WINDOW.0)
            .stderr(Stdio::inherit())
            .stdout(Stdio::inherit());
    } else {
        cmd.creation_flags(CREATE_NO_WINDOW.0);
    }

    match cmd.spawn() {
        Ok(handle) => {
            info!(pid = handle.id(), "Spawned daemon using subprocess");
            Ok(())
        }
        Err(e) => {
            error!(error = ?e, path = ?daemon_path, "Failed to spawn daemon");
            Err(e.into())
        }
    }
}

#[tauri::command]
pub async fn check_theme_exists(
    themes_direcotry: &Path,
    theme_id: &str,
) -> DwallSettingsResult<()> {
    trace!(id = theme_id, "Checking theme existence for theme");
    match ThemeValidator::validate_theme(themes_direcotry, theme_id).await {
        Ok(_) => {
            info!(id = theme_id, "Theme exists and is valid");
            Ok(())
        }
        Err(e) => {
            error!(theme_id = %theme_id, error = ?e, "Theme validation failed");
            Err(e.into())
        }
    }
}

#[tauri::command]
pub async fn get_applied_theme_id(monitor_id: &str) -> DwallSettingsResult<Option<String>> {
    debug!(monitor_id, "Attempting to get currently applied theme ID");

    let daemon_process = find_daemon_process()?;
    if daemon_process.is_none() {
        debug!("No daemon process found");
        return Ok(None);
    }

    match dwall::config::read_config_file().await {
        Ok(config) => {
            let monitor_themes = config.monitor_specific_wallpapers();
            let theme_id = if monitor_id == "all" {
                let theme_id = match monitor_themes {
                    dwall::config::MonitorSpecificWallpapers::All(theme_id) => Some(theme_id),
                    dwall::config::MonitorSpecificWallpapers::Specific(themes_map) => {
                        let mut iter = themes_map.values();
                        let first_value = iter.next();
                        if iter.all(|value| Some(value) == first_value) {
                            first_value
                        } else {
                            None
                        }
                    }
                };

                info!(theme_id = ?theme_id, "Retrieved all theme ID");
                theme_id
            } else {
                let theme_id = monitor_themes.get(monitor_id);
                info!(monitor_id, theme_id =?theme_id, "Retrieved current theme ID");
                theme_id
            };

            Ok(theme_id.map(|s| s.to_string()))
        }
        Err(e) => {
            error!(error = ?e, "Failed to read config file while getting theme ID");
            Err(e.into())
        }
    }
}

#[tauri::command]
pub async fn apply_theme(config: Config) -> DwallSettingsResult<()> {
    trace!("Starting theme application process");

    match kill_daemon() {
        Ok(()) => debug!("Successfully killed existing daemon process"),
        Err(e) => warn!(error = ?e, "Failed to kill existing daemon process"),
    }

    dwall_write_config(&config).await?;

    if let Err(e) = spawn_apply_daemon() {
        error!(error =?e, "Failed to spawn or monitor theme daemon");

        if let Some(e) = read_daemon_error_log() {
            return Err(DwallSettingsError::Daemon(e));
        }

        return Err(e);
    }
    info!("Successfully spawned and monitored theme daemon");

    Ok(())
}
