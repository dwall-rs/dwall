//! Daemon process: watches the config, runs theme update cycles and handles
//! random wallpaper mode.

use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::thread::sleep;
use std::time::{Duration, Instant};

use time::{Date, OffsetDateTime};

use crate::DwallResult;
use crate::config::{
    Config, ConfigReader, ConfigWatcher, ConfigWriter, MonitorSpecificWallpapers, WallpaperMode,
    config_path,
};
use crate::error::DwallError;
use crate::lazy::DWALL_CONFIG_DIR;
use crate::platform::{ColorSchemeScheduler, DisplayMonitorProvider, Positioner, WallpaperSetter};
use crate::solar::GeographicPositionProvider;
use crate::theme::{DailyRandomThemeSelector, filter_themes_by_pool, is_theme_directory};
use crate::theme_engine::ThemeEngine;

/// Backoff before retrying a cycle that failed.
const FAILURE_BACKOFF_BASE: Duration = Duration::from_secs(5);

/// Ceiling for the failure backoff.
///
/// A condition that never clears (location access denied, the location service
/// off, a wedged shell) must not turn into an ever-widening gap, so the wait
/// stops growing here and the daemon keeps trying every few minutes.
const FAILURE_BACKOFF_MAX: Duration = Duration::from_secs(300);

/// Backoff before retrying a configuration read that failed.
const CONFIG_RETRY_DELAY: Duration = Duration::from_secs(5);

/// How long the daemon sleeps before re-checking the configuration file.
///
/// The file used to be checked once per cycle, at the top of the loop, so a setting
/// saved in the window waited out the whole interval before the daemon even looked
/// at it: the machine this was measured on runs `interval = 600`, i.e. up to ten
/// minutes. Only "apply theme" restarts the daemon, so every other setting depended
/// on that check.
///
/// Five seconds rather than one: the wait is also what the schedule uses, so a shorter
/// slice is a wakeup the daemon does not need — reading a file's modification time per
/// second kept every configuration awake at 1 Hz no matter how rarely the wallpapers
/// changed. Saving a setting still takes effect well within the time it takes to look
/// at the window.
const CONFIG_POLL_INTERVAL: Duration = Duration::from_secs(5);

/// Waits up to `duration`, returning early when the configuration file changed.
///
/// The wait is sliced so the file is re-checked every [`CONFIG_POLL_INTERVAL`]
/// instead of once per cycle; a metadata call per second is free next to the
/// wallpaper updates the cycle performs.
fn wait_for_config_change(config_watcher: &ConfigWatcher, duration: Duration) -> bool {
    let deadline = Instant::now() + duration;

    while let Some(remaining) = deadline.checked_duration_since(Instant::now()) {
        sleep(CONFIG_POLL_INTERVAL.min(remaining));

        match config_watcher.has_changed() {
            Ok(true) => {
                info!("Configuration file change detected while waiting, reloading configuration");
                return true;
            }
            Ok(false) => {}
            Err(e) => {
                // A transient metadata failure must not end the wait.
                warn!(
                    error = %e,
                    path = %config_watcher.config_path().display(),
                    "Failed to check the configuration file while waiting"
                );
            }
        }
    }

    false
}

/// Follows a run of failed update cycles and decides when to try again.
///
/// A failed cycle used to terminate the daemon once it had repeated three times.
/// Every realistic cause of that failure is persistent rather than fatal —
/// a shell that stopped answering, a location service that is still starting after
/// sign-in — so exiting converted "the wallpaper is not updating right now" into "the
/// wallpaper never updates again until someone starts the daemon by hand", with no
/// supervisor to do it. Cycles therefore keep running: the failure count only drives
/// the backoff and the log.
///
/// The one exception is [`DwallError::location_unavailable`]: there the position is not
/// missing by accident but switched off by the user, and the daemon deliberately stops
/// instead of guessing where it is. See [`cycle_failure`].
#[derive(Debug, Default)]
struct CycleFailureTracker {
    consecutive: u32,
}

impl CycleFailureTracker {
    /// Records a successful cycle, returning to the configured cadence.
    fn record_success(&mut self) {
        self.consecutive = 0;
    }

