use std::path::{Path, PathBuf};

use windows::{
    core::{Interface, HSTRING},
    Foundation::Uri,
    Storage::{IStorageFile, StorageFile},
    System::UserProfile::LockScreen,
    Win32::UI::WindowsAndMessaging::{
        SystemParametersInfoW, SPIF_SENDWININICHANGE, SPIF_UPDATEINIFILE, SPI_GETDESKWALLPAPER,
        SPI_SETDESKWALLPAPER, SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS,
    },
};

use crate::{
    error::DwallResult,
    solar::{calculate_angle_difference, SolarAngle},
};

/// Wallpaper management utilities
pub struct WallpaperManager;

impl WallpaperManager {
    /// Retrieves the current desktop wallpaper path
    fn get_current_desktop_wallpaper() -> DwallResult<PathBuf> {
        let mut buffer = vec![0u16; 1024];

        // TODO: `windows::Win32::UI::Shell::IDesktopWallpaper::GetWallpaper` may be better
        unsafe {
            SystemParametersInfoW(
                SPI_GETDESKWALLPAPER,
                buffer.len() as u32,
                Some(buffer.as_mut_ptr() as *mut _),
                SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS(0),
            )
            .map_err(|e| {
                error!("Failed to retrieve desktop wallpaper: {}", e);
                e
            })?;

            let current_wallpaper = String::from_utf16_lossy(&buffer)
                .trim_matches('\0')
                .to_string();

            trace!("Current desktop wallpaper path: {}", current_wallpaper);
            Ok(PathBuf::from(current_wallpaper))
        }
    }

    /// Sets the desktop wallpaper
    pub fn set_desktop_wallpaper(image_path: &Path) -> DwallResult<()> {
        let current_wallpaper = Self::get_current_desktop_wallpaper()?;

        if current_wallpaper == image_path {
            debug!("Desktop wallpaper already set: {}", image_path.display());
            return Ok(());
        }

        let wide_path: Vec<u16> = image_path
            .to_string_lossy()
            .encode_utf16()
            .chain(std::iter::once(0))
            .collect();

        // TODO: `windows::Win32::UI::Shell::IDesktopWallpaper::SetWallpaper` may be better
        unsafe {
            SystemParametersInfoW(
                SPI_SETDESKWALLPAPER,
                0,
                Some(wide_path.as_ptr() as *mut _),
                SPIF_UPDATEINIFILE | SPIF_SENDWININICHANGE,
            )
            .map_err(|e| {
                error!(
                    "Failed to set desktop wallpaper: {}, Error: {}",
                    image_path.display(),
                    e
                );
                e
            })?;
        }

        info!("Desktop wallpaper updated: {}", image_path.display());
        Ok(())
    }

    fn get_current_lock_screen_image() -> DwallResult<Uri> {
        let result = LockScreen::OriginalImageFile().map_err(|e| {
            error!("Failed to retrieve lock screen image: {}", e);
            e
        })?;

        trace!("Current lock screen image path: {}", result.DisplayUri()?);
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

        let current_lock_screen_image_uri = Self::get_current_lock_screen_image()?;

        if uri == current_lock_screen_image_uri {
            debug!("Lock screen image already set: {}", image_path.display());
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
    ///
    /// # Arguments
    /// - `solar_configs`: Slice of predefined solar angle configurations
    /// - `current_altitude`: Current solar altitude angle
    /// - `current_azimuth`: Current solar azimuth angle
    ///
    /// # Returns
    /// - `Some(index)` of the closest matching image configuration
    /// - `None` if no suitable match is found
    ///
    /// # Algorithm
    /// Calculates the angle difference for each configuration and selects
    /// the configuration with the minimum difference
    pub fn find_closest_image(
        solar_configs: &[SolarAngle],
        current_altitude: f64,
        current_azimuth: f64,
    ) -> Option<u8> {
        solar_configs
            .iter()
            .map(|config| {
                let angle_difference =
                    calculate_angle_difference(config, current_altitude, current_azimuth);
                (config.index, angle_difference)
            })
            .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
            .map(|(index, _)| index)
    }
}
