use std::path::Path;

use windows::{
    core::HSTRING,
    Win32::{
        System::Com::{CoCreateInstance, CoInitialize, CoUninitialize, CLSCTX_ALL},
        UI::Shell::{DesktopWallpaper, IDesktopWallpaper},
    },
};

use super::monitor_info::DisplayMonitor;

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

/// Manager for wallpaper operations on Windows
/// 
/// This struct handles COM initialization and provides methods for
/// querying and setting desktop wallpapers per monitor.
pub struct WallpaperManager {
    /// Windows Desktop Wallpaper COM interface
    desktop_wallpaper: IDesktopWallpaper,
    /// Flag indicating whether COM was initialized by this instance
    /// If true, COM will be uninitialized when this instance is dropped
    should_cleanup_com: bool,
}

impl WallpaperManager {
    /// Creates a new WallpaperManager instance
    /// 
    /// Initializes COM if necessary and creates the Desktop Wallpaper COM instance.
    /// 
    /// # Returns
    /// - `Ok(WallpaperManager)` - Successfully created wallpaper manager
    /// - `Err(WallpaperError::Instance)` - Failed to create COM instance
    pub fn new() -> WallpaperResult<Self> {
        // Continue execution even if initialization fails, as it might have been initialized elsewhere
        let should_cleanup_com = unsafe {
            // Attempt to initialize COM
            let result = CoInitialize(None);
            // If returns S_OK (0), means initialization succeeded and needs cleanup
            // If returns S_FALSE, means already initialized and no cleanup needed
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
            should_cleanup_com,
        })
    }

    /// Checks if the wallpaper is already set for a specific monitor
    /// 
    /// This optimization avoids unnecessary wallpaper changes when the
    /// desired wallpaper is already active.
    /// 
    /// # Arguments
    /// * `monitor_path` - The device path of the monitor
    /// * `wallpaper_path` - The desired wallpaper file path
    /// 
    /// # Returns
    /// - `Ok(true)` if wallpaper is already set
    /// - `Ok(false)` if wallpaper is different
    /// - `Err(WallpaperError)` if unable to query current wallpaper
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

    /// Sets wallpaper for a specific monitor
    pub async fn set_wallpaper(
        &self,
        monitor: &DisplayMonitor,
        wallpaper_path: &Path,
    ) -> WallpaperResult<()> {
        // Convert wallpaper path to HSTRING
        let wallpaper_path = HSTRING::from(wallpaper_path);
        let device_path = HSTRING::from(monitor.device_path());

        // Check if wallpaper is already set to avoid unnecessary operations
        if self.has_wallpaper(&device_path, &wallpaper_path)? {
            debug!(
                monitor_id = monitor.device_path(),
                wallpaper_path = %wallpaper_path,
                "Wallpaper already set for monitor, skipping"
            );
            return Ok(());
        };

        // Set wallpaper
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
}

impl Drop for WallpaperManager {
    fn drop(&mut self) {
        // Only uninitialize COM if we successfully initialized it
        if self.should_cleanup_com {
            unsafe {
                CoUninitialize();
            }
            debug!("COM resources released");
        }
    }
}