    /// Records a failed cycle and returns how long to wait before the next one.
    fn record_failure(&mut self) -> Duration {
        self.consecutive = self.consecutive.saturating_add(1);
        failure_backoff(self.consecutive)
    }

    /// How many cycles have failed in a row.
    fn consecutive_failures(&self) -> u32 {
        self.consecutive
    }
}

/// Exponential backoff for `consecutive_failures` (1-based), capped at
/// [`FAILURE_BACKOFF_MAX`].
fn failure_backoff(consecutive_failures: u32) -> Duration {
    // Cap the exponent before shifting so a run of failures that lasts for days
    // cannot overflow the shift or the multiplication.
    let doublings = consecutive_failures.saturating_sub(1).min(16);

    FAILURE_BACKOFF_BASE
        .saturating_mul(1u32 << doublings)
        .min(FAILURE_BACKOFF_MAX)
}

/// The engine wired to the Windows backends, as one name.
///
/// [`ThemeEngine`] is generic so its cycle can be tested against doubles; this is
/// the only place the production providers are named.
type PlatformEngine<'a> = ThemeEngine<
    'a,
    DisplayMonitorProvider,
    WallpaperSetter,
    ColorSchemeScheduler,
    GeographicPositionProvider<'a, Positioner>,
>;

/// What the loop does with a failed cycle.
#[derive(Debug, PartialEq, Eq)]
enum CycleFailure {
    /// Retry after the backoff: the failure is transient or environmental.
    Retry(Duration),
    /// Stop the daemon and let the caller report why.
    Stop,
}

/// Classifies a failed cycle.
///
/// A missing location is the one failure the daemon does not retry: the user either
/// turned the location service off, denied it for this app, or is on manual coordinates
/// that failed to parse — in every case there is no position to fall back on, and
/// guessing one (the last position it saw) would silently follow the wrong sun. Stopping
/// with the error, the exit code and the log line is the honest answer, and the settings
/// window can start the daemon again once the setting changes.
fn cycle_failure(error: &DwallError, backoff: Duration) -> CycleFailure {
    if error.location_unavailable() {
        return CycleFailure::Stop;
    }

    CycleFailure::Retry(backoff)
}

/// Builds the engine for one configuration generation.
fn build_engine(config: &Config) -> DwallResult<PlatformEngine<'_>> {
    // One monitor provider, shared with the wallpaper setter: two independent caches
    // meant the setter could still hold a pre-hot-plug snapshot after the engine had
    // refreshed its own, and then fail to find the monitor it was asked for.
    let monitor_provider = Rc::new(DisplayMonitorProvider::new());
    let wallpaper_setter =
        WallpaperSetter::new(Rc::clone(&monitor_provider)).map_err(DwallError::WallpaperManager)?;
    let color_scheme_backend = ColorSchemeScheduler::new();
    let position_provider =
        GeographicPositionProvider::new(config.position_source(), Positioner::new());

    Ok(ThemeEngine::new(
        config,
        wallpaper_setter,
        monitor_provider,
        color_scheme_backend,
        position_provider,
    ))
}

/// Runtime state for Random wallpaper mode
struct RandomModeState {
    selector: DailyRandomThemeSelector,
    current_theme: Option<String>,
}

/// The candidate pool configured for random mode (`None` = every installed theme).
fn random_pool(config: &Config) -> Option<&Vec<String>> {
    match config.wallpaper_mode() {
        WallpaperMode::Random { pool } => pool.as_ref(),
        WallpaperMode::Fixed { .. } => None,
    }
}

/// Main daemon application
pub struct DaemonApplication {
    config_watcher: ConfigWatcher,
    random_state: Option<RandomModeState>,
}

impl DaemonApplication {
    /// Creates a new daemon application instance using the default config file.
    pub fn new() -> Self {
        Self::with_config_path(config_path())
    }

    /// Creates a daemon application instance for a specific config file.
    ///
    /// `dwall --config <path>` uses this so a standalone install can keep several
    /// independent configurations; the random-mode state file follows the config
    /// file's directory.
    pub fn with_config_path(config_path: PathBuf) -> Self {
        let config_watcher = ConfigWatcher::new(config_path);

        Self {
            config_watcher,
            random_state: None,
        }
    }

