use std::path::Path;

use windows::{
    core::{Interface, HSTRING, PWSTR},
    Foundation::Uri,
    Storage::{IStorageFile, StorageFile},
    System::UserProfile::LockScreen,
    Win32::{
        System::Com::{CoCreateInstance, CLSCTX_ALL},
        UI::Shell::{DesktopWallpaper, IDesktopWallpaper},
    },
};

use crate::{error::DwallResult, solar::SolarAngle};

/// Wallpaper management utilities
pub struct WallpaperManager;

impl WallpaperManager {
    /// Retrieves the number of monitors
    fn get_monitor_count(desktop_wallpaper: &IDesktopWallpaper) -> DwallResult<u32> {
        trace!("Attempting to retrieve monitor device path count");
        unsafe {
            desktop_wallpaper.GetMonitorDevicePathCount().map_err(|e| {
                error!(
                    error = %e,
                    "Critical failure: Unable to retrieve monitor device path count.
                     This may indicate a system-level graphics configuration issue."
                );
                trace!("Monitor count retrieval failed with error: {}", e);
                e.into()
            })
        }
    }

    /// Checks if the wallpaper is already set for a specific monitor
    fn is_wallpaper_already_set(
        desktop_wallpaper: &IDesktopWallpaper,
        monitor_path: PWSTR,
        wallpaper_path: &HSTRING,
    ) -> DwallResult<bool> {
        trace!(
            monitor_path = %unsafe{monitor_path.display()},
            "Checking existing wallpaper for monitor"
        );

        let current_wallpaper_path = match unsafe { desktop_wallpaper.GetWallpaper(monitor_path) } {
            Ok(path) => path,
            Err(e) => {
                error!(
                    error = %e,
                    monitor_path = %unsafe{monitor_path.display()},
                    "Failed to retrieve current wallpaper for monitor"
                );
                return Err(e.into());
            }
        };

        let is_same = *wallpaper_path == unsafe { current_wallpaper_path.to_hstring() }.unwrap();

        trace!(
            current_wallpaper_path = %unsafe{current_wallpaper_path.display()},
            new_wallpaper_path = %wallpaper_path,
            is_same = is_same,
            "Wallpaper comparison result"
        );

        Ok(is_same)
    }

    /// Sets wallpaper for a specific monitor
    fn set_monitor_wallpaper(
        desktop_wallpaper: &IDesktopWallpaper,
        monitor_path: PWSTR,
        wallpaper_path: &HSTRING,
        monitor_index: u32,
    ) -> DwallResult<()> {
        trace!(
            monitor_index = monitor_index,
            wallpaper_path = %wallpaper_path,
            "Attempting to set wallpaper for specific monitor"
        );

        unsafe {
            desktop_wallpaper
                .SetWallpaper(monitor_path, wallpaper_path)
                .map_err(|e| {
                    error!(
                        monitor_index = monitor_index,
                        error = %e,
                        wallpaper_path = %wallpaper_path,
                        "Comprehensive wallpaper setting failure:
                     Unable to apply wallpaper to specified monitor. 
                     Check file permissions, path validity, and system graphics settings."
                    );
                    trace!(
                        "Detailed wallpaper setting error for monitor {}: {}",
                        monitor_index,
                        e
                    );
                    e.into()
                })
        }
    }

