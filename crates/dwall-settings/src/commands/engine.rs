//! Engine (daemon process) lifecycle commands.

use dwall::config::{Config, ConfigReader, ConfigWriter, WallpaperMode};

use crate::daemon::{DaemonLauncher, daemon_path, is_daemon_running};
use crate::error::DwallSettingsResult;
use crate::process::stop_daemon;

/// Whether the daemon process is currently running.
///
/// Runs off the main thread: the underlying check is a single `OpenMutex`, but the
/// command still must not touch the window's event loop.
#[tauri::command]
pub async fn get_engine_status() -> DwallSettingsResult<bool> {
    tauri::async_runtime::spawn_blocking(is_daemon_running)
        .await
        .map_err(|e| crate::error::DwallSettingsError::Other(e.to_string()))?
}

/// The daemon's most recent error line, when it has one.
///
/// A running daemon no longer means a working one: a failed cycle is retried with a
/// backoff instead of ending the process, so the status area needs the reason to be
/// able to say anything useful.
#[tauri::command]
pub async fn get_daemon_last_error() -> DwallSettingsResult<Option<String>> {
    tauri::async_runtime::spawn_blocking(|| {
        crate::daemon::get_last_daemon_error(&dwall::DWALL_LOG_DIR)
    })
    .await
    .map_err(|e| crate::error::DwallSettingsError::Other(e.to_string()))
}

/// Start the daemon process
#[tauri::command]
pub async fn start_engine() -> DwallSettingsResult<()> {
    daemon_path()?;
    tauri::async_runtime::spawn_blocking(DaemonLauncher::launch)
        .await
        .map_err(|e| crate::error::DwallSettingsError::Other(e.to_string()))?
}

/// Clears the theme selection that makes the settings window start the daemon.
///
/// "Stop engine" is the user saying they do not want a wallpaper applied. Without
/// this, opening the settings window again saw the old random pool or monitor
/// mapping, called `should_run` true and started the daemon back up.
fn clear_theme_mode(config: &mut Config) {
    config.set_wallpaper_mode(WallpaperMode::default());
}

fn clear_theme_config() -> DwallSettingsResult<()> {
    let path = dwall::config::config_path();
    if !path.exists() {
        return Ok(());
    }

    let mut config = match ConfigReader::read_from_path(&path) {
        Ok(config) => config,
        Err(e) => {
            // A corrupt file already makes `setup_app` fall back to defaults, which
            // do not ask for a daemon; leave the user's file untouched.
            warn!(error = %e, "Could not read the configuration to clear the theme");
            return Ok(());
        }
    };

    if matches!(
        config.wallpaper_mode(),
        WallpaperMode::Fixed {
            monitor_specific_wallpapers
        } if monitor_specific_wallpapers.is_empty()
    ) {
        return Ok(());
    }

    clear_theme_mode(&mut config);
    ConfigWriter.write_to_path(&path, &config)?;
    info!("Cleared the configured theme before stopping the engine");
    Ok(())
}

/// Stop the daemon process. Returns whether a process was actually terminated.
///
/// The daemon is located by executable name and session, so this also stops a
/// portable or headless copy that was started outside this settings app — and it
/// waits for the process to really exit, because its mutex is what the next start
/// has to claim. The theme selection is cleared first, so reopening the settings
/// window does not start the daemon again.
#[tauri::command]
pub async fn stop_engine() -> DwallSettingsResult<bool> {
    tauri::async_runtime::spawn_blocking(|| -> DwallSettingsResult<bool> {
        clear_theme_config()?;
        Ok(stop_daemon()?.is_some())
    })
    .await
    .map_err(|e| crate::error::DwallSettingsError::Other(e.to_string()))?
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clearing_the_theme_mode_makes_the_engine_unneeded() {
        let mut config = Config::default();
        config.set_wallpaper_mode(WallpaperMode::Random { pool: None });
        assert!(crate::daemon::should_run(&config));

        clear_theme_mode(&mut config);

        assert!(!crate::daemon::should_run(&config));
    }
}
