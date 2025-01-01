use std::path::Path;

use tokio::fs;

use crate::{error::DwallResult, solar::SolarAngle, theme::ThemeError};

/// Theme validation utilities
pub struct ThemeValidator;

impl ThemeValidator {
    /// Checks if a theme exists and has valid configuration
    pub async fn validate_theme(themes_directory: &Path, theme_id: &str) -> DwallResult<()> {
        trace!(theme_id = theme_id, "Validating theme");
        let theme_dir = themes_directory.join(theme_id);

        if !theme_dir.exists() {
            warn!(theme_id = theme_id, "Theme directory not found");
            return Err(ThemeError::NotExists.into());
        }

        let solar_angles = Self::read_solar_configuration(&theme_dir).await?;
        let image_indices: Vec<u8> = solar_angles.iter().map(|angle| angle.index).collect();

        if !Self::validate_image_files(&theme_dir, &image_indices, "jpg") {
            warn!(theme_id = theme_id, "Image validation failed for theme");
            return Err(ThemeError::ImageCountMismatch.into());
        }

        debug!(theme_id = theme_id, "Theme validation successful");
        Ok(())
    }

    /// Reads solar configuration from theme directory
    async fn read_solar_configuration(theme_dir: &Path) -> DwallResult<Vec<SolarAngle>> {
        let solar_config_path = theme_dir.join("solar.json");

        if !solar_config_path.exists() {
            error!(solar_config_path = %solar_config_path.display(), "Solar configuration file missing");
            return Err(ThemeError::MissingSolarConfigFile.into());
        }

        let solar_config_content = fs::read_to_string(&solar_config_path).await.map_err(|e| {
            error!(error = ?e, "Failed to read solar configuration");
            e
        })?;

        let solar_angles: Vec<SolarAngle> =
            serde_json::from_str(&solar_config_content).map_err(|e| {
                error!(error = ?e, "Failed to parse solar configuration JSON");
                e
            })?;

        debug!(
            solar_angles_count = solar_angles.len(),
            "Loaded solar angles from configuration"
        );
        Ok(solar_angles)
    }

    /// Validates image files in the theme directory
    fn validate_image_files(theme_dir: &Path, indices: &[u8], image_format: &str) -> bool {
        let image_dir = theme_dir.join(image_format);

        if !image_dir.is_dir() {
            warn!(image_dir = %image_dir.display(), "Image directory not found");
            return false;
        }

        let validation_result = indices.iter().all(|&index| {
            let image_filename = format!("{}.{}", index + 1, image_format);
            let image_path = image_dir.join(image_filename);

            let is_valid = image_path.exists() && image_path.is_file();
            if !is_valid {
                warn!(image_path = %image_path.display(), "Missing or invalid image");
            }
            is_valid
        });

        validation_result
    }
}
