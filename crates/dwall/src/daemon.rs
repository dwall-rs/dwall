//! Daemon process: watches the config, runs theme update cycles and handles
//! random wallpaper mode.

use std::path::{Path, PathBuf};
use std::thread::sleep;
use std::time::Duration;

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

const MAX_CONSECUTIVE_FAILURE_THRESHOLD: u8 = 3;

/// Backoff before retrying a configuration read that failed.
const CONFIG_RETRY_DELAY: Duration = Duration::from_secs(5);

/// Builds the engine for one configuration generation.
fn build_engine(config: &Config) -> DwallResult<ThemeEngine<'_>> {
    let wallpaper_setter = WallpaperSetter::new().map_err(DwallError::WallpaperManager)?;
    let monitor_provider = DisplayMonitorProvider::new();
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
        let mut consecutive_failure_count = 0;

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

            self.run_engine_loop(&theme_engine, &mut consecutive_failure_count)?;
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

    /// Runs the engine loop until config changes or max failures reached
    fn run_engine_loop(
        &mut self,
        theme_engine: &ThemeEngine<'_>,
        consecutive_failure_count: &mut u8,
    ) -> DwallResult<()> {
        let update_interval = Duration::from_secs(theme_engine.update_interval().into());

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

            match theme_engine.run_once() {
                Ok(_) => {
                    *consecutive_failure_count = 0;
                }
                Err(e) => {
                    *consecutive_failure_count += 1;
                    error!(
                        error = %e,
                        consecutive_failures = *consecutive_failure_count,
                        "Theme update cycle failed"
                    );
                    if *consecutive_failure_count >= MAX_CONSECUTIVE_FAILURE_THRESHOLD {
                        error!(
                            consecutive_failures = *consecutive_failure_count,
                            "Maximum consecutive failures reached, terminating daemon"
                        );
                        std::process::exit(1);
                    }
                }
            }

            theme_engine.reload_if_monitors_changed();

            sleep(update_interval);
        }
    }
}

impl Default for DaemonApplication {
    fn default() -> Self {
        Self::new()
    }
}
