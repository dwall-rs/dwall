use std::{
    cell::RefCell,
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
};

use crate::{DwallResult, SolarAngle, domain::visual::ThemeError};

thread_local! {
    /// Cache solar configuration to avoid repeated reads
    static SOLAR_CACHE: RefCell<HashMap<PathBuf, Vec<SolarAngle>>> =
        RefCell::new(HashMap::new());
}

/// Load solar configuration for a specific theme directory
pub(crate) fn load_cached_solar_angles(theme_directory: &Path) -> DwallResult<Vec<SolarAngle>> {
    let theme_directory = theme_directory.canonicalize()?;
    debug!(path = %theme_directory.display(), "Loading solar configuration from canonical and absolute path");

    // Check cache
    {
        if let Some(cached_angles) =
            SOLAR_CACHE.with(|cache| cache.borrow().get(&theme_directory).cloned())
        {
            debug!("Using cached solar configuration");
            return Ok(cached_angles);
        }
    }
    let solar_config_path = theme_directory.join("solar.json");
    // Validate solar configuration file exists
    if !solar_config_path.exists() {
        error!(
            solar_config_path = %solar_config_path.display(),
            "Solar configuration file is missing"
        );
        return Err(ThemeError::SolarConfigurationMissing.into());
    }
    // Read solar configuration file
    let solar_config_content = match fs::read_to_string(&solar_config_path) {
        Ok(content) => content,
        Err(read_error) => {
            error!(
                solar_config_path = %solar_config_path.display(),
                error = ?read_error,
                "Failed to read solar configuration file"
            );
            return Err(read_error.into());
        }
    };
    // Parse solar configuration
    let solar_angles: Vec<SolarAngle> = match serde_json::from_str(&solar_config_content) {
        Ok(angles) => angles,
        Err(parse_error) => {
            error!(
                solar_config_path = %solar_config_path.display(),
                error = ?parse_error,
                "Failed to parse solar configuration JSON"
            );
            return Err(parse_error.into());
        }
    };
    debug!(
        solar_angles_count = solar_angles.len(),
        "Successfully loaded solar configuration"
    );

    // Cache solar configuration
    SOLAR_CACHE.with(|cache| {
        cache
            .borrow_mut()
            .insert(theme_directory.to_path_buf(), solar_angles.clone());
    });

    Ok(solar_angles)
}
