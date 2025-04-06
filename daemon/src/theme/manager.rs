use std::path::Path;

use windows::{
    core::{Interface, HSTRING},
    Foundation::Uri,
    Storage::{IStorageFile, StorageFile},
    System::UserProfile::LockScreen,
};

use crate::monitor::MonitorManager;
use crate::{error::DwallResult, solar::SolarAngle};

/// Wallpaper management utilities
pub struct WallpaperManager {
    pub(super) monitor_manager: MonitorManager,
}

impl WallpaperManager {
    pub fn new() -> DwallResult<Self> {
        let monitor_manager = MonitorManager::new()?;
        Ok(Self { monitor_manager })
    }

    /// Sets wallpaper for a specific monitor
    pub async fn set_monitor_wallpaper(
        &self,
        monitor_id: &str,
        image_path: &Path,
    ) -> DwallResult<()> {
        self.monitor_manager
            .set_wallpaper(monitor_id, image_path)
            .await
    }

    fn get_current_lock_screen_image() -> DwallResult<Uri> {
        let result = LockScreen::OriginalImageFile().map_err(|e| {
            error!(error = ?e, "Failed to retrieve lock screen image");
            e
        })?;

        debug!(path = %result.DisplayUri()?, "Current lock screen image");
        Ok(result)
    }

    pub fn set_lock_screen_image(image_path: &Path) -> DwallResult<()> {
        let image_path_hstring = HSTRING::from(image_path);
        let uri = Uri::CreateUri(&image_path_hstring).map_err(|e| {
            error!(
                path = %image_path.display(),
                error = ?e,
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
                error = ?e,
                "Failed to get storage file for lock screen",
            );
            e
        })?;
        let file = file.get().map_err(|e| {
            error!(
                path = %image_path.display(),
                error = ?e,
                "Failed to retrieve async storage file",
            );
            e
        })?;

        let i_storage_file: IStorageFile = file.cast().map_err(|e| {
            error!(
                path = %image_path.display(),
                error = ?e,
                "Failed to cast storage file",
            );
            e
        })?;
        let result = LockScreen::SetImageFileAsync(&i_storage_file).map_err(|e| {
            error!(
                path = %image_path.display(),
                error = ?e,
                "Failed to set lock screen image async",
            );
            e
        })?;
        result.get().map_err(|e| {
            error!(
                path = %image_path.display(),
                error = ?e,
                "Failed to complete lock screen image setting",
            );
            e
        })?;

        info!(path = %image_path.display(), "Lock screen image updated");
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
