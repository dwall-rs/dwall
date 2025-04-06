use crate::{config::Config, error::DwallResult};

use self::processor::ThemeProcessor;
pub use self::validator::ThemeValidator;

mod manager;
mod processor;
mod validator;

/// Comprehensive error handling for theme-related operations
#[derive(Debug, thiserror::Error)]
pub enum ThemeError {
    #[error("Theme does not exist")]
    NotExists,
    #[error("Missing default theme")]
    MissingDefaultTheme,
    #[error("Missing solar configuration file")]
    MissingSolarConfigFile,
    #[error("Image count does not match solar configuration")]
    ImageCountMismatch,
    #[error("Wallpaper file does not exist")]
    MissingWallpaperFile,
}

/// Applies a theme and starts a background task for periodic wallpaper updates
pub async fn apply_theme(config: Config) -> DwallResult<()> {
    let theme_processor = ThemeProcessor::new(&config);

    theme_processor.start_update_loop().await
}
