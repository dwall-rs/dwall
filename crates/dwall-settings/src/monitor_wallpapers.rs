//! The fixed wallpaper mode's monitor → theme mapping.
//!
//! The configuration stores that mapping in one of two shapes: a single theme for
//! every monitor, or a map with one entry per monitor. The renderer works with the
//! resolved form instead, and this module is the only place that knows both shapes
//! and the "all monitors" pseudo-scope.

use std::collections::HashMap;

use dwall::config::{MonitorSpecificWallpapers, WallpaperMode};
use serde::{Deserialize, Serialize};

/// The "all monitors" scope. It is a UI pseudo-monitor, never a device path, and it is
/// not stored as an individual entry.
pub const ALL_MONITORS: &str = "all";

/// One scope and the theme it uses.
#[cfg_attr(feature = "typegen", derive(ts_rs::TS))]
#[cfg_attr(feature = "typegen", ts(export))]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MonitorTheme {
    /// A monitor's device path, or [`ALL_MONITORS`].
    pub monitor_id: String,
    pub theme_id: String,
}

/// A requested change to the resolved mapping.
#[cfg_attr(feature = "typegen", derive(ts_rs::TS))]
#[cfg_attr(feature = "typegen", ts(export))]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MonitorThemeChange {
    /// The scope being applied or stopped.
    pub monitor_id: String,
    pub theme_id: String,
    /// Whether the change concerns the "all monitors" scope.
    pub unified: bool,
    /// Whether the scope is currently applied; an applied scope is stopped instead.
    pub applied: bool,
}

/// The resolved mapping, plus the mode a requested change produces.
#[cfg_attr(feature = "typegen", derive(ts_rs::TS))]
#[cfg_attr(feature = "typegen", ts(export))]
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MonitorThemes {
    /// Whether the mode stores one theme for every monitor.
    pub unified: bool,
    /// The effective theme per scope: [`ALL_MONITORS`] first, then the configured
    /// entries in their stored order. A monitor without an entry is absent.
    pub themes: Vec<MonitorTheme>,
    /// The scopes the configuration stores. [`ALL_MONITORS`] is absent from an
    /// individual map: there it is a representative, not a stored entry.
    pub configured: Vec<String>,
    /// The mode to write: the current one, or the one the requested change produces.
    pub mode: WallpaperMode,
}

/// Resolve the fixed-mode wallpapers, optionally applying a requested change.
pub fn resolve(
    mode: &WallpaperMode,
    monitors: &[String],
    change: Option<&MonitorThemeChange>,
) -> MonitorThemes {
    let mode = match change {
        Some(change) => apply_change(mode, change),
        None => mode.clone(),
    };

    let (unified, themes, configured) = match &mode {
        WallpaperMode::Fixed {
            monitor_specific_wallpapers,
        } => resolve_wallpapers(monitor_specific_wallpapers, monitors),
        // Random mode has no monitor mapping. A caller reaches that branch only while
        // reading a random configuration; a change above has already made it fixed.
        WallpaperMode::Random { .. } => (false, Vec::new(), Vec::new()),
    };

    MonitorThemes {
        unified,
        themes,
        configured,
        mode,
    }
}

/// The effective scope → theme list for one shape of the configuration.
fn resolve_wallpapers(
    wallpapers: &MonitorSpecificWallpapers,
    monitors: &[String],
) -> (bool, Vec<MonitorTheme>, Vec<String>) {
    match wallpapers {
        MonitorSpecificWallpapers::All(theme_id) => {
            let mut themes = Vec::with_capacity(monitors.len() + 1);
            themes.push(MonitorTheme {
                monitor_id: ALL_MONITORS.to_string(),
                theme_id: theme_id.clone(),
            });
            themes.extend(monitors.iter().map(|monitor_id| MonitorTheme {
                monitor_id: monitor_id.clone(),
                theme_id: theme_id.clone(),
            }));

            let configured = std::iter::once(ALL_MONITORS.to_string())
                .chain(monitors.iter().cloned())
                .collect::<Vec<_>>();

            (true, themes, configured)
        }
        MonitorSpecificWallpapers::Individual(entries) => {
            let mut themes = Vec::with_capacity(entries.len() + 1);

            // An individual map has no single theme, so the "all monitors" scope shows
            // the first stored entry — the rule the stage has always used.
            if !entries.contains_key(ALL_MONITORS)
                && let Some(theme_id) = entries.values().next()
            {
                themes.push(MonitorTheme {
                    monitor_id: ALL_MONITORS.to_string(),
                    theme_id: theme_id.clone(),
                });
            }

            themes.extend(entries.iter().map(|(monitor_id, theme_id)| MonitorTheme {
                monitor_id: monitor_id.clone(),
                theme_id: theme_id.clone(),
            }));

            let configured = entries.keys().cloned().collect::<Vec<_>>();

            (false, themes, configured)
        }
    }
}

