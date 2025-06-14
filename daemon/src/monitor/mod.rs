mod device_interface;
mod display_config;
pub mod error;
mod monitor_info;
mod wallpaper_manager;

use std::collections::HashMap;
use std::path::Path;

use crate::error::DwallResult;

// Re-export Monitor struct for public use
pub use monitor_info::{DisplayMonitor, DisplayMonitorProvider};
pub(crate) use wallpaper_manager::{WallpaperError, WallpaperManager};

/// For backward compatibility with existing code
/// This will be deprecated in future versions
pub struct MonitorManager {
    /// Wallpaper manager instance
    wallpaper_manager: WallpaperManager,
    /// Monitor information provider
    monitor_provider: DisplayMonitorProvider,
}

impl MonitorManager {
    /// Creates a new MonitorManager instance
    pub fn new() -> DwallResult<Self> {
        Ok(Self {
            wallpaper_manager: WallpaperManager::new()?,
            monitor_provider: DisplayMonitorProvider::new(),
        })
    }

    /// Sets wallpaper for a specific monitor
    pub(crate) async fn set_wallpaper(
        &self,
        monitor_id: &str,
        wallpaper_path: &Path,
    ) -> DwallResult<()> {
        // Verify wallpaper path exists
        if !wallpaper_path.exists() {
            error!(
                image_path = %wallpaper_path.display(),
                "Image path does not exist. Cannot proceed with wallpaper setting."
            );
            return Err(
                std::io::Error::new(std::io::ErrorKind::NotFound, "Image file not found").into(),
            );
        }

        let monitors = self.get_monitors().await?;
        // Find monitor with specified ID
        let monitor = monitors.get(monitor_id).ok_or_else(|| {
            error!(
                monitor_id = monitor_id,
                "Monitor with specified ID not found"
            );
            WallpaperError::MonitorNotFound(monitor_id.to_string())
        })?;

        if let Err(error) = self
            .wallpaper_manager
            .set_wallpaper(monitor, wallpaper_path)
            .await
        {
            match error {
                WallpaperError::SetWallpaper(_) => {
                    self.retry_set_wallpaper(monitor_id, wallpaper_path).await?
                }
                _ => {
                    error!(
                        error = ?error,
                        "Failed to set wallpaper for monitor"
                    );
                    return Err(error.into());
                }
            }
        }

        Ok(())
    }

    /// If setting wallpaper failed, try to refresh monitor info and retry once
    async fn retry_set_wallpaper(
        &self,
        monitor_id: &str,
        wallpaper_path: &Path,
    ) -> DwallResult<()> {
        warn!("Refreshing monitor information and retrying...");
        self.refresh_monitors().await?;

        let monitors = self.get_monitors().await?;
        // Find monitor with specified ID
        let monitor = monitors.get(monitor_id).ok_or_else(|| {
            error!(
                monitor_id = monitor_id,
                "Monitor with specified ID not found"
            );
            WallpaperError::MonitorNotFound(monitor_id.to_string())
        })?;

        self.wallpaper_manager
            .set_wallpaper(monitor, wallpaper_path)
            .await
            .map_err(|e| {
                error!(
                    error = ?e,
                    monitor_id = monitor_id,
                    wallpaper_path = %wallpaper_path.display(),
                    "Failed to set wallpaper for monitor after refresh"
                );
                e
            })?;

        Ok(())
    }

    /// Gets all available monitors with caching
    pub async fn get_monitors(&self) -> DwallResult<HashMap<String, DisplayMonitor>> {
        self.monitor_provider.get_monitors().await
    }

    /// Forces a refresh of monitor information
    pub async fn refresh_monitors(&self) -> DwallResult<HashMap<String, DisplayMonitor>> {
        self.monitor_provider.refresh_monitors().await
    }

    /// Detects if monitor configuration has changed
    pub async fn has_monitor_config_changed(&self) -> DwallResult<bool> {
        self.monitor_provider.has_configuration_changed().await
    }
}