    /// Sets the desktop wallpaper across all monitors
    pub fn set_desktop_wallpaper(image_path: &Path) -> DwallResult<()> {
        // Enhanced trace-level logging for initial wallpaper setting attempt
        trace!(
            image_path = %image_path.display(),
            "Initiating desktop wallpaper configuration process"
        );

        // Validate image path before proceeding
        if !image_path.exists() {
            error!(
                image_path = %image_path.display(),
                "Image path does not exist. Cannot proceed with wallpaper setting."
            );
            return Err(
                std::io::Error::new(std::io::ErrorKind::NotFound, "Image file not found").into(),
            );
        }

        // Create desktop wallpaper instance with enhanced error handling
        let desktop_wallpaper: IDesktopWallpaper = unsafe {
            CoCreateInstance(&DesktopWallpaper as *const _, None, CLSCTX_ALL).map_err(|e| {
                error!(
                    error = %e,
                    "Critical initialization failure:
                     Unable to create desktop wallpaper COM instance. 
                     This may indicate system COM registration issues."
                );
                trace!("Desktop wallpaper instance creation error: {}", e);
                e
            })?
        };

        // Get total number of monitors with context
        let monitor_count = match Self::get_monitor_count(&desktop_wallpaper) {
            Ok(count) => {
                trace!(
                    monitor_count = count,
                    "Successfully retrieved monitor count"
                );
                count
            }
            Err(e) => {
                error!(
                    error = %e,
                    "Wallpaper configuration aborted due to monitor count retrieval failure"
                );
                return Err(e);
            }
        };

        // Convert image path to HSTRING with logging
        let wallpaper_path = {
            let path_str = image_path.to_string_lossy();
            trace!(wallpaper_path = %path_str, "Converted image path to HSTRING");
            HSTRING::from(path_str.as_ref())
        };

        // Iterate through monitors and set wallpaper with comprehensive logging
        for i in 0..monitor_count {
            let monitor_path = match unsafe { desktop_wallpaper.GetMonitorDevicePathAt(i) } {
                Ok(path) => {
                    trace!(
                        monitor_index = i,
                        monitor_path = %unsafe{path.display()},
                        "Retrieved monitor device path"
                    );
                    path
                }
                Err(e) => {
                    error!(
                        monitor_index = i,
                        error = %e,
                        "Failed to retrieve monitor device path. Skipping this monitor."
                    );
                    continue;
                }
            };

            // Skip if wallpaper is already set, with detailed logging
            match Self::is_wallpaper_already_set(&desktop_wallpaper, monitor_path, &wallpaper_path)
            {
                Ok(true) => {
                    info!(
                        image_path = %image_path.display(),
                        monitor_path = %unsafe{monitor_path.display()},
                        monitor_index = i,
                        "Wallpaper already set for this monitor. Skipping."
                    );
                    continue;
                }
                Ok(false) => {
                    trace!(
                        monitor_index = i,
                        "Wallpaper needs to be updated for this monitor"
                    );
                }
                Err(e) => {
                    error!(
                        monitor_index = i,
                        error = %e,
                        "Error checking existing wallpaper. Attempting to set new wallpaper."
                    );
                }
            }

            // Set wallpaper for the monitor
            if let Err(e) =
                Self::set_monitor_wallpaper(&desktop_wallpaper, monitor_path, &wallpaper_path, i)
            {
                warn!(
                    monitor_index = i,
                    error = %e,
                    "Failed to set wallpaper for a specific monitor. Continuing with other monitors."
                );
                // Continue to next monitor instead of completely failing
                continue;
            }
        }

        info!(
            image_path = %image_path.display(),
            monitor_count = monitor_count,
            "Wallpaper configuration completed. Processed across all available monitors."
        );

        Ok(())
    }

    fn get_current_lock_screen_image() -> DwallResult<Uri> {
        let result = LockScreen::OriginalImageFile().map_err(|e| {
            error!("Failed to retrieve lock screen image: {}", e);
            e
        })?;

        debug!("Current lock screen image path: {}", result.DisplayUri()?);
        Ok(result)
    }

    pub fn set_lock_screen_image(image_path: &Path) -> DwallResult<()> {
        let image_path_hstring = HSTRING::from(image_path);
        let uri = Uri::CreateUri(&image_path_hstring).map_err(|e| {
            error!(
                "Failed to create URI for lock screen image: {}, Error: {}",
                image_path.display(),
                e
            );
            e
        })?;
        debug!("Target lock screen image path: {}", uri.DisplayUri()?);

        let current_lock_screen_image_uri = Self::get_current_lock_screen_image()?;

        if uri.Equals(&current_lock_screen_image_uri)? {
            info!("Lock screen image already set: {}", image_path.display());
            return Ok(());
        }

        let file = StorageFile::GetFileFromPathAsync(&image_path_hstring).map_err(|e| {
            error!(
                "Failed to get storage file for lock screen: {}, Error: {}",
                image_path.display(),
                e
            );
            e
        })?;
        let file = file.get().map_err(|e| {
            error!(
                "Failed to retrieve async storage file: {}, Error: {}",
                image_path.display(),
                e
            );
            e
        })?;

        let i_storage_file: IStorageFile = file.cast().map_err(|e| {
            error!(
                "Failed to cast storage file: {}, Error: {}",
                image_path.display(),
                e
            );
            e
        })?;
        let result = LockScreen::SetImageFileAsync(&i_storage_file).map_err(|e| {
            error!(
                "Failed to set lock screen image async: {}, Error: {}",
                image_path.display(),
                e
            );
            e
        })?;
        result.get().map_err(|e| {
            error!(
                "Failed to complete lock screen image setting: {}, Error: {}",
                image_path.display(),
                e
            );
            e
        })?;

        info!("Lock screen image updated: {}", image_path.display());
        Ok(())
    }

    /// Finds the closest matching image based on solar angles
    pub fn find_closest_image(
        solar_configs: &[SolarAngle],
        current_altitude: f64,
        current_azimuth: f64,
    ) -> Option<u8> {
        let min_altitude_diff = solar_configs
            .iter()
            .map(|sa| (sa.altitude - current_altitude).abs())
            .min_by(|a, b| a.partial_cmp(b).unwrap())?;

        let closest_altitude_matches: Vec<&SolarAngle> = solar_configs
            .iter()
            .filter(|sa| (sa.altitude - current_altitude).abs() == min_altitude_diff)
            .collect();

        if closest_altitude_matches.len() == 1 {
            return closest_altitude_matches[0].index.into();
        }

        closest_altitude_matches
            .iter()
            .min_by(|&&a, &&b| {
                (a.azimuth - current_azimuth)
                    .abs()
                    .partial_cmp(&(b.azimuth - current_azimuth).abs())
                    .unwrap()
            })
            .map(|&sa| sa.index)
    }
}