/// The mode a requested apply/stop writes.
fn apply_change(mode: &WallpaperMode, change: &MonitorThemeChange) -> WallpaperMode {
    if change.unified {
        // The unified scope is not one entry among others: applying it stores the
        // single-theme variant, and stopping it clears the mapping.
        let wallpapers = if change.applied {
            MonitorSpecificWallpapers::Individual(HashMap::new())
        } else {
            MonitorSpecificWallpapers::All(change.theme_id.clone())
        };

        return WallpaperMode::Fixed {
            monitor_specific_wallpapers: wallpapers,
        };
    }

    let mut entries = match mode {
        WallpaperMode::Fixed {
            monitor_specific_wallpapers: MonitorSpecificWallpapers::Individual(entries),
        } => entries.clone(),
        _ => HashMap::new(),
    };

    if change.applied {
        entries.remove(&change.monitor_id);
    } else {
        entries.insert(change.monitor_id.clone(), change.theme_id.clone());
    }

    WallpaperMode::Fixed {
        monitor_specific_wallpapers: MonitorSpecificWallpapers::Individual(entries),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn monitors(ids: &[&str]) -> Vec<String> {
        ids.iter().map(|id| (*id).to_string()).collect()
    }

    fn individual(entries: &[(&str, &str)]) -> MonitorSpecificWallpapers {
        MonitorSpecificWallpapers::Individual(
            entries
                .iter()
                .map(|(monitor, theme)| ((*monitor).to_string(), (*theme).to_string()))
                .collect(),
        )
    }

    fn fixed(wallpapers: MonitorSpecificWallpapers) -> WallpaperMode {
        WallpaperMode::Fixed {
            monitor_specific_wallpapers: wallpapers,
        }
    }

    fn theme(monitor: &str, theme_id: &str) -> MonitorTheme {
        MonitorTheme {
            monitor_id: monitor.to_string(),
            theme_id: theme_id.to_string(),
        }
    }

    fn change(monitor: &str, theme_id: &str, unified: bool, applied: bool) -> MonitorThemeChange {
        MonitorThemeChange {
            monitor_id: monitor.to_string(),
            theme_id: theme_id.to_string(),
            unified,
            applied,
        }
    }

    #[test]
    fn all_expands_to_every_monitor_and_the_pseudo_scope() {
        let mode = fixed(MonitorSpecificWallpapers::All("Sunrise".to_string()));

        let resolved = resolve(&mode, &monitors(&["MONITOR#1", "MONITOR#2"]), None);

        assert!(resolved.unified);
        assert_eq!(
            resolved.themes,
            vec![
                theme(ALL_MONITORS, "Sunrise"),
                theme("MONITOR#1", "Sunrise"),
                theme("MONITOR#2", "Sunrise"),
            ]
        );
        assert_eq!(
            resolved.configured,
            vec![
                ALL_MONITORS.to_string(),
                "MONITOR#1".to_string(),
                "MONITOR#2".to_string(),
            ]
        );
    }

    #[test]
    fn a_monitor_without_an_entry_is_absent() {
        let mode = fixed(individual(&[("MONITOR#1", "Catalina")]));

        let resolved = resolve(&mode, &monitors(&["MONITOR#1", "MONITOR#2"]), None);

        assert!(!resolved.unified);
        assert_eq!(
            resolved.themes,
            vec![
                theme(ALL_MONITORS, "Catalina"),
                theme("MONITOR#1", "Catalina"),
            ],
            "the unconfigured monitor must not gain a theme"
        );
        assert_eq!(resolved.configured, vec!["MONITOR#1".to_string()]);
    }

    #[test]
    fn an_empty_individual_map_resolves_to_nothing() {
        let mode = fixed(individual(&[]));

        let resolved = resolve(&mode, &monitors(&["MONITOR#1"]), None);

        assert!(!resolved.unified);
        assert!(resolved.themes.is_empty());
        assert!(resolved.configured.is_empty());
    }

    #[test]
    fn the_all_scope_representative_is_the_first_stored_entry() {
        let wallpapers = individual(&[("MONITOR#1", "Catalina"), ("MONITOR#2", "Big Sur")]);
        let first = match &wallpapers {
            MonitorSpecificWallpapers::Individual(entries) => entries
                .iter()
                .next()
                .map(|(monitor, theme_id)| (monitor.clone(), theme_id.clone()))
                .unwrap(),
            MonitorSpecificWallpapers::All(_) => unreachable!(),
        };

        let resolved = resolve(&fixed(wallpapers.clone()), &[], None);

        assert_eq!(
            resolved.themes.first(),
            Some(&theme(ALL_MONITORS, &first.1))
        );
    }

    #[test]
    fn stored_entries_keep_their_order() {
        let wallpapers = individual(&[
            ("MONITOR#1", "Catalina"),
            ("MONITOR#2", "Big Sur"),
            ("MONITOR#3", "Mojave"),
        ]);
        let entries = match &wallpapers {
            MonitorSpecificWallpapers::Individual(entries) => entries,
            MonitorSpecificWallpapers::All(_) => unreachable!(),
        };

        let resolved = resolve(&fixed(wallpapers.clone()), &[], None);

        let expected = entries
            .iter()
            .map(|(monitor, theme_id)| theme(monitor, theme_id))
            .collect::<Vec<_>>();
        assert_eq!(&resolved.themes[1..], expected.as_slice());
    }

    #[test]
    fn applying_to_all_monitors_stores_the_shared_theme() {
        let resolved = resolve(
            &fixed(individual(&[("MONITOR#1", "Catalina")])),
            &monitors(&["MONITOR#1", "MONITOR#2"]),
            Some(&change(ALL_MONITORS, "Sunrise", true, false)),
        );

        assert_eq!(
            resolved.mode,
            fixed(MonitorSpecificWallpapers::All("Sunrise".to_string()))
        );
        assert!(resolved.unified);
        assert_eq!(
            resolved.themes,
            vec![
                theme(ALL_MONITORS, "Sunrise"),
                theme("MONITOR#1", "Sunrise"),
                theme("MONITOR#2", "Sunrise"),
            ]
        );
    }

    #[test]
    fn stopping_all_monitors_clears_the_mapping() {
        let resolved = resolve(
            &fixed(MonitorSpecificWallpapers::All("Sunrise".to_string())),
            &monitors(&["MONITOR#1"]),
            Some(&change(ALL_MONITORS, "Sunrise", true, true)),
        );

        assert_eq!(resolved.mode, fixed(individual(&[])));
        assert!(!resolved.unified);
        assert!(resolved.themes.is_empty());
    }

    #[test]
    fn applying_a_monitor_adds_only_its_entry() {
        let resolved = resolve(
            &fixed(individual(&[("MONITOR#1", "Catalina")])),
            &monitors(&["MONITOR#1", "MONITOR#2"]),
            Some(&change("MONITOR#2", "Big Sur", false, false)),
        );

        assert_eq!(
            resolved.mode,
            fixed(individual(&[
                ("MONITOR#1", "Catalina"),
                ("MONITOR#2", "Big Sur"),
            ]))
        );
        assert_eq!(
            resolved.themes,
            vec![
                theme(ALL_MONITORS, "Catalina"),
                theme("MONITOR#1", "Catalina"),
                theme("MONITOR#2", "Big Sur"),
            ]
        );
    }

    #[test]
    fn stopping_a_monitor_removes_only_its_entry() {
        let resolved = resolve(
            &fixed(individual(&[
                ("MONITOR#1", "Catalina"),
                ("MONITOR#2", "Big Sur"),
            ])),
            &monitors(&["MONITOR#1", "MONITOR#2"]),
            Some(&change("MONITOR#2", "Big Sur", false, true)),
        );

        assert_eq!(
            resolved.mode,
            fixed(individual(&[("MONITOR#1", "Catalina")]))
        );
        assert_eq!(
            resolved.themes,
            vec![
                theme(ALL_MONITORS, "Catalina"),
                theme("MONITOR#1", "Catalina"),
            ]
        );
    }

    #[test]
    fn a_change_from_random_mode_starts_a_fixed_mapping() {
        let resolved = resolve(
            &WallpaperMode::Random {
                pool: Some(vec!["Sunrise".to_string()]),
            },
            &monitors(&["MONITOR#1"]),
            Some(&change(ALL_MONITORS, "Sunrise", true, false)),
        );

        assert_eq!(
            resolved.mode,
            fixed(MonitorSpecificWallpapers::All("Sunrise".to_string()))
        );
        assert!(resolved.unified);
    }

    #[test]
    fn the_all_scope_uses_the_single_theme_when_the_monitor_list_is_empty() {
        let resolved = resolve(
            &fixed(MonitorSpecificWallpapers::All("Sunrise".to_string())),
            &[],
            None,
        );

        assert_eq!(resolved.themes, vec![theme(ALL_MONITORS, "Sunrise")]);
        assert_eq!(resolved.configured, vec![ALL_MONITORS.to_string()]);
    }

    #[test]
    fn reading_a_random_mode_has_no_monitor_mapping() {
        let mode = WallpaperMode::Random { pool: None };

        let resolved = resolve(&mode, &monitors(&["MONITOR#1"]), None);

        assert!(!resolved.unified);
        assert!(resolved.themes.is_empty());
        assert!(resolved.configured.is_empty());
        assert_eq!(resolved.mode, mode);
    }
}
