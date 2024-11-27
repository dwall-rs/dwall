use std::path::PathBuf;

use windows::Win32::UI::WindowsAndMessaging::{
    SystemParametersInfoW, SPIF_SENDWININICHANGE, SPIF_UPDATEINIFILE, SPI_GETDESKWALLPAPER,
    SPI_SETDESKWALLPAPER, SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS,
};

use crate::{
    error::DwallResult,
    solar::{calculate_angle_difference, SolarAngle},
};

/// Wallpaper management utilities
pub struct WallpaperManager;

impl WallpaperManager {
    /// Retrieves the current desktop wallpaper path
    fn get_current_wallpaper() -> DwallResult<PathBuf> {
        let mut buffer = vec![0u16; 1024];

        unsafe {
            SystemParametersInfoW(
                SPI_GETDESKWALLPAPER,
                buffer.len() as u32,
                Some(buffer.as_mut_ptr() as *mut _),
                SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS(0),
            )?;

            let current_wallpaper = String::from_utf16_lossy(&buffer)
                .trim_matches('\0')
                .to_string();

            trace!("Current wallpaper path: {}", current_wallpaper);
            Ok(PathBuf::from(current_wallpaper))
        }
    }

    /// Sets the desktop wallpaper
    pub fn set_wallpaper(image_path: PathBuf) -> DwallResult<()> {
        let current_wallpaper = Self::get_current_wallpaper()?;

        if current_wallpaper == image_path {
            debug!("Wallpaper already set: {:?}", image_path);
            return Ok(());
        }

        let wide_path: Vec<u16> = image_path
            .to_string_lossy()
            .encode_utf16()
            .chain(std::iter::once(0))
            .collect();

        unsafe {
            SystemParametersInfoW(
                SPI_SETDESKWALLPAPER,
                0,
                Some(wide_path.as_ptr() as *mut _),
                SPIF_UPDATEINIFILE | SPIF_SENDWININICHANGE,
            )?;
        }

        info!("Wallpaper updated: {:?}", image_path);
        Ok(())
    }

    /// Finds the closest matching image based on solar angles
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
