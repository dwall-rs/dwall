//! Theme update engine: applies wallpapers and color scheme for a single cycle.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use time::OffsetDateTime;

use crate::DwallResult;
use crate::color_scheme::ColorSchemeApplier;
use crate::config::{Config, MonitorSpecificWallpapers};
use crate::monitor::MonitorProvider;
use crate::platform::{
    ColorSchemeScheduler, DisplayMonitor, DisplayMonitorProvider, Positioner, WallpaperSetter,
};
use crate::solar::{GeographicPositionProvider, Position, SolarPosition};
use crate::theme::{
    ThemeError, get_theme_directory_path, load_cached_theme_data, wallpaper_image_path,
};
use crate::wallpaper::{WallpaperProvider, WallpaperSelector};

/// Orchestrates the solar-based theme update cycle.
pub(crate) struct ThemeEngine<'a> {
    config: &'a Config,
    position_provider: GeographicPositionProvider<'a, Positioner>,
    wallpaper_setter: WallpaperSetter,
    /// Shared with [`WallpaperSetter`], so both see one snapshot of the displays.
    monitor_provider: Rc<DisplayMonitorProvider>,
    color_scheme_applier: ColorSchemeApplier<ColorSchemeScheduler>,
}

impl<'a> ThemeEngine<'a> {
    pub(crate) fn new(
        config: &'a Config,
        wallpaper_setter: WallpaperSetter,
        monitor_provider: Rc<DisplayMonitorProvider>,
        color_scheme_provider: ColorSchemeScheduler,
        position_provider: GeographicPositionProvider<'a, Positioner>,
    ) -> Self {
        info!(
            auto_detect_color_mode = config.auto_detect_color_scheme(),
            image_format = ?config.image_format(),
            update_interval_seconds = config.interval(),
            "Initializing theme engine"
        );

        Self {
            config,
            position_provider,
            wallpaper_setter,
            monitor_provider,
            color_scheme_applier: ColorSchemeApplier::new(color_scheme_provider),
        }
    }

    /// Returns the update interval in seconds
    pub(crate) fn update_interval(&self) -> u16 {
        self.config.interval()
    }

    /// Runs a single theme update cycle
    pub(crate) fn run_once(&self) -> DwallResult<()> {
        let position = self.position_provider.get_current_position()?;
        self.process_theme_cycle(&position)
    }

    /// Checks if monitor configuration has changed and reloads if necessary
    pub(crate) fn reload_if_monitors_changed(&self) -> bool {
        let changed = match self.monitor_provider.has_configuration_changed() {
            Ok(changed) => changed,
            Err(e) => {
                // The only place a monitor-enumeration failure used to disappear
                // without a trace.
                warn!(
                    error = %e,
                    "Failed to check the display configuration, keeping the current one"
                );
                false
            }
        };

        if changed {
            info!("Monitor configuration changed, reapplying wallpapers");
            if let Ok(position) = self.position_provider.get_current_position()
                && let Err(e) = self.process_theme_cycle(&position)
            {
                error!(error = %e, "Failed to reapply after monitor change");
            }
        }

        changed
    }

    /// Process theme cycle for the current geographic position
    pub(crate) fn process_theme_cycle(&self, position: &Position) -> DwallResult<()> {
        let now_local = OffsetDateTime::now_local()?;
        let now = now_local.assume_utc();
        let solar_position = SolarPosition::new(position, &now);
        let available_monitors = self.monitor_provider.get_monitors()?;

        let outcome = apply_monitor_targets(
            monitor_targets(
                self.config.monitor_specific_wallpapers(),
                &available_monitors,
            ),
            |monitor_id, theme_id| self.process_single(monitor_id, theme_id, &solar_position),
        );

        if outcome.applied > 0
            && let Some(theme_id) = self.get_first_configured_theme()
            && let Err(e) = self.process_lock_screen(&solar_position, &theme_id)
        {
            warn!(error = %e, theme_id = theme_id, "Failed to apply lock screen wallpaper");
        }

        // Runs even when no wallpaper could be applied: the light/dark switch is
        // driven by the sun and has nothing to do with a broken theme.
        self.color_scheme_applier.update_color_scheme(
            self.config,
            &now_local,
            position,
            &solar_position,
        )?;

        if outcome.applied == 0 && outcome.failed == 0 && !available_monitors.is_empty() {
            // Nothing was configured for these monitors, which is not a failure:
            // a default configuration has no monitor map at all, and `dwall --once`
            // has to keep exiting 0 there.
            warn!(
                total_monitors = available_monitors.len(),
                "No monitor received a wallpaper in this cycle"
            );
        }

        debug!(
            successful_monitors = outcome.applied,
            failed_monitors = outcome.failed,
            total_monitors = available_monitors.len(),
            "Theme cycle completed"
        );

        // Fail the cycle only now that everything else has run, so the caller's
        // backoff and the log entry stand for "the wallpapers could not be
        // applied" rather than "the whole update failed".
        if outcome.every_configured_monitor_failed() {
            return Err(ThemeError::AllConfiguredMonitorsFailed {
                failed: outcome.failed,
            }
            .into());
        }

        Ok(())
    }

