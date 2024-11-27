use std::path::Path;

use tokio::fs;

use crate::{
    error::DwallResult,
    solar::SolarAngle,
    theme::{ThemeError, THEMES_DIR},
};

/// Theme validation utilities
pub struct ThemeValidator;

impl ThemeValidator {
    /// Checks if a theme exists and has valid configuration
    pub async fn validate_theme(theme_id: &str) -> DwallResult<()> {
        trace!("Validating theme: {}", theme_id);
        let theme_dir = THEMES_DIR.join(theme_id);

        if !theme_dir.exists() {
            warn!("Theme directory not found: {}", theme_id);
            return Err(ThemeError::NotExists.into());
        }

        let solar_angles = Self::read_solar_configuration(&theme_dir).await?;
        let image_indices: Vec<u8> = solar_angles.iter().map(|angle| angle.index).collect();

        if !Self::validate_image_files(&theme_dir, &image_indices, "jpg") {
            warn!("Image validation failed for theme: {}", theme_id);
            return Err(ThemeError::ImageCountMismatch.into());
        }

        info!("Theme validation successful: {}", theme_id);
        Ok(())
    }

    /// Reads solar configuration from theme directory
    async fn read_solar_configuration(theme_dir: &Path) -> DwallResult<Vec<SolarAngle>> {
        let solar_config_path = theme_dir.join("solar.json");

        if !solar_config_path.exists() {
            error!("Solar configuration file missing: {:?}", solar_config_path);
            return Err(ThemeError::MissingSolarConfigFile.into());
        }

        let solar_config_content = fs::read_to_string(&solar_config_path).await.map_err(|e| {
            error!("Failed to read solar configuration: {}", e);
            e
        })?;

        let solar_angles: Vec<SolarAngle> =
            serde_json::from_str(&solar_config_content).map_err(|e| {
                error!("Failed to parse solar configuration JSON: {}", e);
                e
            })?;

        debug!(
            "Loaded {} solar angles from configuration",
            solar_angles.len()
        );
        Ok(solar_angles)
    }

    /// Validates image files in the theme directory
    fn validate_image_files(theme_dir: &Path, indices: &[u8], image_format: &str) -> bool {
        let image_dir = theme_dir.join(image_format);

        if !image_dir.is_dir() {
            warn!("Image directory not found: {:?}", image_dir);
            return false;
        }

        let validation_result = indices.iter().all(|&index| {
            let image_filename = format!("{}.{}", index + 1, image_format);
            let image_path = image_dir.join(image_filename);

            let is_valid = image_path.exists() && image_path.is_file();
            if !is_valid {
                warn!("Missing or invalid image: {:?}", image_path);
            }
            is_valid
        });

        validation_result
    }
}
