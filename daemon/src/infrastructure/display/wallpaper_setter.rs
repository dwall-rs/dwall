//! Wallpaper setting infrastructure for Windows platform

use std::{collections::HashMap, path::Path};

use windows::{
    Foundation::Uri,
    Storage::{IStorageFile, StorageFile},
    System::UserProfile::LockScreen,
    Win32::{
        System::Com::{CLSCTX_ALL, CoCreateInstance, CoInitialize, CoUninitialize},
        UI::Shell::{DesktopWallpaper, IDesktopWallpaper},
    },
    core::{HSTRING, Interface},
};

use crate::{
    domain::{time::solar_calculator::SolarAngle, visual::wallpaper::WallpaperSelector},
    error::DwallResult,
};

use super::monitor_manager::{DisplayMonitor, DisplayMonitorProvider};

/// Wallpaper operation errors
#[derive(Debug, thiserror::Error)]
pub enum WallpaperError {
    #[error("Failed to create desktop wallpaper COM instance: {0}")]
    Instance(windows::core::Error),
    #[error("Failed to retrieve current wallpaper for monitor: {0}")]
    GetWallpaper(windows::core::Error),
    #[error("Failed to set wallpaper for monitor: {0}")]
    SetWallpaper(windows::core::Error),
    #[error("Monitor with specified ID not found: {0}")]
    MonitorNotFound(String),
}

type WallpaperResult<T> = Result<T, WallpaperError>;

/// Windows wallpaper setter with COM management
pub(crate) struct WallpaperSetter {
    desktop_wallpaper: IDesktopWallpaper,
    monitor_provider: DisplayMonitorProvider,
    should_cleanup_com: bool,
}

impl WallpaperSetter {
    /// Creates a new WallpaperSetter instance
    pub(crate) fn new() -> WallpaperResult<Self> {
        let should_cleanup_com = unsafe {
            let result = CoInitialize(None);
            result.0 == 0
        };

        let desktop_wallpaper: IDesktopWallpaper = unsafe {
            CoCreateInstance(&DesktopWallpaper as *const _, None, CLSCTX_ALL).map_err(|e| {
                error!(
                    error = %e,
                    "Failed to create desktop wallpaper COM instance"
                );
                WallpaperError::Instance(e)
            })?
        };

        Ok(Self {
            desktop_wallpaper,
            monitor_provider: DisplayMonitorProvider::new(),
            should_cleanup_com,
        })
    }

    /// Sets wallpaper for a specific monitor
    pub(crate) fn set_monitor_wallpaper(
        &self,
        monitor_id: &str,
        image_path: &Path,
    ) -> DwallResult<()> {
        if !image_path.exists() {
            error!(
                image_path = %image_path.display(),
                "Image path does not exist. Cannot proceed with wallpaper setting."
            );
            return Err(
                std::io::Error::new(std::io::ErrorKind::NotFound, "Image file not found").into(),
            );
        }

        let monitors = self.list_available_monitors()?;
        let monitor = monitors.get(monitor_id).ok_or_else(|| {
            error!(
                monitor_id = monitor_id,
                "Monitor with specified ID not found"
            );
            WallpaperError::MonitorNotFound(monitor_id.to_string())
        })?;

        if let Err(error) = self.set_wallpaper(monitor, image_path) {
            match error {
                WallpaperError::SetWallpaper(_) => {
                    self.retry_set_wallpaper(monitor_id, image_path)?
                }
                _ => {
                    error!(
                        error = %error,
                        "Failed to set wallpaper for monitor"
                    );
                    return Err(error.into());
                }
            }
        }

        Ok(())
    }

    /// Gets all available monitors with caching
    pub(crate) fn list_available_monitors(&self) -> DwallResult<HashMap<String, DisplayMonitor>> {
        self.monitor_provider.get_monitors()
    }

    /// Forces a refresh of monitor information
    pub(crate) fn reload_monitor_configuration(
        &self,
    ) -> DwallResult<HashMap<String, DisplayMonitor>> {
        self.monitor_provider.refresh_monitors()
    }

    /// Detects if monitor configuration has changed since last check
    ///
    /// This is useful for detecting when monitors are plugged/unplugged during daemon runtime.
    /// The check is lightweight and relies on the cached monitor list comparison.
    ///
    /// # Returns
    /// - `Ok(true)` if monitor configuration has changed
    /// - `Ok(false)` if monitor configuration is unchanged
    /// - `Err(_)` if unable to query current monitor configuration
    pub fn is_monitor_configuration_stale(&self) -> DwallResult<bool> {
        self.monitor_provider.has_configuration_changed()
    }

