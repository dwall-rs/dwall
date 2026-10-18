//! Current theme status lookup.

use std::path::{Path, PathBuf};

use dwall::DWALL_CONFIG_DIR;
use dwall::config::{Config, ConfigReader, MonitorSpecificWallpapers};

use crate::error::DwallSettingsResult;

/// The configuration file the lookup reads, in the user's profile.
fn default_config_path() -> PathBuf {
    DWALL_CONFIG_DIR.join("config.toml")
}

/// Provides information about the current theme state
pub struct ThemeStatusProvider;

impl ThemeStatusProvider {
    /// Get the currently applied theme ID for a monitor
    pub fn get_current_theme_id(monitor_id: &str) -> DwallSettingsResult<Option<String>> {
        // The config only reflects what is on screen while a daemon applies it.
        if !crate::daemon::is_daemon_running()? {
            return Ok(None);
        }

        Ok(Self::theme_id_for(
            monitor_id,
            read_config(&default_config_path()).as_ref(),
        ))
    }

    /// The theme `monitor_id` should show, for an already-read configuration.
    ///
    /// `"all"` is the stage's aggregate: it answers with a theme only when every
    /// configured monitor agrees, because showing one of several different themes as
    /// "the current theme" would be a guess.
    fn theme_id_for(monitor_id: &str, config: Option<&Config>) -> Option<String> {
        match monitor_id {
            "all" => match config?.monitor_specific_wallpapers()? {
                MonitorSpecificWallpapers::All(theme_id) => Some(theme_id.clone()),
                MonitorSpecificWallpapers::Individual(themes) => {
                    let mut values = themes.values();
                    let first = values.next();
                    if values.all(|value| Some(value) == first) {
                        first.cloned()
                    } else {
                        None
                    }
                }
            },
            monitor_id => config?
                .monitor_specific_wallpapers()?
                .get(monitor_id)
                .cloned(),
        }
    }
}

/// Reads the configuration, or `None` when it is missing or unreadable.
fn read_config(config_path: &Path) -> Option<Config> {
    ConfigReader::read_from_path(config_path).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A configuration with a monitor map, built through `serde` because the fields
    /// are private.
    fn config_with_monitors(pairs: &[(&str, &str)]) -> Config {
        let map: std::collections::HashMap<&str, &str> = pairs.iter().copied().collect();
        let json = format!(
            r#"{{"wallpaper_mode":{{"mode":"fixed","monitor_specific_wallpapers":{}}}}}"#,
            serde_json::to_string(&map).unwrap(),
        );

        serde_json::from_str(&json).unwrap()
    }

    fn config_with_one_theme(theme_id: &str) -> Config {
        let json = format!(
            r#"{{"wallpaper_mode":{{"mode":"fixed","monitor_specific_wallpapers":{}}}}}"#,
            serde_json::to_string(theme_id).unwrap(),
        );

        serde_json::from_str(&json).unwrap()
    }

    #[test]
    fn a_named_monitor_gets_its_own_theme() {
        let config = config_with_monitors(&[("MON-A", "Big Sur"), ("MON-B", "Catalina")]);

        assert_eq!(
            ThemeStatusProvider::theme_id_for("MON-A", Some(&config)),
            Some("Big Sur".to_string())
        );
        assert_eq!(
            ThemeStatusProvider::theme_id_for("MON-B", Some(&config)),
            Some("Catalina".to_string())
        );
    }

    #[test]
    fn a_monitor_without_a_configured_theme_has_none() {
        let config = config_with_monitors(&[("MON-A", "Big Sur")]);

        assert_eq!(
            ThemeStatusProvider::theme_id_for("MON-B", Some(&config)),
            None
        );
    }

    /// The aggregate answers only when there is one answer to give.
    #[test]
    fn all_monitors_agreeing_returns_that_theme() {
        let config = config_with_monitors(&[("MON-A", "Big Sur"), ("MON-B", "Big Sur")]);

        assert_eq!(
            ThemeStatusProvider::theme_id_for("all", Some(&config)),
            Some("Big Sur".to_string())
        );
    }

    #[test]
    fn all_disagreeing_monitors_return_nothing() {
        let config = config_with_monitors(&[("MON-A", "Big Sur"), ("MON-B", "Catalina")]);

        assert_eq!(
            ThemeStatusProvider::theme_id_for("all", Some(&config)),
            None
        );
    }

    #[test]
    fn all_returns_the_theme_of_the_all_variant() {
        let config = config_with_one_theme("Monterey Bay 1");

        assert_eq!(
            ThemeStatusProvider::theme_id_for("all", Some(&config)),
            Some("Monterey Bay 1".to_string())
        );
        // The same variant answers for a named monitor, which is what makes the
        // stage's per-monitor lookup agree with its aggregate.
        assert_eq!(
            ThemeStatusProvider::theme_id_for("MON-A", Some(&config)),
            Some("Monterey Bay 1".to_string())
        );
    }

    #[test]
    fn without_a_configuration_there_is_no_theme() {
        assert_eq!(ThemeStatusProvider::theme_id_for("all", None), None);
        assert_eq!(ThemeStatusProvider::theme_id_for("MON-A", None), None);
    }

    #[test]
    fn a_configuration_without_a_monitor_map_has_no_theme() {
        let config: Config = serde_json::from_str("{}").unwrap();

        assert_eq!(
            ThemeStatusProvider::theme_id_for("all", Some(&config)),
            None
        );
        assert_eq!(
            ThemeStatusProvider::theme_id_for("MON-A", Some(&config)),
            None
        );
    }
}