    /// Runs the daemon application
    pub fn run(&mut self) -> DwallResult<()> {
        // Establish the baseline file stamp so the first loop iteration does
        // not report a change that never happened.
        if let Err(e) = self.config_watcher.update_stamp() {
            warn!(error = %e, "Failed to read the initial configuration file stamp");
        }

        loop {
            let mut config = match self.load_config() {
                Ok(config) => config,
                Err(e) => {
                    // A hand-edited or half-written file must not kill the daemon:
                    // keep running and retry, otherwise wallpapers silently stop
                    // updating until the user restarts it manually.
                    error!(
                        error = %e,
                        path = %self.config_watcher.config_path().display(),
                        retry_in_seconds = CONFIG_RETRY_DELAY.as_secs(),
                        "Failed to read configuration, retrying"
                    );
                    sleep(CONFIG_RETRY_DELAY);
                    continue;
                }
            };

            self.apply_wallpaper_mode(&mut config)?;

            let theme_engine = build_engine(&config)?;

            info!(
                update_interval_seconds = config.interval(),
                "Starting daemon with config change detection"
            );

            self.run_engine_loop(&theme_engine)?;
        }
    }

    /// Applies exactly one update cycle and returns.
    ///
    /// Backs `dwall --once`, which makes the daemon usable from a script, a
    /// scheduled task or a smoke test without leaving a process behind.
    pub fn run_once(&mut self) -> DwallResult<()> {
        let mut config = self.load_config()?;
        self.apply_wallpaper_mode(&mut config)?;

        build_engine(&config)?.run_once()?;

        Ok(())
    }

    /// Reads the configuration, creating a documented default file on first run.
    fn load_config(&self) -> DwallResult<Config> {
        let path = self.config_watcher.config_path();

        if !path.exists() {
            match ConfigWriter.write_default(path) {
                Ok(()) => info!(
                    path = %path.display(),
                    "Created a default configuration file"
                ),
                Err(e) => warn!(
                    error = %e,
                    path = %path.display(),
                    "Failed to create a default configuration file"
                ),
            }
        }

        ConfigReader::read_from_path(path)
    }

    /// Applies the mode-specific preparation before a cycle.
    fn apply_wallpaper_mode(&mut self, config: &mut Config) -> DwallResult<()> {
        match config.wallpaper_mode() {
            WallpaperMode::Random { .. } => self.handle_random_mode(config),
            WallpaperMode::Fixed { .. } => {
                // Fixed mode: clear random state if switching from random
                self.random_state = None;
                Ok(())
            }
        }
    }

    /// Directory holding the state file that accompanies the configuration.
    fn state_dir(&self) -> PathBuf {
        self.config_watcher
            .config_path()
            .parent()
            .map_or_else(|| DWALL_CONFIG_DIR.clone(), Path::to_path_buf)
    }

    /// Handle Random wallpaper mode: select today's theme and always carry it in
    /// the (in-memory) config so the engine can apply it.
    fn handle_random_mode(&mut self, config: &mut Config) -> DwallResult<()> {
        let today = OffsetDateTime::now_local()?.date();

        if self.random_state.is_none() {
            self.random_state = Some(RandomModeState {
                selector: DailyRandomThemeSelector::new(),
                current_theme: None,
            });
        }

        let needs_switch = self
            .random_state
            .as_ref()
            .is_some_and(|state| state.selector.needs_switch(&today));

        if needs_switch
            // A restart must not re-roll today's theme, so reuse the persisted draw
            // when it is still valid and fall back to a fresh draw otherwise.
            && !self.restore_persisted_selection(config, &today)?
        {
            self.select_random_theme(config, &today)?;
        }

        // Always re-apply today's selection. A same-day config reload (e.g. saving
        // any setting) re-reads `Random` from disk; without this, the engine would
        // see no `monitor_specific_wallpapers` and stop applying anything.
        if let Some(theme_id) = self
            .random_state
            .as_ref()
            .and_then(|state| state.current_theme.clone())
        {
            config.set_wallpaper_mode(WallpaperMode::Fixed {
                monitor_specific_wallpapers: MonitorSpecificWallpapers::All(theme_id),
            });
        }

        Ok(())
    }

