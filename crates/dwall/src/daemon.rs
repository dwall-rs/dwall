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
const CONFIG_POLL_INTERVAL: Duration = Duration::from_secs(1);

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
/// location access denied (the default position source is automatic), the
/// location service switched off, a shell that stopped answering — so exiting
/// converted "the wallpaper is not updating right now" into "the wallpaper never
/// updates again until someone starts the daemon by hand", with no supervisor to
/// do it. Cycles therefore keep running: the failure count only drives the
/// backoff and the log.
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

/// Builds the engine for one configuration generation.
fn build_engine(config: &Config) -> DwallResult<ThemeEngine<'_>> {
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
        // Establish the baseline modification time so the first loop iteration does
        // not report a change that never happened.
        if let Err(e) = self.config_watcher.update_modified_time() {
            warn!(error = %e, "Failed to read the initial configuration modification time");
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
    fn run_engine_loop(&mut self, theme_engine: &ThemeEngine<'_>) -> DwallResult<()> {
        let update_interval = Duration::from_secs(theme_engine.update_interval().into());

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
                if let Err(e) = self.config_watcher.update_modified_time() {
                    warn!(error = %e, "Failed to record the configuration modification time");
                }
                return Ok(());
            }

            let next_interval = match theme_engine.run_once() {
                Ok(_) => {
                    failures.record_success();
                    update_interval
                }
                Err(e) => {
                    // Keep running: see `CycleFailureTracker`. The backoff keeps a
                    // persistently failing cycle from being retried at the full
                    // configured rate, and the count makes a repeating error
                    // distinguishable from a one-off.
                    let backoff = failures.record_failure();
                    error!(
                        error = %e,
                        consecutive_failures = failures.consecutive_failures(),
                        retry_in_seconds = backoff.as_secs(),
                        "Theme update cycle failed, retrying"
                    );
                    backoff
                }
            };

            theme_engine.reload_if_monitors_changed();

            // `continue` rather than sleeping and then re-checking: the top of the
            // loop records the new modification time and rebuilds the engine.
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

    // ── wait_for_config_change ──────────────────────────────────────────────

    /// A watcher over a scratch config file, with its modification time recorded.
    fn watcher_for(name: &str) -> (ConfigWatcher, PathBuf) {
        let dir = std::env::temp_dir().join(format!("dwall_daemon_wait_{name}"));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();

        let path = dir.join("config.toml");
        std::fs::write(&path, "interval = 600\n").unwrap();

        let mut watcher = ConfigWatcher::new(path.clone());
        watcher.update_modified_time().unwrap();

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
