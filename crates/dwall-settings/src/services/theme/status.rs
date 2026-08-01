//! Theme status provider service

use dwall::DWALL_CONFIG_DIR;
use dwall::infrastructure::filesystem::config_reader::ConfigReader;

use crate::error::DwallSettingsResult;
use crate::infrastructure::process::finder::find_process_by_path;

/// Provides information about the current theme state
pub struct ThemeStatusProvider;

impl ThemeStatusProvider {
    /// Get the currently applied theme ID for a monitor
    pub fn get_current_theme_id(monitor_id: &str) -> DwallSettingsResult<Option<String>> {
        let daemon_path = crate::DAEMON_EXE_PATH.get().unwrap();
        if find_process_by_path(daemon_path)?.is_none() {
            return Ok(None);
        }

        let config_path = DWALL_CONFIG_DIR.join("config.toml");
        let config = ConfigReader::read_from_path(&config_path)?;

        let monitor_themes = config.monitor_specific_wallpapers();

        let theme_id = if monitor_id == "all" {
            match monitor_themes {
                Some(dwall::config::MonitorSpecificWallpapers::All(theme_id)) => {
                    Some(theme_id.clone())
                }
                Some(dwall::config::MonitorSpecificWallpapers::Individual(themes_map)) => {
                    let mut iter = themes_map.values();
                    let first_value = iter.next();
                    if iter.all(|value| Some(value) == first_value) {
                        first_value.cloned()
                    } else {
                        None
                    }
                }
                None => None,
            }
        } else {
            monitor_themes.and_then(|t| t.get(monitor_id).cloned())
        };

        Ok(theme_id)
    }
}
