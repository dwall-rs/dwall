//! Theme application: restarts the daemon with the new configuration.

use std::time::Duration;

use dwall::Config;
use tokio::time::sleep;

use crate::daemon::{DaemonLauncher, get_last_daemon_error, is_daemon_running, should_run};
use crate::error::{DwallSettingsError, DwallSettingsResult};
use crate::filesystem::validate_theme_roots;
use crate::process::{START_POLL_INTERVAL, START_TIMEOUT, stop_daemon, wait_until};

/// Orchestrates the theme application use case
pub struct ThemeApplier;

impl ThemeApplier {
    /// Apply a theme configuration by restarting the daemon
    pub async fn apply(config: Config) -> DwallSettingsResult<()> {
        validate_theme_roots(&config)?;

        // The configuration is written *before* the daemon is stopped: stopping it
        // first meant a failed write left the user with a dead engine and a change
        // that never reached the disk.
        let config_path = dwall::DWALL_CONFIG_DIR.join("config.toml");
        let writer = dwall::config::ConfigWriter;
        writer.write_to_path(&config_path, &config)?;

        // Stop whichever daemon is running, wherever it was installed from, and wait
        // for it to release its mutex. Failing to stop it is not fatal any more: the
        // daemon re-reads the configuration within a second of it changing.
        // `stop_daemon` waits on the process handle (up to five seconds), so it must
        // run off the async runtime thread that is driving the window.
        let stopped = tauri::async_runtime::spawn_blocking(stop_daemon)
            .await
            .map_err(|e| DwallSettingsError::Other(e.to_string()))?;
        match stopped {
            Ok(Some(pid)) => info!(pid = pid, "Stopped the existing daemon"),
            Ok(None) => info!("No daemon process found"),
            Err(e) => error!(error = %e, "Failed to stop the daemon process"),
        }

        // Random mode has no monitor map but still needs the engine running.
        if !should_run(&config) {
            return Ok(());
        }

        if let Err(e) = DaemonLauncher::launch() {
            sleep(Duration::from_millis(100)).await;

            if let Some(log_error) = get_last_daemon_error(&dwall::DWALL_LOG_DIR) {
                return Err(DwallSettingsError::Daemon(log_error));
            }

            return Err(e);
        }

        // The daemon claims its mutex before doing any work, so this is the startup
        // handshake rather than the fixed sleep it used to be — which failed on a
        // slow start and reported it as "the daemon did not start".
        let started = tauri::async_runtime::spawn_blocking(|| {
            wait_until(START_TIMEOUT, START_POLL_INTERVAL, || {
                is_daemon_running().unwrap_or(false)
            })
        })
        .await
        .map_err(|e| DwallSettingsError::Other(e.to_string()))?;

        if !started {
            if let Some(log_error) = get_last_daemon_error(&dwall::DWALL_LOG_DIR) {
                return Err(DwallSettingsError::Daemon(log_error));
            }

            return Err(DwallSettingsError::Daemon(
                "Daemon process failed to start".to_string(),
            ));
        }

        Ok(())
    }
}