    fn process_single(
        &self,
        monitor_id: &str,
        theme_id: &str,
        solar_position: &SolarPosition,
    ) -> DwallResult<()> {
        let (theme_dir, is_customized) = get_theme_directory_path(self.config, theme_id);

        debug!(
            theme_dir = %theme_dir.display(),
            is_customized = is_customized,
            theme_id = theme_id,
            "Using theme directory"
        );

        let image_path =
            self.select_wallpaper_path(&theme_dir, solar_position, is_customized, theme_id)?;

        self.wallpaper_setter
            .set_monitor_wallpaper(monitor_id, &image_path)?;

        debug!(
            monitor_id = monitor_id,
            wallpaper_path = %image_path.display(),
            "Successfully applied solar wallpaper to monitor"
        );

        Ok(())
    }

    /// Applies the lock screen wallpaper if enabled
    fn process_lock_screen(
        &self,
        solar_position: &SolarPosition,
        lock_screen_theme_id: &str,
    ) -> DwallResult<()> {
        if !self.config.lock_screen_wallpaper_enabled() {
            return Ok(());
        }

        let (theme_dir, is_customized) =
            get_theme_directory_path(self.config, lock_screen_theme_id);

        let image_path = self.select_wallpaper_path(
            &theme_dir,
            solar_position,
            is_customized,
            lock_screen_theme_id,
        )?;

        self.wallpaper_setter.set_lock_screen_image(&image_path)?;

        Ok(())
    }

    /// Selects the wallpaper image path matching the current solar position
    fn select_wallpaper_path(
        &self,
        theme_dir: &Path,
        solar_position: &SolarPosition,
        is_customized: bool,
        theme_id: &str,
    ) -> DwallResult<PathBuf> {
        let theme_data = load_cached_theme_data(theme_dir)?;
        let sun_altitude_degrees = solar_position.altitude();
        let sun_azimuth_degrees = solar_position.azimuth();

        debug!(
            sun_altitude = sun_altitude_degrees,
            sun_azimuth = sun_azimuth_degrees,
            theme_directory = %theme_dir.display(),
            "Calculated current solar position for wallpaper selection"
        );

        let optimal_image_index = WallpaperSelector::find_closest_image(
            theme_data.angles.as_slice(),
            sun_altitude_degrees,
            sun_azimuth_degrees,
        )
        .ok_or(ThemeError::ImageSolarConfigurationMismatch {
            expected: theme_data.angles.len(),
            found: 0,
        })?;

        // Custom themes declare their own image format; legacy themes use the config's.
        let image_format = theme_data
            .image_format
            .clone()
            .unwrap_or_else(|| self.config.image_format().clone());

        let image_path =
            wallpaper_image_path(theme_dir, optimal_image_index, &image_format, is_customized);

        if !image_path.exists() {
            return Err(ThemeError::WallpaperImageMissing {
                path: image_path.display().to_string(),
            }
            .into());
        }

        debug!(
            wallpaper_path = %image_path.display(),
            image_index = optimal_image_index,
            theme_id = theme_id,
            "Selected optimal solar wallpaper"
        );

        Ok(image_path)
    }

    fn get_first_configured_theme(&self) -> Option<String> {
        match self.config.monitor_specific_wallpapers() {
            Some(MonitorSpecificWallpapers::All(theme_id)) => Some(theme_id.clone()),
            Some(MonitorSpecificWallpapers::Individual(map)) => map.values().next().cloned(),
            None => None,
        }
    }
}

/// What one pass over the configured monitors did.
#[derive(Debug, Default, PartialEq, Eq)]
struct MonitorPassOutcome {
    /// Monitors that were given their wallpaper.
    applied: usize,
    /// Configured monitors whose wallpaper could not be applied.
    failed: usize,
}

impl MonitorPassOutcome {
    /// Whether nothing that was configured succeeded.
    ///
    /// Deliberately not "nothing was applied": a configuration that names no
    /// monitor has nothing to do and is not a failure, which is what keeps
    /// `dwall --once` on a default configuration exiting 0.
    fn every_configured_monitor_failed(&self) -> bool {
        self.applied == 0 && self.failed > 0
    }
}

/// The (monitor, theme) pairs a cycle should apply, in monitor order.
fn monitor_targets<'a>(
    configs: Option<&'a MonitorSpecificWallpapers>,
    monitors: &'a HashMap<String, DisplayMonitor>,
) -> impl Iterator<Item = (&'a str, &'a str)> + 'a {
    monitors.keys().filter_map(move |monitor_id| {
        let Some(theme_id) = configs.and_then(|configs| configs.get(monitor_id)) else {
            debug!(monitor_id = monitor_id, "No theme config, skipping");
            return None;
        };

        debug!(
            monitor_id = monitor_id,
            theme_id = theme_id,
            "Processing wallpaper for monitor"
        );

        Some((monitor_id.as_str(), theme_id.as_str()))
    })
}

