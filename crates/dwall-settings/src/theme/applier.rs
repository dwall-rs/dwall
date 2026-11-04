//! Theme application: restarts the daemon with the new configuration.

use dwall::Config;

use crate::daemon::{should_run, start_and_wait};
use crate::error::{DwallSettingsError, DwallSettingsResult};
use crate::filesystem::validate_theme_roots;
use crate::process::stop_daemon;

/// Orchestrates the theme application use case
pub struct ThemeApplier;

impl ThemeApplier {
    /// Apply a theme configuration by restarting the daemon
    pub async fn apply(config: Config) -> DwallSettingsResult<()> {
        // The renderer can send any interval or coordinate pair; writing an invalid
        // one used to make the daemon fail to read its own configuration after the
        // restart that this command performs.
        config.validate()?;
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

        // Launch and handshake in one blocking call: the daemon counts as started only
        // once it owns its mutex, and waiting for that must not block the window.
        tauri::async_runtime::spawn_blocking(start_and_wait)
            .await
            .map_err(|e| DwallSettingsError::Other(e.to_string()))?
    }
}