    /// Reuses the persisted draw for `today` when the theme is still installed.
    ///
    /// Returns `true` when today's selection was restored.
    fn restore_persisted_selection(&mut self, config: &Config, today: &Date) -> DwallResult<bool> {
        let selection = match crate::random_state::read_from(&self.state_dir()) {
            Ok(Some(selection)) => selection,
            Ok(None) => return Ok(false),
            Err(e) => {
                warn!(error = %e, "Failed to read the persisted random selection");
                return Ok(false);
            }
        };

        if selection.date != crate::random_state::date_key(today) {
            return Ok(false);
        }

        let available = self.list_available_themes(config)?;
        let candidates = filter_themes_by_pool(&available, random_pool(config));

        let Some(state) = self.random_state.as_mut() else {
            return Ok(false);
        };

        if state
            .selector
            .restore(&selection.theme_id, &candidates, today)
        {
            info!(
                theme_id = %selection.theme_id,
                "Restored today's random theme selection"
            );
            state.current_theme = Some(selection.theme_id);
            Ok(true)
        } else {
            debug!(
                theme_id = %selection.theme_id,
                "Persisted random theme is no longer available"
            );
            Ok(false)
        }
    }

    /// Draw today's theme from the candidate pool (once per day).
    fn select_random_theme(&mut self, config: &Config, today: &Date) -> DwallResult<()> {
        let available = self.list_available_themes(config)?;
        if available.is_empty() {
            warn!("Random mode: no themes available to select");
            return Ok(());
        }

        let themes_to_select = filter_themes_by_pool(&available, random_pool(config));
        if themes_to_select.is_empty() {
            warn!("Random mode: candidate pool is empty; nothing to apply");
            return Ok(());
        }

        let state_dir = self.state_dir();
        let state = self.random_state.get_or_insert_with(|| RandomModeState {
            selector: DailyRandomThemeSelector::new(),
            current_theme: None,
        });

        if let Some(theme_id) = state.selector.select_next(&themes_to_select) {
            state.selector.mark_applied(today);
            state.current_theme = Some(theme_id.clone());

            if let Err(e) = crate::random_state::write_to(
                &state_dir,
                &crate::random_state::RandomSelection::new(today, theme_id.clone()),
            ) {
                warn!(error = %e, "Failed to persist random selection");
            }

            info!(theme_id = %theme_id, "Random theme selected for today");
        }

        Ok(())
    }

    /// List installed themes (bundled + custom) that look like valid themes.
    fn list_available_themes(&self, config: &Config) -> DwallResult<Vec<String>> {
        let mut themes: Vec<String> = Vec::new();

        for directory in [
            config.themes_directory(),
            config.customized_themes_directory(),
        ] {
            let Ok(entries) = std::fs::read_dir(directory) else {
                continue;
            };

            for entry in entries.flatten() {
                if !entry.file_type().map(|kind| kind.is_dir()).unwrap_or(false) {
                    continue;
                }

                let path = entry.path();
                if !is_theme_directory(&path) {
                    continue;
                }

                if let Some(name) = entry.file_name().to_str()
                    && !themes.iter().any(|existing| existing == name)
                {
                    themes.push(name.to_string());
                }
            }
        }

        themes.sort();
        Ok(themes)
    }

