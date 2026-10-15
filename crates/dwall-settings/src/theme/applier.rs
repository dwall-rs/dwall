//! Theme application: restarts the daemon with the new configuration.

use std::time::Duration;

use dwall::Config;
use tokio::time::sleep;

use crate::daemon::{DaemonLauncher, daemon_path, get_last_daemon_error, should_run};
use crate::error::{DwallSettingsError, DwallSettingsResult};
use crate::filesystem::validate_directory;
use crate::process::{find_process_by_path, terminate_process};

/// Orchestrates the theme application use case
pub struct ThemeApplier;

impl ThemeApplier {
    /// Apply a theme configuration by restarting the daemon
    pub async fn apply(config: Config) -> DwallSettingsResult<()> {
        validate_directory(config.themes_directory(), "themes_directory")?;
        validate_directory(
            config.customized_themes_directory(),
            "customized_themes_directory",
        )?;

        match find_process_by_path(daemon_path()?) {
            Ok(Some(pid)) => {
                info!(pid = pid, "Stopping existing daemon");
                terminate_process(pid)?;
            }
            Ok(None) => info!("No daemon process found"),
            Err(e) => error!(error = %e, "Failed to find daemon process"),
        }

        let config_path = dwall::DWALL_CONFIG_DIR.join("config.toml");
        let writer = dwall::config::ConfigWriter;
        writer.write_to_path(&config_path, &config)?;

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

        sleep(Duration::from_millis(100)).await;

        find_process_by_path(daemon_path()?)?.ok_or(DwallSettingsError::Daemon(
            "Daemon process failed to start".to_string(),
        ))?;

        Ok(())
    }
}