    /// Sets lock screen wallpaper
    pub(crate) fn set_lock_screen_image(image_path: &Path) -> DwallResult<()> {
        let image_path_hstring = HSTRING::from(image_path);
        let uri = Uri::CreateUri(&image_path_hstring).map_err(|e| {
            error!(
                path = %image_path.display(),
                error = %e,
                "Failed to create URI for lock screen image",
            );
            e
        })?;

        debug!(path = %uri.DisplayUri()?, "Target lock screen image");

        let current_lock_screen_image_uri = Self::get_current_lock_screen_image()?;

        if uri.Equals(&current_lock_screen_image_uri)? {
            info!(path = %image_path.display(), "Lock screen image already set");
            return Ok(());
        }

        let file = StorageFile::GetFileFromPathAsync(&image_path_hstring).map_err(|e| {
            error!(
                path = %image_path.display(),
                error = %e,
                "Failed to get storage file for lock screen",
            );
            e
        })?;

        let file = file.get().map_err(|e| {
            error!(
                path = %image_path.display(),
                error = %e,
                "Failed to retrieve async storage file",
            );
            e
        })?;

        let i_storage_file: IStorageFile = file.cast().map_err(|e| {
            error!(
                path = %image_path.display(),
                error = %e,
                "Failed to cast storage file",
            );
            e
        })?;

        let result = LockScreen::SetImageFileAsync(&i_storage_file).map_err(|e| {
            error!(
                path = %image_path.display(),
                error = %e,
                "Failed to set lock screen image async",
            );
            e
        })?;

        result.get().map_err(|e| {
            error!(
                path = %image_path.display(),
                error = %e,
                "Failed to complete lock screen image setting",
            );
            e
        })?;

        info!(path = %image_path.display(), "Lock screen image updated");
        Ok(())
    }

    /// Finds the closest matching image using wallpaper selection logic
    pub(crate) fn find_closest_image(
        solar_configs: &[SolarAngle],
        current_altitude: f64,
        current_azimuth: f64,
    ) -> Option<u8> {
        WallpaperSelector::find_closest_image(solar_configs, current_altitude, current_azimuth)
    }

    // Private methods
    fn retry_set_wallpaper(&self, monitor_id: &str, wallpaper_path: &Path) -> DwallResult<()> {
        warn!("Refreshing monitor information and retrying...");
        self.reload_monitor_configuration()?;

        let monitors = self.list_available_monitors()?;
        let monitor = monitors.get(monitor_id).ok_or_else(|| {
            error!(
                monitor_id = monitor_id,
                "Monitor with specified ID not found"
            );
            WallpaperError::MonitorNotFound(monitor_id.to_string())
        })?;

        self.set_wallpaper(monitor, wallpaper_path).map_err(|e| {
            error!(
                error = %e,
                monitor_id = monitor_id,
                wallpaper_path = %wallpaper_path.display(),
                "Failed to set wallpaper for monitor after refresh"
            );
            e
        })?;

        Ok(())
    }

    fn has_wallpaper(
        &self,
        monitor_path: &HSTRING,
        wallpaper_path: &HSTRING,
    ) -> WallpaperResult<bool> {
        debug!(
            monitor_path = %monitor_path,
            "Checking existing wallpaper for monitor"
        );

        let current_wallpaper_ptr =
            match unsafe { self.desktop_wallpaper.GetWallpaper(monitor_path) } {
                Ok(path) => path,
                Err(e) => {
                    error!(
                        error = %e,
                        monitor_path = %monitor_path,
                        "Failed to retrieve wallpaper for monitor"
                    );
                    return Err(WallpaperError::GetWallpaper(e));
                }
            };

        let current_wallpaper_path = unsafe { current_wallpaper_ptr.to_hstring() };
        let is_same = *wallpaper_path == current_wallpaper_path;

        debug!(
            current = %current_wallpaper_path,
            new = %wallpaper_path,
            is_same = is_same,
            "Wallpaper comparison"
        );

        Ok(is_same)
    }

    fn set_wallpaper(
        &self,
        monitor: &DisplayMonitor,
        wallpaper_path: &Path,
    ) -> WallpaperResult<()> {
        let wallpaper_path = HSTRING::from(wallpaper_path);
        let device_path = HSTRING::from(monitor.device_path());

        if self.has_wallpaper(&device_path, &wallpaper_path)? {
            debug!(
                monitor_id = monitor.device_path(),
                wallpaper_path = %wallpaper_path,
                "Wallpaper already set for monitor, skipping"
            );
            return Ok(());
        };

        unsafe {
            self.desktop_wallpaper
                .SetWallpaper(&device_path, &wallpaper_path)
                .map_err(|e| {
                    error!(
                        error = %e,
                        monitor_id = monitor.device_path(),
                        wallpaper_path = %wallpaper_path,
                        "Failed to set wallpaper for monitor"
                    );
                    WallpaperError::SetWallpaper(e)
                })
        }
    }

    fn get_current_lock_screen_image() -> DwallResult<Uri> {
        let result = LockScreen::OriginalImageFile().map_err(|e| {
            error!(error = %e, "Failed to retrieve lock screen image");
            e
        })?;

        debug!(path = %result.DisplayUri()?, "Current lock screen image");
        Ok(result)
    }
}

impl Drop for WallpaperSetter {
    fn drop(&mut self) {
        if self.should_cleanup_com {
            unsafe {
                CoUninitialize();
            }
            debug!("COM resources released");
        }
    }
}