    /// Runs the engine loop until the configuration changes.
    fn run_engine_loop(&mut self, theme_engine: &PlatformEngine<'_>) -> DwallResult<()> {
        // A configuration change rebuilds the engine, which resets this: the user
        // editing the configuration is the most likely fix, so retry promptly.
        let mut failures = CycleFailureTracker::default();

        loop {
            let config_changed = match self.config_watcher.has_changed() {
                Ok(changed) => changed,
                Err(e) => {
                    // A transient metadata failure must not terminate the daemon.
                    warn!(
                        error = %e,
                        path = %self.config_watcher.config_path().display(),
                        "Failed to check the configuration file, keeping the current configuration"
                    );
                    false
                }
            };

            if config_changed {
                info!("Configuration file change detected, reloading configuration");
                if let Err(e) = self.config_watcher.update_stamp() {
                    warn!(error = %e, "Failed to record the configuration file stamp");
                }
                return Ok(());
            }

            let next_interval = match theme_engine.run_once() {
                Ok(_) => {
                    failures.record_success();
                    // The interval is the ceiling, not the schedule: a frame or
                    // light/dark change due sooner is met at the instant it happens.
                    theme_engine.next_wait()
                }
                Err(e) => {
                    // Keep running unless the user has switched the position off: see
                    // `cycle_failure`. The backoff keeps a persistently failing cycle
                    // from being retried at the full configured rate, and the count
                    // makes a repeating error distinguishable from a one-off.
                    let backoff = failures.record_failure();

                    match cycle_failure(&e, backoff) {
                        // The caller logs the reason and turns it into an exit code.
                        CycleFailure::Stop => return Err(e),
                        CycleFailure::Retry(delay) => {
                            error!(
                                error = %e,
                                consecutive_failures = failures.consecutive_failures(),
                                retry_in_seconds = delay.as_secs(),
                                "Theme update cycle failed, retrying"
                            );

                            delay
                        }
                    }
                }
            };

            theme_engine.reload_if_monitors_changed();

            // `continue` rather than sleeping and then re-checking: the top of the
            // loop records the new file stamp and rebuilds the engine.
            if wait_for_config_change(&self.config_watcher, next_interval) {
                continue;
            }
        }
    }
}

impl Default for DaemonApplication {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::solar::GeolocationAccessError;

    // ── cycle_failure ───────────────────────────────────────────────────────

    fn denied() -> DwallError {
        GeolocationAccessError::Denied.into()
    }

    #[test]
    fn a_denied_location_stops_the_daemon() {
        assert_eq!(
            cycle_failure(&denied(), FAILURE_BACKOFF_BASE),
            CycleFailure::Stop
        );
    }

    #[test]
    fn a_switched_off_location_service_stops_the_daemon() {
        let error: DwallError = GeolocationAccessError::Unspecified.into();

        assert_eq!(
            cycle_failure(&error, FAILURE_BACKOFF_MAX),
            CycleFailure::Stop
        );
    }

    #[test]
    fn an_access_denied_hresult_stops_the_daemon() {
        let error = DwallError::Windows(windows::core::Error::from_hresult(
            windows::core::HRESULT::from_win32(5),
        ));

        assert_eq!(
            cycle_failure(&error, FAILURE_BACKOFF_BASE),
            CycleFailure::Stop
        );
    }

    /// The failures the daemon used to die from silently keep being retried: a
    /// location service that is not up yet at sign-in must not end the run.
    #[test]
    fn a_transient_failure_is_retried_with_the_backoff() {
        for error in [
            DwallError::AsyncTimeout(Duration::from_secs(30)),
            DwallError::Windows(windows::core::Error::from_hresult(
                windows::core::HRESULT::from_win32(1053),
            )),
            GeolocationAccessError::Denied.into(),
        ] {
            let retried = cycle_failure(&error, Duration::from_secs(20));

            if error.location_unavailable() {
                assert_eq!(retried, CycleFailure::Stop);
            } else {
                assert_eq!(retried, CycleFailure::Retry(Duration::from_secs(20)));
            }
        }
    }

    #[test]
    fn only_the_location_errors_are_classified_as_unavailable() {
        assert!(denied().location_unavailable());
        assert!(
            DwallError::Windows(windows::core::Error::from_hresult(
                windows::core::HRESULT::from_win32(5)
            ))
            .location_unavailable()
        );
        assert!(
            !DwallError::Windows(windows::core::Error::from_hresult(
                windows::core::HRESULT::from_win32(1053)
            ))
            .location_unavailable(),
            "a busy service is not the user's decision"
        );
        assert!(!DwallError::AsyncTimeout(Duration::from_secs(30)).location_unavailable());
    }

