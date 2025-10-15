//! Theme service module
//!
//! This module contains the application service logic for theme management.

use std::{os::windows::process::CommandExt, process::Command, time::Duration};

use dwall::{
    read_config_file as dwall_read_config, write_config_file as dwall_write_config, Config,
    DWALL_CONFIG_DIR,
};
use tokio::time::sleep;
use windows::Win32::System::Threading::CREATE_NO_WINDOW;

use crate::{
    domain::theme::get_last_daemon_error,
    error::{DwallSettingsError, DwallSettingsResult},
    infrastructure::process::{find_daemon_process, kill_daemon},
    DAEMON_EXE_PATH,
};

/// Launches the daemon process to apply wallpaper settings
///
/// Returns an error if the daemon couldn't be started
pub fn launch_daemon() -> DwallSettingsResult<()> {
    let daemon_path = match DAEMON_EXE_PATH.get() {
        Some(path) => path.as_os_str(),
        None => {
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
        Ok(_process) => Ok(()),
        Err(e) => Err(e.into()),
    }
}

/// Gets the currently applied theme ID for a monitor
pub async fn get_applied_theme_id(monitor_id: &str) -> DwallSettingsResult<Option<String>> {
    // Check if daemon is running
    let daemon_process = find_daemon_process()?;
    if daemon_process.is_none() {
        return Ok(None);
    }

    // Read current configuration
    match dwall_read_config().await {
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

            Ok(theme_id.map(|s| s.to_string()))
        }
        Err(e) => Err(e.into()),
    }
}

/// Applies a theme configuration
pub async fn apply_theme(config: Config) -> DwallSettingsResult<()> {
    match kill_daemon() {
        Ok(()) => {}
        Err(_e) => {}
    }

    dwall_write_config(&config).await?;

    // If no themes are configured, we're done
    if config.monitor_specific_wallpapers().is_empty() {
        return Ok(());
    }

    // Launch daemon to apply the theme
    if let Err(e) = launch_daemon() {
        // Wait briefly for daemon to log any errors
        sleep(Duration::from_millis(100)).await;

        // Check for error logs
        if let Some(e) = get_last_daemon_error(&DWALL_CONFIG_DIR) {
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

    Ok(())
}
