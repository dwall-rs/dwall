//! Theme validation utilities for solar-based wallpaper themes

use std::{fs, path::Path};

use crate::{
    config::ImageFormat,
    domain::{time::solar_calculator::SolarAngle, visual::theme::ThemeError},
    error::DwallResult,
};

/// Solar configuration filename
const SOLAR_CONFIG_FILENAME: &str = "solar.json";

/// Theme validation utilities for solar-based wallpaper themes
pub struct ThemeValidator;

impl ThemeValidator {
    /// Validates if a solar theme exists and has proper configuration and image files
    pub fn validate(
        themes_directory: &Path,
        theme_identifier: &str,
        is_customized: bool,
        image_format: &ImageFormat,
    ) -> DwallResult<()> {
        trace!(
            theme_id = theme_identifier,
            themes_directory = %themes_directory.display(),
            "Starting solar theme validation"
        );

        let theme_directory_path = themes_directory.join(theme_identifier);

        if !theme_directory_path.exists() {
            warn!(
                theme_id = theme_identifier,
                theme_path = %theme_directory_path.display(),
                "Solar theme directory not found"
            );
            return Err(ThemeError::ThemeDirectoryNotFound(theme_identifier.to_string()).into());
        }

        let solar_angle_configuration = load_solar_angle_configuration(&theme_directory_path)?;
        let expected_image_indices: Vec<u8> = solar_angle_configuration
            .iter()
            .map(|angle| angle.index())
            .collect();

        if !validate_theme_image_files(
            &theme_directory_path,
            &expected_image_indices,
            is_customized,
            image_format.as_str(),
        ) {
            warn!(
                theme_id = theme_identifier,
                expected_images = expected_image_indices.len(),
                "Solar theme image validation failed"
            );
            return Err(ThemeError::ImageSolarConfigurationMismatch {
                expected: expected_image_indices.len(),
                found: 0,
            }
            .into());
        }

        info!(
            theme_id = theme_identifier,
            solar_angles_count = solar_angle_configuration.len(),
            is_customized = is_customized,
            "Solar theme validation completed successfully"
        );
        Ok(())
    }
}

/// Loads solar angle configuration from theme directory
fn load_solar_angle_configuration(theme_directory: &Path) -> DwallResult<Vec<SolarAngle>> {
    let solar_configuration_file_path = theme_directory.join(SOLAR_CONFIG_FILENAME);

    if !solar_configuration_file_path.exists() {
        error!(
            solar_config_path = %solar_configuration_file_path.display(),
            "Solar configuration file 'solar.json' is missing from theme directory"
        );
        return Err(ThemeError::SolarConfigurationMissing.into());
    }

    let solar_configuration_content =
        fs::read_to_string(&solar_configuration_file_path).map_err(|io_error| {
            error!(
                error = %io_error,
                solar_config_path = %solar_configuration_file_path.display(),
                "Failed to read solar configuration file"
            );
            io_error
        })?;

    let solar_angle_list: Vec<SolarAngle> = serde_json::from_str(&solar_configuration_content)
        .map_err(|json_error| {
            error!(
                error = %json_error,
                solar_config_path = %solar_configuration_file_path.display(),
                "Failed to parse solar configuration JSON"
            );
            json_error
        })?;

    debug!(
        solar_angles_count = solar_angle_list.len(),
        solar_config_path = %solar_configuration_file_path.display(),
        "Successfully loaded solar angle configuration"
    );
    Ok(solar_angle_list)
}

/// Validates that all required image files exist for the theme's solar configuration
fn validate_theme_image_files(
    theme_directory: &Path,
    expected_image_indices: &[u8],
    is_customized: bool,
    image_file_format: &str,
) -> bool {
    let images_directory_path = if is_customized {
        theme_directory.join("images")
    } else {
        theme_directory.join(image_file_format)
    };

    if !images_directory_path.is_dir() {
        warn!(
            images_directory = %images_directory_path.display(),
            "Theme images directory not found or is not a directory"
        );
        return false;
    }

    let mut missing_images_count = 0;
    let validation_successful = expected_image_indices.iter().all(|&image_index| {
        let image_filename = format!("{}.{}", image_index + 1, image_file_format);
        let image_file_path = images_directory_path.join(image_filename);

        let image_exists = image_file_path.exists() && image_file_path.is_file();
        if !image_exists {
            missing_images_count += 1;
            warn!(
                image_path = %image_file_path.display(),
                image_index = image_index,
                "Required theme image file is missing"
            );
        }
        image_exists
    });

    if !validation_successful {
        error!(
            missing_images = missing_images_count,
            total_expected = expected_image_indices.len(),
            "Theme image validation failed due to missing files"
        );
    }

    validation_successful
}