    // ── failure_backoff ─────────────────────────────────────────────────────

    #[test]
    fn the_backoff_starts_at_the_base_delay() {
        assert_eq!(failure_backoff(1), FAILURE_BACKOFF_BASE);
    }

    #[test]
    fn the_backoff_doubles_per_consecutive_failure() {
        assert_eq!(failure_backoff(2), FAILURE_BACKOFF_BASE * 2);
        assert_eq!(failure_backoff(3), FAILURE_BACKOFF_BASE * 4);
        assert_eq!(failure_backoff(4), FAILURE_BACKOFF_BASE * 8);
    }

    #[test]
    fn the_backoff_stops_growing_at_the_ceiling() {
        // 5s doubling reaches the 300s ceiling on the seventh failure.
        assert_eq!(failure_backoff(6), Duration::from_secs(160));
        assert_eq!(failure_backoff(7), FAILURE_BACKOFF_MAX);
        assert_eq!(failure_backoff(64), FAILURE_BACKOFF_MAX);
        assert_eq!(failure_backoff(u32::MAX), FAILURE_BACKOFF_MAX);
    }

    #[test]
    fn a_successful_cycle_resets_the_failure_run() {
        let mut tracker = CycleFailureTracker::default();

        assert_eq!(tracker.record_failure(), FAILURE_BACKOFF_BASE);
        assert_eq!(tracker.record_failure(), FAILURE_BACKOFF_BASE * 2);
        assert_eq!(tracker.consecutive_failures(), 2);

        tracker.record_success();
        assert_eq!(tracker.consecutive_failures(), 0);
        assert_eq!(tracker.record_failure(), FAILURE_BACKOFF_BASE);
    }

    /// The previous policy called `std::process::exit(1)` here, which turned a
    /// denied location permission — the default position source, and a condition
    /// that never clears on its own — into a daemon that stayed dead until someone
    /// started it by hand. There is deliberately no terminating state left: the
    /// third failure, and the three-thousandth, are just another retry.
    #[test]
    fn three_consecutive_failures_no_longer_end_the_run() {
        let mut tracker = CycleFailureTracker::default();

        for _ in 0..3 {
            assert!(tracker.record_failure() >= FAILURE_BACKOFF_BASE);
        }
        assert_eq!(tracker.consecutive_failures(), 3);

        // Still retrying, and the delay is bounded by the ceiling.
        let fourth = tracker.record_failure();
        assert_eq!(tracker.consecutive_failures(), 4);
        assert!(fourth >= FAILURE_BACKOFF_BASE && fourth <= FAILURE_BACKOFF_MAX);
    }

    #[test]
    fn a_failure_run_that_lasts_for_days_neither_panics_nor_overflows() {
        let mut tracker = CycleFailureTracker::default();

        // A week at the 300s ceiling is a few thousand cycles; the count must
        // saturate rather than wrap (which would reset the log and the backoff).
        for _ in 0..10_000 {
            let backoff = tracker.record_failure();
            assert!(
                backoff >= FAILURE_BACKOFF_BASE && backoff <= FAILURE_BACKOFF_MAX,
                "backoff {backoff:?} outside [{FAILURE_BACKOFF_BASE:?}, {FAILURE_BACKOFF_MAX:?}]"
            );
        }

        assert_eq!(tracker.consecutive_failures(), 10_000);
    }

    #[test]
    fn the_capped_backoff_is_never_shorter_than_the_base_delay() {
        for failures in 1..=1_000 {
            let backoff = failure_backoff(failures);
            assert!(backoff >= FAILURE_BACKOFF_BASE);
        }
    }

    /// The boundaries the review names: zero failures is still a valid delay, the
    /// first failure is the base, and 17 failures are already past the cap.
    #[test]
    fn the_backoff_is_defined_at_the_reviewed_boundaries() {
        assert_eq!(failure_backoff(0), FAILURE_BACKOFF_BASE);
        assert_eq!(failure_backoff(1), FAILURE_BACKOFF_BASE);
        assert_eq!(failure_backoff(17), FAILURE_BACKOFF_MAX);
        assert_eq!(failure_backoff(u32::MAX), FAILURE_BACKOFF_MAX);
    }

