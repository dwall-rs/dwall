//! Update loop scheduling with failure handling

use std::time::Duration;

use crate::DwallResult;

pub struct LoopConfig {
    pub max_consecutive_failures: u8,
    pub update_interval: Duration,
}

impl Default for LoopConfig {
    fn default() -> Self {
        Self {
            max_consecutive_failures: 3,
            update_interval: Duration::from_secs(300),
        }
    }
}

/// Controls the update loop lifecycle
///
/// Responsible for:
/// - Timing and sleep intervals
/// - Consecutive failure tracking
/// - Monitor change detection triggers
pub struct UpdateLoop;

impl UpdateLoop {
    pub fn run<F, M>(
        &self,
        config: &LoopConfig,
        mut operation: F,
        mut monitor_checker: M,
    ) -> DwallResult<()>
    where
        F: FnMut() -> DwallResult<()>,
        M: FnMut() -> bool,
    {
        let mut failure_count = 0;

        loop {
            match operation() {
                Ok(_) => {
                    debug!("Theme cycle completed successfully");
                    failure_count = 0;
                }
                Err(e) => {
                    failure_count += 1;
                    error!(
                        error = %e,
                        consecutive_failures = failure_count,
                        max_failures = config.max_consecutive_failures,
                        "Theme cycle failed"
                    );
                    if failure_count >= config.max_consecutive_failures {
                        error!("Maximum consecutive failures reached, terminating loop");
                        break;
                    }
                }
            }

            if monitor_checker() {
                info!("Monitor configuration change detected, reapplying wallpapers");
                continue;
            }

            debug!(
                sleep_seconds = config.update_interval.as_secs(),
                "Waiting before next theme update cycle"
            );

            std::thread::sleep(config.update_interval);
        }

        warn!("Theme update loop terminated");
        Ok(())
    }
}
