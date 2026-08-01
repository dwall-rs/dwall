use std::path::PathBuf;

use crate::config::Config;

/// Gets the theme directory path for a given theme identifier
pub(crate) fn get_theme_directory_path(
    configuration: &Config,
    theme_identifier: &str,
) -> (PathBuf, bool) {
    let path = configuration.themes_directory().join(theme_identifier);
    if path.exists() {
        return (path, false);
    }

    let path = configuration
        .customized_themes_directory()
        .join(theme_identifier);

    (path, true)
}
