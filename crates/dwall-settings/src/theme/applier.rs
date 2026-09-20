//! Theme application: restarts the daemon with the new configuration.

use std::time::Duration;

use dwall::Config;
use tokio::time::sleep;

use crate::daemon::{DaemonLauncher, get_last_daemon_error};
use crate::error::{DwallSettingsError, DwallSettingsResult};
use crate::process::{find_process_by_path, terminate_process};

/// Orchestrates the theme application use case
pub struct ThemeApplier;

impl ThemeApplier {
    /// Apply a theme configuration by restarting the daemon
    pub async fn apply(config: Config) -> DwallSettingsResult<()> {
        match find_process_by_path(&std::path::PathBuf::from(
            crate::DAEMON_EXE_PATH.get().unwrap().to_str().unwrap(),
        )) {
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

        if config
            .monitor_specific_wallpapers()
            .is_none_or(|w| w.is_empty())
        {
            return Ok(());
        }

        if let Err(e) = DaemonLauncher::launch() {
            sleep(Duration::from_millis(100)).await;

            if let Some(log_error) = get_last_daemon_error(&dwall::DWALL_CONFIG_DIR) {
                return Err(DwallSettingsError::Daemon(log_error));
            }

            return Err(e);
        }

        sleep(Duration::from_millis(100)).await;

        find_process_by_path(&std::path::PathBuf::from(
            crate::DAEMON_EXE_PATH.get().unwrap().to_str().unwrap(),
        ))?
        .ok_or(DwallSettingsError::Daemon(
            "Daemon process failed to start".to_string(),
        ))?;

        Ok(())
    }
}
