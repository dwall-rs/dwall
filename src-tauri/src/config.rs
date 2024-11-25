use serde::{Deserialize, Serialize};
use tokio::fs;

use crate::{error::DwallResult, lazy::APP_CONFIG_DIR};

#[derive(Debug, Serialize, Deserialize)]
struct Config {
    github_mirror_template: Option<String>,
    selected_id: Option<String>,

    /// The time interval for detecting the solar elevation
    /// angle and azimuth angle, measured in seconds: [1, 300]
    interval: u8,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            github_mirror_template: None,
            selected_id: None,
            interval: 1,
        }
    }
}

async fn read_config() -> DwallResult<Config> {
    let config_path = APP_CONFIG_DIR.join("config.toml");
    if !config_path.exists() && !config_path.is_file() {
        return Ok(Default::default());
    }

    let content = fs::read_to_string(config_path).await?;

    toml::from_str(&content).map_err(Into::into)
}

async fn write_config(config: Config) -> DwallResult<()> {
    let string = toml::to_string(&config)?;

    let config_path = APP_CONFIG_DIR.join("config.toml");
    tokio::fs::write(config_path, string.as_bytes())
        .await
        .map_err(Into::into)
}
