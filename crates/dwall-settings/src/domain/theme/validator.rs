//! Theme validator (domain knowledge: theme structure rules)

use std::path::Path;

use dwall::config::ImageFormat;

/// Validates theme structure and content
pub struct ThemeValidator;

impl ThemeValidator {
    /// Validate a theme against the solar theme specification
    pub fn validate(
        themes_directory: &Path,
        theme_id: &str,
        is_customized: bool,
        image_format: &ImageFormat,
    ) -> Result<(), dwall::error::DwallError> {
        dwall::ThemeValidator::validate(themes_directory, theme_id, is_customized, image_format)
    }
}