/// Applies `apply` to every target, counting successes and failures.
fn apply_monitor_targets<'a, I, F>(targets: I, mut apply: F) -> MonitorPassOutcome
where
    I: IntoIterator<Item = (&'a str, &'a str)>,
    F: FnMut(&str, &str) -> DwallResult<()>,
{
    let mut outcome = MonitorPassOutcome::default();

    for (monitor_id, theme_id) in targets {
        match apply(monitor_id, theme_id) {
            Ok(()) => outcome.applied += 1,
            // Surface the reason: without this the only symptom of a broken theme
            // is that the wallpaper stops changing.
            Err(e) => {
                warn!(
                    error = %e,
                    monitor_id = monitor_id,
                    theme_id = theme_id,
                    "Failed to apply wallpaper for monitor"
                );
                outcome.failed += 1;
            }
        }
    }

    outcome
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{MonitorSpecificWallpapers, WallpaperMode};

    fn monitors(ids: &[&str]) -> HashMap<String, DisplayMonitor> {
        ids.iter()
            .enumerate()
            .map(|(index, id)| {
                (
                    (*id).to_string(),
                    DisplayMonitor::new_test(
                        (*id).to_string(),
                        format!("Display {}", index + 1),
                        Some(index as u32),
                    ),
                )
            })
            .collect()
    }

    fn config_with(mode: WallpaperMode) -> Config {
        let mut config = Config::default();
        config.set_wallpaper_mode(mode);

        config
    }

    fn individual(pairs: &[(&str, &str)]) -> WallpaperMode {
        WallpaperMode::Fixed {
            monitor_specific_wallpapers: MonitorSpecificWallpapers::Individual(
                pairs
                    .iter()
                    .map(|(monitor, theme)| ((*monitor).to_string(), (*theme).to_string()))
                    .collect(),
            ),
        }
    }

    fn targets(
        config: &Config,
        monitors: &HashMap<String, DisplayMonitor>,
    ) -> Vec<(String, String)> {
        monitor_targets(config.monitor_specific_wallpapers(), monitors)
            .map(|(monitor, theme)| (monitor.to_string(), theme.to_string()))
            .collect()
    }

    #[test]
    fn a_monitor_without_a_configured_theme_is_not_a_target() {
        let config = config_with(individual(&[("MON-A", "Big Sur")]));
        let monitors = monitors(&["MON-A", "MON-B"]);

        assert_eq!(
            targets(&config, &monitors),
            vec![("MON-A".to_string(), "Big Sur".to_string())]
        );
    }

    #[test]
    fn every_monitor_is_a_target_of_the_all_variant() {
        let config = config_with(WallpaperMode::Fixed {
            monitor_specific_wallpapers: MonitorSpecificWallpapers::All("Catalina".to_string()),
        });
        let monitors = monitors(&["MON-A", "MON-B"]);

        let mut targets = targets(&config, &monitors);
        targets.sort();

        assert_eq!(
            targets,
            vec![
                ("MON-A".to_string(), "Catalina".to_string()),
                ("MON-B".to_string(), "Catalina".to_string()),
            ]
        );
    }

    #[test]
    fn a_configuration_without_a_monitor_map_has_no_targets() {
        let config = Config::default();
        let monitors = monitors(&["MON-A"]);

        assert!(targets(&config, &monitors).is_empty());
    }

    #[test]
    fn every_configured_monitor_failure_is_counted() {
        let outcome = apply_monitor_targets([("MON-A", "Broken"), ("MON-B", "Broken")], |_, _| {
            Err(crate::error::DwallError::ThemeProcessing(
                ThemeError::ThemeDirectoryNotFound("Broken".to_string()),
            ))
        });

        assert_eq!(
            outcome,
            MonitorPassOutcome {
                applied: 0,
                failed: 2
            }
        );
        assert!(outcome.every_configured_monitor_failed());
    }

    #[test]
    fn one_successful_monitor_keeps_the_cycle_from_failing() {
        let outcome =
            apply_monitor_targets([("MON-A", "Broken"), ("MON-B", "Fine")], |_, theme| {
                if theme == "Fine" {
                    Ok(())
                } else {
                    Err(crate::error::DwallError::ThemeProcessing(
                        ThemeError::ThemeDirectoryNotFound(theme.to_string()),
                    ))
                }
            });

        assert_eq!(
            outcome,
            MonitorPassOutcome {
                applied: 1,
                failed: 1
            }
        );
        assert!(!outcome.every_configured_monitor_failed());
    }

    #[test]
    fn nothing_to_do_is_not_a_failure() {
        // An empty monitor map, or a configuration that names none, must not fail
        // the cycle: `dwall --once` exits 0 in both cases.
        let empty = apply_monitor_targets([], |_, _| Ok(()));
        assert_eq!(empty, MonitorPassOutcome::default());
        assert!(!empty.every_configured_monitor_failed());

        let applied = apply_monitor_targets([("MON-A", "Big Sur")], |_, _| Ok(()));
        assert_eq!(
            applied,
            MonitorPassOutcome {
                applied: 1,
                failed: 0
            }
        );
        assert!(!applied.every_configured_monitor_failed());
    }
}
