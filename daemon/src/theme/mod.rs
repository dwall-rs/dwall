use std::{
    path::PathBuf,
    sync::{Arc, LazyLock},
};

use processor::ThemeProcessor;

use crate::{config::Config, error::DwallResult, lazy::APP_CONFIG_DIR};

pub use self::validator::ThemeValidator;

mod manager;
mod processor;
mod validator;

/// Directory for storing theme configurations
pub static THEMES_DIR: LazyLock<PathBuf> = LazyLock::new(|| APP_CONFIG_DIR.join("themes"));

/// Comprehensive error handling for theme-related operations
#[derive(Debug, thiserror::Error)]
pub enum ThemeError {
    #[error("Theme does not exist")]
    NotExists,
    #[error("Missing solar configuration file")]
    MissingSolarConfigFile,
    #[error("Image count does not match solar configuration")]
    ImageCountMismatch,
}

/// Applies a theme and starts a background task for periodic wallpaper updates
pub async fn apply_theme(config: Config<'_>) -> DwallResult<()> {
    let owned_config = config.owned();
    let theme_id = owned_config.theme_id();

    let theme_id = match theme_id {
        Some(id) => id,
        None => return Ok(()),
    };

    validate_theme(&theme_id).await?;

    let theme_processor = ThemeProcessor::new(&theme_id, Arc::new(owned_config));

    theme_processor.start_update_loop().await
}

async fn validate_theme(theme_id: &str) -> DwallResult<()> {
    ThemeValidator::validate_theme(theme_id).await
}
