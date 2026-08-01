//! Configuration commands

use dwall::infrastructure::filesystem::{config_reader::ConfigReader, config_writer::ConfigWriter};

use crate::error::DwallSettingsResult;

#[tauri::command]
pub async fn read_config_file() -> DwallSettingsResult<dwall::Config> {
    let config_path = dwall::DWALL_CONFIG_DIR.join("config.toml");
    ConfigReader::read_from_path(&config_path).map_err(Into::into)
}

#[tauri::command]
pub async fn write_config_file(config: dwall::Config) -> DwallSettingsResult<()> {
    debug!(config = ?config, "Writing config file");
    let config_path = dwall::DWALL_CONFIG_DIR.join("config.toml");
    let writer = ConfigWriter;
    writer
        .write_to_path(&config_path, &config)
        .map_err(Into::into)
}
