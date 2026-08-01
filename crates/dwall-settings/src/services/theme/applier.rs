//! Theme applier service (use case orchestration)

use std::time::Duration;

use dwall::Config;
use tokio::time::sleep;

use crate::error::{DwallSettingsError, DwallSettingsResult};
use crate::infrastructure::logging::daemon_log::get_last_daemon_error;
use crate::infrastructure::process::finder::find_process_by_path;
use crate::infrastructure::process::lifecycle::terminate_process;
use crate::services::daemon::launcher::DaemonLauncher;

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
        let writer = dwall::infrastructure::filesystem::config_writer::ConfigWriter;
        writer.write_to_path(&config_path, &config)?;

        if config.monitor_specific_wallpapers().is_empty() {
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
