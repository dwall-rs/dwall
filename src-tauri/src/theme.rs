use std::{
    fs::File,
    io::{BufReader, Read},
    os::windows::process::CommandExt,
    path::Path,
    process::Command,
    time::Duration,
};

use dwall::{
    config::{write_config_file as dwall_write_config, Config},
    ThemeValidator,
};
use serde::Deserialize;
use serde_json::Value;
use tokio::time::sleep;
use windows::Win32::System::Threading::CREATE_NO_WINDOW;

use crate::{
    error::{DwallSettingsError, DwallSettingsResult},
    process_manager::{find_daemon_process, kill_daemon},
    DAEMON_EXE_PATH, DWALL_CONFIG_DIR,
};

#[derive(Deserialize)]
struct DaemonLogEntry {
    fields: Value,
}

/// Attempts to read the most recent error from the daemon log file
///
/// Returns the error message if found, or None if no error was found or the log file couldn't be read
pub fn get_last_daemon_error() -> Option<String> {
    let log_file_path = DWALL_CONFIG_DIR.join("dwall.log");
    if !log_file_path.exists() {
        debug!("Daemon log file not found");
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

    // Get lines and search from newest to oldest
    let lines: Vec<&str> = content.lines().collect();

    for line in lines.iter().rev() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        if line.to_lowercase().contains("error") {
            match serde_json::from_str::<DaemonLogEntry>(line) {
                Ok(log_line) => {
                    return Some(log_line.fields.to_string());
                }
                Err(_) => {
                    // Just continue searching if this line couldn't be parsed
                    continue;
                }
            }
        }
    }

    None
}

/// Launches the daemon process to apply wallpaper settings
///
/// Returns an error if the daemon couldn't be started
pub fn launch_daemon() -> DwallSettingsResult<()> {
    let daemon_path = match DAEMON_EXE_PATH.get() {
        Some(path) => path.as_os_str(),
        None => {
            error!("DAEMON_EXE_PATH not configured");
            return Err(DwallSettingsError::Daemon(
                "Daemon executable path not configured".into(),
            ));
        }
    };

    let mut cmd = Command::new(daemon_path);

    // In debug mode, inherit stdio for easier debugging
    if cfg!(debug_assertions) {
        use std::process::Stdio;

        cmd.creation_flags(CREATE_NO_WINDOW.0)
            .stderr(Stdio::inherit())
            .stdout(Stdio::inherit());
    } else {
        cmd.creation_flags(CREATE_NO_WINDOW.0);
    }

    match cmd.spawn() {
        Ok(process) => {
            info!(pid = process.id(), "Daemon process started");
            Ok(())
        }
        Err(e) => {
            error!(error = ?e, path = ?daemon_path, "Failed to start daemon process");
            Err(e.into())
        }
    }
}

#[tauri::command]
pub async fn validate_theme(themes_direcotry: &Path, theme_id: &str) -> DwallSettingsResult<()> {
    trace!(id = theme_id, "Validating theme");
    match ThemeValidator::validate_theme(themes_direcotry, theme_id).await {
        Ok(_) => {
            debug!(id = theme_id, "Theme validation successful");
            Ok(())
        }
        Err(e) => {
            error!(theme_id, error = %e, "Theme validation failed");
            Err(e.into())
        }
    }
}

#[tauri::command]
pub async fn get_applied_theme_id(monitor_id: &str) -> DwallSettingsResult<Option<String>> {
    debug!(monitor_id, "Getting current theme for monitor");

    // Check if daemon is running
    let daemon_process = find_daemon_process()?;
    if daemon_process.is_none() {
        debug!("No active daemon process found");
        return Ok(None);
    }

    // Read current configuration
    match dwall::config::read_config_file().await {
        Ok(config) => {
            let monitor_themes = config.monitor_specific_wallpapers();

            // Handle special case for "all" monitors
            // TODO: `monitor_id == "all"` is deprecated, remove in the future
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

                theme_id
            } else {
                monitor_themes.get(monitor_id)
            };

            debug!(monitor_id, theme_id = ?theme_id, "Retrieved theme ID");
            Ok(theme_id.map(|s| s.to_string()))
        }
        Err(e) => {
            error!(error = %e, "Failed to read configuration");
            Err(e.into())
        }
    }
}

#[tauri::command]
pub async fn apply_theme(config: Config) -> DwallSettingsResult<()> {
    trace!("Starting theme application");

    match kill_daemon() {
        Ok(()) => debug!("Successfully killed existing daemon process"),
        Err(e) => warn!(error = %e, "Failed to kill existing daemon process"),
    }

    dwall_write_config(&config).await?;

    // If no themes are configured, we're done
    if config.monitor_specific_wallpapers().is_empty() {
        debug!("No themes configured, skipping daemon launch");
        return Ok(());
    }

    // Launch daemon to apply the theme
    if let Err(e) = launch_daemon() {
        error!(error =%e, "Failed to launch daemon");

        // Wait briefly for daemon to log any errors
        sleep(Duration::from_millis(100)).await;

        // Check for error logs
        if let Some(e) = get_last_daemon_error() {
            return Err(DwallSettingsError::Daemon(e));
        }

        return Err(e);
    }

    // Wait for daemon to start
    sleep(Duration::from_millis(100)).await;

    // Verify daemon is running
    find_daemon_process()?.ok_or(DwallSettingsError::Daemon(
        "Daemon process failed to start".to_string(),
    ))?;

    info!("Successfully spawned and monitored theme daemon");
    Ok(())
}