    // ── Random mode ─────────────────────────────────────────────────────────

    #[test]
    fn random_pool_is_the_configured_pool_only_in_random_mode() {
        let mut config = Config::default();

        config.set_wallpaper_mode(WallpaperMode::Random { pool: None });
        assert_eq!(random_pool(&config), None, "no pool means every theme");

        let pool = vec!["Alpha".to_string(), "Beta".to_string()];
        config.set_wallpaper_mode(WallpaperMode::Random {
            pool: Some(pool.clone()),
        });
        assert_eq!(random_pool(&config), Some(&pool));

        config.set_wallpaper_mode(WallpaperMode::Fixed {
            monitor_specific_wallpapers: MonitorSpecificWallpapers::All("Alpha".to_string()),
        });
        assert_eq!(random_pool(&config), None, "fixed mode has no pool");
    }

    /// A scratch themes root with one valid theme in it, plus a config file that
    /// points both theme roots at the scratch directory. The state file the random
    /// mode writes lives next to that config file.
    fn random_config(name: &str, mode: WallpaperMode) -> (Config, PathBuf, PathBuf) {
        let dir = std::env::temp_dir().join(format!("dwall_daemon_random_{name}"));
        let _ = std::fs::remove_dir_all(&dir);
        let themes = dir.join("themes");
        let customized = dir.join("customized");
        let theme_dir = themes.join("Alpha");
        std::fs::create_dir_all(&theme_dir).unwrap();
        std::fs::create_dir_all(&customized).unwrap();
        std::fs::write(theme_dir.join("solar.json"), "[]").unwrap();

        let config_path = dir.join("config.toml");
        let themes = themes.display().to_string().replace('\\', "/");
        let customized = customized.display().to_string().replace('\\', "/");
        std::fs::write(
            &config_path,
            format!(
                "themes_directory = \"{themes}\"\ncustomized_themes_directory = \"{customized}\"\n"
            ),
        )
        .unwrap();

        let mut config = ConfigReader::read_from_path(&config_path).unwrap();
        config.set_wallpaper_mode(mode);

        (config, config_path, dir)
    }

    /// Fixed mode drops any in-memory random state, so switching back to random draws
    /// afresh rather than reusing a theme chosen for another mode.
    #[test]
    fn fixed_mode_clears_the_random_state() {
        let mut daemon = DaemonApplication::with_config_path(
            std::env::temp_dir().join("dwall_daemon_fixed.toml"),
        );
        daemon.random_state = Some(RandomModeState {
            selector: DailyRandomThemeSelector::new(),
            current_theme: Some("Alpha".to_string()),
        });

        let mut config = Config::default();
        config.set_wallpaper_mode(WallpaperMode::Fixed {
            monitor_specific_wallpapers: MonitorSpecificWallpapers::All("Alpha".to_string()),
        });

        daemon.apply_wallpaper_mode(&mut config).unwrap();

        assert!(daemon.random_state.is_none());
    }

    /// An empty pool is a configured state, not an error: nothing is selected, the
    /// mode stays Random, and no state file is written.
    #[test]
    fn handle_random_mode_with_an_empty_pool_selects_nothing() {
        let (mut config, config_path, dir) = random_config(
            "empty_pool",
            WallpaperMode::Random {
                pool: Some(Vec::new()),
            },
        );
        let mut daemon = DaemonApplication::with_config_path(config_path);

        daemon.handle_random_mode(&mut config).unwrap();

        assert!(
            daemon
                .random_state
                .as_ref()
                .unwrap()
                .current_theme
                .is_none()
        );
        assert!(matches!(
            config.wallpaper_mode(),
            WallpaperMode::Random { .. }
        ));
        assert!(!dir.join("random_selection.json").exists());

        let _ = std::fs::remove_dir_all(&dir);
    }

