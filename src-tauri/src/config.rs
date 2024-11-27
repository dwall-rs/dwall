use serde::{Deserialize, Serialize};
use tokio::fs;

use crate::{error::DwallResult, lazy::APP_CONFIG_DIR};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ImageFormat {
    Jpeg,
}

impl From<&ImageFormat> for &'static str {
    fn from(val: &ImageFormat) -> Self {
        match val {
            ImageFormat::Jpeg => "jpg",
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    #[serde(skip_serializing_if = "Option::is_none")]
    github_mirror_template: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    selected_theme_id: Option<String>,
    image_format: ImageFormat,

    /// The time interval for detecting the solar elevation
    /// angle and azimuth angle, measured in seconds: [1, 300]
    interval: u8,
}

impl Config {
    pub fn theme_id(&self) -> Option<String> {
        self.selected_theme_id.clone()
    }

    pub fn interval(&self) -> u8 {
        self.interval
    }

    pub fn image_format(&self) -> &ImageFormat {
        &self.image_format
    }

    pub fn github_asset_url(&self, github_url: &str) -> String {
        self.github_mirror_template
            .as_ref()
            .and_then(|template| {
                use regex::Regex;

                let re = Regex::new(
                    r"https://github.com/([^/]+)/([^/]+)/releases/download/([^/]+)/(.*)",
                )
                .unwrap();
                re.captures(github_url).map(|caps| {
                    template
                        .replace("<owner>", &caps[1])
                        .replace("<repo>", &caps[2])
                        .replace("<version>", &caps[3])
                        .replace("<asset>", &caps[4])
                })
            })
            .unwrap_or(github_url.to_owned())
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            github_mirror_template: None,
            selected_theme_id: None,
            image_format: ImageFormat::Jpeg,
            interval: if cfg!(debug_assertions) { 1 } else { 30 },
        }
    }
}

#[tauri::command]
pub async fn read_config_file() -> DwallResult<Config> {
    let config_path = APP_CONFIG_DIR.join("config.toml");
    if !config_path.exists() && !config_path.is_file() {
        return Ok(Default::default());
    }

    let content = fs::read_to_string(config_path).await?;

    toml::from_str(&content).map_err(Into::into)
}

#[tauri::command]
pub async fn write_config_file(config: Config) -> DwallResult<()> {
    let string = toml::to_string(&config)?;

    let config_path = APP_CONFIG_DIR.join("config.toml");
    tokio::fs::write(config_path, string.as_bytes())
        .await
        .map_err(Into::into)
}