    /// A persisted draw naming a theme that has since been deleted must not fail the
    /// cycle: the selector redraws from what is still installed and persists that.
    #[test]
    fn handle_random_mode_redraws_when_todays_theme_was_deleted() {
        let (mut config, config_path, dir) =
            random_config("deleted", WallpaperMode::Random { pool: None });
        let today = OffsetDateTime::now_local().unwrap().date();
        crate::random_state::write_to(
            &dir,
            &crate::random_state::RandomSelection::new(&today, "Gone".to_string()),
        )
        .unwrap();

        let mut daemon = DaemonApplication::with_config_path(config_path);
        daemon.handle_random_mode(&mut config).unwrap();

        assert_eq!(
            daemon
                .random_state
                .as_ref()
                .unwrap()
                .current_theme
                .as_deref(),
            Some("Alpha")
        );
        match config.wallpaper_mode() {
            WallpaperMode::Fixed {
                monitor_specific_wallpapers,
            } => assert_eq!(
                monitor_specific_wallpapers,
                &MonitorSpecificWallpapers::All("Alpha".to_string())
            ),
            other => panic!("expected the selected theme to be carried as Fixed, got {other:?}"),
        }

        let persisted = crate::random_state::read_from(&dir).unwrap().unwrap();
        assert_eq!(persisted.date, crate::random_state::date_key(&today));
        assert_eq!(persisted.theme_id, "Alpha");

        let _ = std::fs::remove_dir_all(&dir);
    }

    // ── wait_for_config_change ──────────────────────────────────────────────

    /// A watcher over a scratch config file, with its file stamp recorded.
    fn watcher_for(name: &str) -> (ConfigWatcher, PathBuf) {
        let dir = std::env::temp_dir().join(format!("dwall_daemon_wait_{name}"));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();

        let path = dir.join("config.toml");
        std::fs::write(&path, "interval = 600\n").unwrap();

        let mut watcher = ConfigWatcher::new(path.clone());
        watcher.update_stamp().unwrap();

        (watcher, path)
    }

    #[test]
    fn the_wait_ends_after_the_duration_when_nothing_changes() {
        let (watcher, path) = watcher_for("quiet");
        let dir = path.parent().unwrap().to_path_buf();
        let started = Instant::now();

        assert!(!wait_for_config_change(
            &watcher,
            Duration::from_millis(250)
        ));
        assert!(started.elapsed() >= Duration::from_millis(250));

        let _ = std::fs::remove_dir_all(&dir);
    }

    /// The regression test for the latency: with `interval = 600` a saved setting
    /// used to wait out the whole interval, because the file was only checked at the
    /// top of the loop.
    #[test]
    fn a_change_during_the_wait_ends_it_early() {
        let (watcher, path) = watcher_for("changed");
        let dir = path.parent().unwrap().to_path_buf();

        let writer = std::thread::spawn(move || {
            // Long enough that the wait is already sleeping, short enough that a
            // missed slice would be visible against the 30s timeout below.
            sleep(Duration::from_millis(150));
            std::fs::write(&path, "interval = 30\n").unwrap();
        });

        let started = Instant::now();
        let changed = wait_for_config_change(&watcher, Duration::from_secs(30));
        let elapsed = started.elapsed();

        writer.join().unwrap();

        assert!(changed, "the wait has to notice the change");
        assert!(
            elapsed < Duration::from_secs(10),
            "the wait must not run out the interval: {elapsed:?}"
        );

        let _ = std::fs::remove_dir_all(&dir);
    }

    /// A configuration file that disappears is a change too, and the same wait
    /// notices it.
    #[test]
    fn a_removed_configuration_file_ends_the_wait() {
        let (watcher, path) = watcher_for("removed");
        let dir = path.parent().unwrap().to_path_buf();

        let remover = std::thread::spawn(move || {
            sleep(Duration::from_millis(150));
            std::fs::remove_file(&path).unwrap();
        });

        let started = Instant::now();
        let changed = wait_for_config_change(&watcher, Duration::from_secs(30));
        let elapsed = started.elapsed();

        remover.join().unwrap();

        assert!(changed);
        assert!(elapsed < Duration::from_secs(10), "{elapsed:?}");

        let _ = std::fs::remove_dir_all(&dir);
    }
}
