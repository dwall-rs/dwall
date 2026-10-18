//! Theme update engine: applies wallpapers and color scheme for a single cycle.

use std::cell::Cell;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::time::Duration;

use time::OffsetDateTime;

use crate::DwallResult;
use crate::color_scheme::{ColorScheme, ColorSchemeApplier, ColorSchemeProvider};
use crate::config::{Config, MonitorSpecificWallpapers};
use crate::monitor::MonitorProvider;
use crate::platform::DisplayMonitor;
use crate::solar::{Position, PositionProvider, SolarPosition};
use crate::theme::{
    ThemeError, get_theme_directory_path, load_cached_theme_data, wallpaper_image_path,
};
use crate::wallpaper::{WallpaperProvider, WallpaperSelector};

/// Orchestrates the solar-based theme update cycle.
///
/// Generic over what it drives rather than fixed to the Windows backends, so the
/// cycle — which monitor gets which theme, what happens when one of them fails, what
/// the lock screen shows — can be tested against doubles. It used to take the
/// concrete `WallpaperSetter`/`DisplayMonitorProvider`/`ColorSchemeScheduler`, which
/// left the entire orchestration layer untestable: nothing outside
/// [`crate::daemon`] could build one.
pub(crate) struct ThemeEngine<'a, M, W, C, P>
where
    M: MonitorProvider,
    W: WallpaperProvider,
    C: ColorSchemeProvider,
    P: PositionProvider,
{
    config: &'a Config,
    position_provider: P,
    wallpaper_setter: W,
    /// Shared with the wallpaper setter in production, so both see one snapshot of
    /// the displays.
    monitor_provider: Rc<M>,
    color_scheme_applier: ColorSchemeApplier<C>,
    /// The position the last cycle ran with.
    ///
    /// The schedule needs the same position the cycle used; asking the provider again
    /// would re-read the location sensor just to decide when to wake up.
    last_position: Cell<Option<Position>>,
}

impl<'a, M, W, C, P> ThemeEngine<'a, M, W, C, P>
where
    M: MonitorProvider,
    W: WallpaperProvider,
    C: ColorSchemeProvider,
    P: PositionProvider,
{
    pub(crate) fn new(
        config: &'a Config,
        wallpaper_setter: W,
        monitor_provider: Rc<M>,
        color_scheme_provider: C,
        position_provider: P,
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
            last_position: Cell::new(None),
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

        self.process_theme_cycle_at(position, &now_local)
    }

    /// [`process_theme_cycle`](Self::process_theme_cycle) for an explicit local time.
    ///
    /// The frame a theme shows is chosen from the sun's position at that instant, so
    /// testing the choice means naming the instant rather than depending on when the
    /// test happens to run.
    fn process_theme_cycle_at(
        &self,
        position: &Position,
        now_local: &OffsetDateTime,
    ) -> DwallResult<()> {
        let now = now_local.assume_utc();
        let solar_position = SolarPosition::new(position, &now);
        let available_monitors = self.monitor_provider.get_monitors()?;

        // Recorded before anything can fail: the schedule only needs to know which
        // position the daemon is reasoning about, and a failed wallpaper write does not
        // change that.
        self.last_position.set(Some(*position));

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
            now_local,
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

    /// The image index every configured monitor would show at `solar_position`.
    ///
    /// Read-only: nothing is applied, and the selection goes through the same
    /// [`WallpaperSelector`] the cycle uses. `None` means the question cannot be
    /// answered right now (a theme directory that moved, a monitor list that could not
    /// be read) — the caller then falls back to its plain interval instead of guessing.
    fn selected_images(&self, solar_position: &SolarPosition) -> Option<Vec<u8>> {
        let monitors = self.monitor_provider.get_monitors().ok()?;
        let mut images = Vec::new();

        for (_, theme_id) in monitor_targets(self.config.monitor_specific_wallpapers(), &monitors) {
            let (theme_dir, _) = get_theme_directory_path(self.config, theme_id);
            let theme_data = load_cached_theme_data(&theme_dir).ok()?;

            images.push(WallpaperSelector::find_closest_image(
                theme_data.angles.as_slice(),
                solar_position.altitude(),
                solar_position.azimuth(),
            )?);
        }

        Some(images)
    }

    /// How long until the daemon's output would change, within `horizon`.
    ///
    /// "Output" is both what the monitors show and the light/dark scheme, combined into
    /// one state so the loop wakes for whichever comes first. `None` when nothing
    /// changes inside the window, or when the state cannot be evaluated; the caller
    /// keeps its interval, so a missing answer only costs precision.
    pub(crate) fn next_event_within(
        &self,
        now_local: &OffsetDateTime,
        horizon: Duration,
    ) -> Option<Duration> {
        let position = self.last_position.get()?;
        let current_scheme = self.current_scheme_for_schedule();

        crate::schedule::next_change(now_local, horizon, |at| {
            self.output_at(&position, at, current_scheme)
        })
    }

    /// The daemon's output at `at`, as one comparable value.
    ///
    /// Kept separate from the search so a test can compare the schedule against a
    /// second-by-second scan of the very same function — the search must find the first
    /// change of *this* state, not of a re-implementation of it.
    fn output_at(
        &self,
        position: &Position,
        at: &OffsetDateTime,
        current_scheme: Option<ColorScheme>,
    ) -> Option<(Vec<u8>, Option<ColorScheme>)> {
        let solar_position = SolarPosition::new(position, &at.assume_utc());
        let images = self.selected_images(&solar_position)?;
        let scheme = current_scheme.map(|current_scheme| {
            self.color_scheme_applier
                .scheme_at(at, position, &solar_position, current_scheme)
        });

        Some((images, scheme))
    }

    /// The scheme the schedule measures changes against: the one the system shows now.
    ///
    /// `None` when the light/dark switch is off, or when the current scheme cannot be
    /// read — the schedule then only watches the wallpapers.
    fn current_scheme_for_schedule(&self) -> Option<ColorScheme> {
        if !self.config.auto_detect_color_scheme() {
            return None;
        }

        self.color_scheme_applier.current_scheme().ok()
    }

    /// The wait the loop should use now that a cycle has finished.
    ///
    /// The interval stays the ceiling: it is the period after which the daemon
    /// recomputes everything anyway, which is what bounds the damage from a clock jump,
    /// a suspended machine, a theme edited on disk or a position that moved. The
    /// schedule only trims the wait when a change is due sooner.
    pub(crate) fn next_wait(&self) -> Duration {
        let interval = Duration::from_secs(self.update_interval().into());

        let Ok(now_local) = OffsetDateTime::now_local() else {
            return interval;
        };

        scheduled_wait(interval, self.next_event_within(&now_local, interval))
    }
}

/// The wait for a finished cycle: the schedule's answer if it is closer, else the
/// interval, and never zero.
///
/// A zero wait would run the cycle again immediately, which is what a change that is
/// already due would otherwise produce.
fn scheduled_wait(interval: Duration, event: Option<Duration>) -> Duration {
    event.map_or(interval, |event| {
        event.clamp(crate::schedule::MIN_WAIT, interval)
    })
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
    use std::cell::RefCell;
    use std::collections::HashSet;
    use std::fs;

    use super::*;
    use crate::color_scheme::ColorScheme;
    use crate::config::{MonitorSpecificWallpapers, WallpaperMode};

    // ── Doubles ─────────────────────────────────────────────────────────────
    //
    // The engine is generic over what it drives so these can stand in for the
    // Windows backends. They record what they were asked to do rather than only
    // answering, because what is under test is the *orchestration*: which monitor
    // gets which theme, and what a failure in one of them changes for the others.

    /// A theme's frames, expressed as the `solar.json` a legacy theme carries.
    const TWO_FRAMES: &str = r#"[
        {"index": 0, "altitude": -30.0, "azimuth": 0.0},
        {"index": 1, "altitude": 30.0, "azimuth": 0.0}
    ]"#;

    /// Frames whose boundary sits at 71° of altitude, which the summer sun above
    /// Beijing crosses again about fifty minutes after local noon.
    const MIDDAY_FRAMES: &str = r#"[
        {"index": 0, "altitude": 68.0, "azimuth": 0.0},
        {"index": 1, "altitude": 74.0, "azimuth": 0.0}
    ]"#;

    /// A theme with one frame: its image can never change.
    const ONE_FRAME: &str = r#"[
        {"index": 0, "altitude": 0.0, "azimuth": 0.0}
    ]"#;

    fn scratch(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("dwall_engine_{name}"));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();

        dir
    }

    /// Writes a legacy theme: `solar.json` plus one image per frame.
    fn write_theme(
        themes_directory: &Path,
        theme_id: &str,
        frames: &str,
        format: &str,
        images: &[u32],
    ) {
        let theme_dir = themes_directory.join(theme_id);
        fs::create_dir_all(theme_dir.join(format)).unwrap();
        fs::write(theme_dir.join("solar.json"), frames).unwrap();

        for index in images {
            fs::write(
                theme_dir.join(format).join(format!("{index}.{format}")),
                "image",
            )
            .unwrap();
        }
    }

    /// A configuration rooted in a scratch directory, built through `serde` because
    /// the fields are private.
    fn config_at(themes_directory: &Path, mode: WallpaperMode, lock_screen: bool) -> Config {
        let json = format!(
            r#"{{"themes_directory":{},"customized_themes_directory":{},"image_format":"jpeg","lock_screen_wallpaper_enabled":{}}}"#,
            serde_json::to_string(&themes_directory.to_string_lossy()).unwrap(),
            serde_json::to_string(&themes_directory.join("custom").to_string_lossy()).unwrap(),
            lock_screen,
        );

        let mut config: Config = serde_json::from_str(&json).unwrap();
        config.set_wallpaper_mode(mode);
        config
    }

    struct StaticMonitors {
        monitors: HashMap<String, DisplayMonitor>,
    }

    impl MonitorProvider for StaticMonitors {
        fn get_monitors(&self) -> DwallResult<HashMap<String, DisplayMonitor>> {
            Ok(self.monitors.clone())
        }

        fn refresh_monitors(&self) -> DwallResult<HashMap<String, DisplayMonitor>> {
            Ok(self.monitors.clone())
        }

        fn has_configuration_changed(&self) -> DwallResult<bool> {
            Ok(false)
        }
    }

    /// What the cycle asked the wallpaper backend for.
    ///
    /// The state is shared rather than owned so the test keeps a handle while the
    /// engine owns its own: the engine takes its providers by value.
    #[derive(Default)]
    struct WallpaperCalls {
        applied: RefCell<Vec<(String, String)>>,
        lock_screen: RefCell<Vec<String>>,
        failing_monitors: HashSet<String>,
        lock_screen_fails: bool,
    }

    #[derive(Clone, Default)]
    struct RecordingWallpapers {
        calls: Rc<WallpaperCalls>,
    }

    impl RecordingWallpapers {
        fn failing_for(monitors: &[&str]) -> Self {
            Self {
                calls: Rc::new(WallpaperCalls {
                    failing_monitors: monitors.iter().map(|id| (*id).to_string()).collect(),
                    ..Default::default()
                }),
            }
        }

        fn failing_lock_screen() -> Self {
            Self {
                calls: Rc::new(WallpaperCalls {
                    lock_screen_fails: true,
                    ..Default::default()
                }),
            }
        }

        /// The image file names applied to `monitor_id`, in order.
        fn images_for(&self, monitor_id: &str) -> Vec<String> {
            self.calls
                .applied
                .borrow()
                .iter()
                .filter(|(monitor, _)| monitor == monitor_id)
                .map(|(_, image)| image.clone())
                .collect()
        }

        fn monitors_touched(&self) -> Vec<String> {
            let mut monitors: Vec<String> = self
                .calls
                .applied
                .borrow()
                .iter()
                .map(|(monitor, _)| monitor.clone())
                .collect();
            monitors.sort();
            monitors.dedup();

            monitors
        }

        fn lock_screen_images(&self) -> Vec<String> {
            self.calls.lock_screen.borrow().clone()
        }
    }

    impl WallpaperProvider for RecordingWallpapers {
        fn set_monitor_wallpaper(&self, monitor_id: &str, image_path: &Path) -> DwallResult<()> {
            if self.calls.failing_monitors.contains(monitor_id) {
                return Err(ThemeError::WallpaperImageMissing {
                    path: image_path.display().to_string(),
                }
                .into());
            }

            self.calls.applied.borrow_mut().push((
                monitor_id.to_string(),
                image_path
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .into_owned(),
            ));

            Ok(())
        }

        fn set_lock_screen_image(&self, image_path: &Path) -> DwallResult<()> {
            if self.calls.lock_screen_fails {
                return Err(ThemeError::WallpaperImageMissing {
                    path: image_path.display().to_string(),
                }
                .into());
            }

            self.calls.lock_screen.borrow_mut().push(
                image_path
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .into_owned(),
            );

            Ok(())
        }
    }

    struct FixedPosition(Position);

    impl PositionProvider for FixedPosition {
        fn get_current_position(&self) -> DwallResult<Position> {
            Ok(self.0)
        }

        fn check_location_permission(&self) -> DwallResult<()> {
            Ok(())
        }
    }

    /// Records the color schemes the cycle asked for.
    #[derive(Clone, Default)]
    struct RecordingScheme {
        applied: Rc<RefCell<Vec<ColorScheme>>>,
    }

    impl RecordingScheme {
        fn applied(&self) -> Vec<ColorScheme> {
            self.applied.borrow().clone()
        }
    }

    impl ColorSchemeProvider for RecordingScheme {
        fn get_current_scheme(&self) -> DwallResult<ColorScheme> {
            Ok(ColorScheme::Light)
        }

        fn set_color_scheme(&self, scheme: ColorScheme) -> DwallResult<()> {
            self.applied.borrow_mut().push(scheme);

            Ok(())
        }
    }

    /// Everything one cycle needs, kept alive together.
    struct Fixture {
        config: Config,
        wallpapers: RecordingWallpapers,
        monitors: Rc<StaticMonitors>,
        scheme: RecordingScheme,
        position: Position,
        directory: PathBuf,
        themes_directory: PathBuf,
    }

    impl Fixture {
        fn new(
            name: &str,
            monitors: &[&str],
            mode: WallpaperMode,
            themes: &[(&str, &[u32])],
        ) -> Self {
            Self::new_with_frames(name, monitors, mode, themes, TWO_FRAMES)
        }

        /// [`new`](Self::new) with the frames every theme declares.
        ///
        /// A test that needs other frames has to hand them over here rather than
        /// rewriting `solar.json` afterwards: the theme cache is invalidated on the
        /// file's modification time, which has a one-second resolution, so a rewrite in
        /// the same second keeps serving the old angles.
        fn new_with_frames(
            name: &str,
            monitors: &[&str],
            mode: WallpaperMode,
            themes: &[(&str, &[u32])],
            frames: &str,
        ) -> Self {
            let directory = scratch(name);
            let themes_directory = directory.join("themes");
            fs::create_dir_all(&themes_directory).unwrap();

            for (theme_id, images) in themes {
                write_theme(&themes_directory, theme_id, frames, "jpg", images);
            }

            Self {
                config: config_at(&themes_directory, mode, false),
                wallpapers: RecordingWallpapers::default(),
                monitors: Rc::new(StaticMonitors {
                    monitors: tests::monitors(monitors),
                }),
                scheme: RecordingScheme::default(),
                position: beijing(),
                directory,
                themes_directory,
            }
        }

        fn fail_for(mut self, monitors: &[&str]) -> Self {
            self.wallpapers = RecordingWallpapers::failing_for(monitors);

            self
        }

        /// Rebuilds the configuration with the lock screen enabled, which is a field
        /// without a setter.
        fn with_lock_screen(mut self) -> Self {
            self.config = config_at(
                &self.themes_directory,
                self.config.wallpaper_mode().clone(),
                true,
            );

            self
        }

        /// Rebuilds the configuration with the light/dark switch off, so a schedule
        /// test only has the wallpapers to watch.
        fn without_color_scheme(mut self) -> Self {
            self.config = self.config_with_extra(r#""auto_detect_color_scheme":false"#);

            self
        }

        /// The fixture's configuration plus raw JSON keys, through `serde` again
        /// because those fields have no setter.
        fn config_with_extra(&self, extra: &str) -> Config {
            let json = format!(
                r#"{{"themes_directory":{},"customized_themes_directory":{},"image_format":"jpeg","lock_screen_wallpaper_enabled":false,{extra}}}"#,
                serde_json::to_string(&self.themes_directory.to_string_lossy()).unwrap(),
                serde_json::to_string(&self.themes_directory.join("custom").to_string_lossy())
                    .unwrap(),
            );

            let mut config: Config = serde_json::from_str(&json).unwrap();
            config.set_wallpaper_mode(self.config.wallpaper_mode().clone());

            config
        }

        fn engine(
            &self,
        ) -> ThemeEngine<'_, StaticMonitors, RecordingWallpapers, RecordingScheme, FixedPosition>
        {
            ThemeEngine::new(
                &self.config,
                self.wallpapers.clone(),
                Rc::clone(&self.monitors),
                self.scheme.clone(),
                FixedPosition(self.position),
            )
        }

        fn run(&self) -> DwallResult<()> {
            self.engine()
                .process_theme_cycle_at(&self.position, &summer_noon())
        }

        fn cleanup(&self) {
            let _ = fs::remove_dir_all(&self.directory);
        }
    }

    /// A position in Beijing.
    fn beijing() -> Position {
        Position::new(39.9, 116.4, 50.0).unwrap()
    }

    /// A local time with the offset `+08:00`.
    fn local_time(hour: u8) -> OffsetDateTime {
        OffsetDateTime::new(
            2026,
            time::Month::June,
            21,
            hour,
            0,
            0,
            "+08:00".parse().unwrap(),
        )
        .unwrap()
    }

    /// Midday at the summer solstice: the sun is high above Beijing, so a theme's
    /// daylight frame is the one selected.
    fn summer_noon() -> OffsetDateTime {
        local_time(12)
    }

    /// The same day at midnight, when the sun is well below the horizon.
    fn summer_midnight() -> OffsetDateTime {
        local_time(0)
    }

    // ── monitor_targets / apply_monitor_targets (pure) ──────────────────────

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

    // ── The cycle, driven through the providers ─────────────────────────────

    #[test]
    fn every_configured_monitor_is_attempted_even_when_one_fails() {
        let fixture = Fixture::new(
            "one_fails",
            &["MON-A", "MON-B"],
            individual(&[("MON-A", "Broken"), ("MON-B", "Big Sur")]),
            &[("Big Sur", &[1, 2])],
        )
        .fail_for(&["MON-A"]);

        // MON-A's theme does not exist, so its wallpaper fails; that must not stop
        // MON-B, which is the next target.
        assert!(fixture.run().is_ok(), "one success keeps the cycle alive");
        assert_eq!(fixture.wallpapers.monitors_touched(), vec!["MON-B"]);
    }

    #[test]
    fn a_cycle_where_every_configured_monitor_fails_is_a_failure() {
        let fixture = Fixture::new(
            "all_fail",
            &["MON-A", "MON-B"],
            individual(&[("MON-A", "Missing One"), ("MON-B", "Missing Two")]),
            &[],
        );

        // Before this, a per-monitor failure was only a warning and the cycle always
        // reported success: a broken theme was neither retried with a backoff nor
        // visible as an error.
        let error = fixture.run().expect_err("nothing was applied");

        assert!(
            error.to_string().contains("2"),
            "the error names how many monitors failed: {error}"
        );
    }

    /// The other side of the same rule: a configuration that names no monitor has
    /// nothing to do, which is not a failure — `dwall --once` has to keep exiting 0
    /// on a default configuration.
    #[test]
    fn monitors_without_a_configured_theme_are_skipped_and_do_not_fail_the_cycle() {
        let fixture = Fixture::new(
            "skipped",
            &["MON-A", "MON-B"],
            individual(&[("MON-A", "Big Sur")]),
            &[("Big Sur", &[1, 2])],
        );

        assert!(fixture.run().is_ok());
        assert_eq!(
            fixture.wallpapers.monitors_touched(),
            vec!["MON-A"],
            "MON-B has no theme configured and is left alone"
        );
    }

    #[test]
    fn a_missing_image_is_reported_per_monitor() {
        // The theme has one frame declared but no image for it: the failure is
        // attributed to the monitor that asked for it, and the other monitor with a
        // working theme still gets its wallpaper.
        let fixture = Fixture::new(
            "missing_image",
            &["MON-A", "MON-B"],
            individual(&[("MON-A", "No Image"), ("MON-B", "Big Sur")]),
            &[("No Image", &[]), ("Big Sur", &[1, 2])],
        );

        assert!(fixture.run().is_ok());
        assert_eq!(fixture.wallpapers.monitors_touched(), vec!["MON-B"]);
    }

    #[test]
    fn a_custom_theme_uses_its_manifest_image_format() {
        let directory = scratch("manifest_format");
        let themes_directory = directory.join("themes");
        let custom_directory = directory.join("custom");
        let theme_dir = custom_directory.join("My Theme");
        fs::create_dir_all(&theme_dir).unwrap();
        // A custom theme declares its frames and its format in `theme.toml`, and keeps
        // its images under `images/` rather than under the format's name.
        fs::write(
            theme_dir.join("theme.toml"),
            r#"schema = 1

[theme]
id = "my-theme"
name = "My Theme"
author = "tester"
version = 1
created_at = 1700000000
image_format = "png"

[solar]
mode = "native"

[[solar.frames]]
index = 0
altitude = -30.0
azimuth = 0.0

[[solar.frames]]
index = 1
altitude = 30.0
azimuth = 0.0
"#,
        )
        .unwrap();
        fs::create_dir_all(theme_dir.join("images")).unwrap();
        for index in [1, 2] {
            fs::write(
                theme_dir.join("images").join(format!("{index}.png")),
                "image",
            )
            .unwrap();
        }

        let json = format!(
            r#"{{"themes_directory":{},"customized_themes_directory":{},"image_format":"jpeg"}}"#,
            serde_json::to_string(&themes_directory.to_string_lossy()).unwrap(),
            serde_json::to_string(&custom_directory.to_string_lossy()).unwrap(),
        );
        let mut config: Config = serde_json::from_str(&json).unwrap();
        config.set_wallpaper_mode(individual(&[("MON-A", "My Theme")]));

        let wallpapers = RecordingWallpapers::default();
        let engine = ThemeEngine::new(
            &config,
            wallpapers.clone(),
            Rc::new(StaticMonitors {
                monitors: tests::monitors(&["MON-A"]),
            }),
            RecordingScheme::default(),
            FixedPosition(beijing()),
        );

        engine
            .process_theme_cycle_at(&beijing(), &summer_noon())
            .unwrap();

        // The configuration says jpeg; the manifest says png, and the manifest wins.
        assert_eq!(wallpapers.images_for("MON-A"), vec!["2.png".to_string()]);

        let _ = fs::remove_dir_all(&directory);
    }

    #[test]
    fn lock_screen_is_skipped_when_no_monitor_succeeded() {
        let fixture = Fixture::new(
            "lock_skipped",
            &["MON-A"],
            individual(&[("MON-A", "Missing")]),
            &[],
        )
        .with_lock_screen();

        assert!(fixture.run().is_err());
        assert!(
            fixture.wallpapers.lock_screen_images().is_empty(),
            "the lock screen must not show a theme the desktop could not apply"
        );

        fixture.cleanup();
    }

    #[test]
    fn lock_screen_theme_is_deterministic_for_a_multi_monitor_map() {
        // Two monitors with different themes and the lock screen enabled: the lock
        // screen has to pick the same theme every cycle, not whichever one the map
        // happens to yield first.
        for _ in 0..5 {
            let fixture = Fixture::new(
                "lock_deterministic",
                &["MON-A", "MON-B"],
                individual(&[("MON-A", "Alpha"), ("MON-B", "Beta")]),
                &[("Alpha", &[1, 2]), ("Beta", &[1, 2])],
            )
            .with_lock_screen();

            fixture.run().unwrap();

            let applied = fixture.wallpapers.lock_screen_images();
            assert_eq!(applied.len(), 1, "the lock screen is set once per cycle");
            assert_eq!(
                applied[0], "2.jpg",
                "both themes declare the same frames, so the image name must match"
            );

            fixture.cleanup();
        }
    }

    #[test]
    fn a_failing_lock_screen_does_not_fail_the_cycle() {
        let mut fixture = Fixture::new(
            "lock_fails",
            &["MON-A"],
            individual(&[("MON-A", "Big Sur")]),
            &[("Big Sur", &[1, 2])],
        )
        .with_lock_screen();
        fixture.wallpapers = RecordingWallpapers::failing_lock_screen();

        // The desktop wallpaper worked; a lock screen failure is a warning, not a
        // failed cycle.
        assert!(fixture.run().is_ok());

        fixture.cleanup();
    }

    /// The light/dark switch follows the sun and must not depend on a theme being
    /// applicable: the wallpaper can be broken while the scheme still works.
    #[test]
    fn the_color_scheme_is_updated_even_when_every_monitor_fails() {
        let fixture = Fixture::new(
            "scheme_survives",
            &["MON-A"],
            individual(&[("MON-A", "Missing")]),
            &[],
        );

        assert!(fixture.run().is_err());
        assert_eq!(
            fixture.scheme.applied().len(),
            1,
            "the scheme update runs before the cycle is failed"
        );

        fixture.cleanup();
    }

    #[test]
    fn the_image_format_comes_from_the_configuration_for_a_legacy_theme() {
        let fixture = Fixture::new(
            "legacy_format",
            &["MON-A"],
            individual(&[("MON-A", "Big Sur")]),
            &[("Big Sur", &[1, 2])],
        );

        assert!(fixture.run().is_ok());
        assert_eq!(
            fixture.wallpapers.images_for("MON-A"),
            vec!["2.jpg".to_string()],
            "midday selects the second frame, named 1-based"
        );

        fixture.cleanup();
    }

    #[test]
    fn the_frame_follows_the_sun_at_the_given_time() {
        let fixture = Fixture::new(
            "frames",
            &["MON-A"],
            individual(&[("MON-A", "Big Sur")]),
            &[("Big Sur", &[1, 2])],
        );
        let engine = fixture.engine();

        engine
            .process_theme_cycle_at(&fixture.position, &summer_noon())
            .unwrap();
        engine
            .process_theme_cycle_at(&fixture.position, &summer_midnight())
            .unwrap();

        // `TWO_FRAMES` declares altitude -30 then +30, and the files are 1-based:
        // the high sun picks the second frame, the night picks the first.
        assert_eq!(
            fixture.wallpapers.images_for("MON-A"),
            vec!["2.jpg".to_string(), "1.jpg".to_string()]
        );

        fixture.cleanup();
    }

    #[test]
    fn the_all_variant_applies_one_theme_to_every_monitor() {
        let fixture = Fixture::new(
            "all_variant",
            &["MON-A", "MON-B"],
            WallpaperMode::Fixed {
                monitor_specific_wallpapers: MonitorSpecificWallpapers::All("Big Sur".to_string()),
            },
            &[("Big Sur", &[1, 2])],
        );

        assert!(fixture.run().is_ok());
        assert_eq!(
            fixture.wallpapers.monitors_touched(),
            vec!["MON-A", "MON-B"]
        );

        fixture.cleanup();
    }

    // ── Solar event schedule ────────────────────────────────────────────────
    //
    // The schedule exists so a frame or light/dark change is applied at the instant it
    // happens instead of up to one interval later. What has to hold is that it finds
    // the *first* change of exactly the state the cycle produces, so these compare it
    // against a second-by-second scan of the same function rather than against
    // hand-picked times.

    /// The first second at which the daemon's output differs from what it is at `now`.
    fn first_change_second_by_second(
        engine: &ThemeEngine<
            '_,
            StaticMonitors,
            RecordingWallpapers,
            RecordingScheme,
            FixedPosition,
        >,
        position: &Position,
        now: &OffsetDateTime,
        horizon: Duration,
    ) -> Option<u64> {
        let current_scheme = engine.current_scheme_for_schedule();
        let initial = engine.output_at(position, now, current_scheme);

        for second in 1..=horizon.as_secs() {
            let at = now.add_seconds(second).unwrap();

            if engine.output_at(position, &at, current_scheme) != initial {
                return Some(second);
            }
        }

        None
    }

    #[test]
    fn the_schedule_finds_the_first_change_a_second_by_second_scan_finds() {
        let fixture = Fixture::new(
            "schedule_matches",
            &["MON-A"],
            individual(&[("MON-A", "Big Sur")]),
            &[("Big Sur", &[1, 2])],
        );
        let engine = fixture.engine();

        // Twelve hours from midday: the sun crosses this theme's frame boundary at
        // sunset, well beyond six hours, and the light/dark threshold shortly after.
        let horizon = Duration::from_secs(12 * 3600);
        let now = summer_noon();

        engine
            .process_theme_cycle_at(&fixture.position, &now)
            .unwrap();

        let scheduled = engine.next_event_within(&now, horizon);
        let scanned = first_change_second_by_second(&engine, &fixture.position, &now, horizon);

        assert_eq!(
            scheduled,
            scanned.map(Duration::from_secs),
            "the schedule has to agree with a second-by-second scan of the same state"
        );
        assert!(scheduled.is_some(), "something changes within twelve hours");

        fixture.cleanup();
    }

    #[test]
    fn the_schedule_agrees_with_the_scan_from_midnight_too() {
        let fixture = Fixture::new(
            "schedule_midnight",
            &["MON-A"],
            individual(&[("MON-A", "Big Sur")]),
            &[("Big Sur", &[1, 2])],
        );
        let engine = fixture.engine();

        let horizon = Duration::from_secs(6 * 3600);
        let now = summer_midnight();

        engine
            .process_theme_cycle_at(&fixture.position, &now)
            .unwrap();

        assert_eq!(
            engine.next_event_within(&now, horizon),
            first_change_second_by_second(&engine, &fixture.position, &now, horizon)
                .map(Duration::from_secs)
        );

        fixture.cleanup();
    }

    #[test]
    fn a_single_frame_theme_without_the_color_switch_schedules_nothing() {
        let fixture = Fixture::new_with_frames(
            "schedule_one_frame",
            &["MON-A"],
            individual(&[("MON-A", "Single")]),
            &[("Single", &[1])],
            ONE_FRAME,
        )
        .without_color_scheme();
        let engine = fixture.engine();
        let now = summer_noon();

        engine
            .process_theme_cycle_at(&fixture.position, &now)
            .unwrap();

        assert_eq!(
            engine.next_event_within(&now, Duration::from_secs(24 * 3600)),
            None,
            "one frame and no light/dark switch means nothing ever changes"
        );

        fixture.cleanup();
    }

    #[test]
    fn a_theme_that_disappeared_schedules_nothing() {
        let fixture = Fixture::new(
            "schedule_missing_theme",
            &["MON-A"],
            individual(&[("MON-A", "Big Sur")]),
            &[("Big Sur", &[1, 2])],
        );
        let engine = fixture.engine();
        let now = summer_noon();

        engine
            .process_theme_cycle_at(&fixture.position, &now)
            .unwrap();

        std::fs::remove_dir_all(fixture.themes_directory.join("Big Sur")).unwrap();

        assert_eq!(
            engine.next_event_within(&now, Duration::from_secs(3600)),
            None,
            "an unreadable theme is not a reason to wake early"
        );

        fixture.cleanup();
    }

    /// The interval stays the ceiling: with the default fifteen seconds and the next
    /// change hours away, the loop waits its interval.
    #[test]
    fn the_wait_is_the_interval_when_no_change_is_due_sooner() {
        let fixture = Fixture::new(
            "schedule_capped",
            &["MON-A"],
            individual(&[("MON-A", "Big Sur")]),
            &[("Big Sur", &[1, 2])],
        );
        let engine = fixture.engine();

        engine
            .process_theme_cycle_at(&fixture.position, &summer_noon())
            .unwrap();

        assert_eq!(engine.next_wait(), Duration::from_secs(15));

        fixture.cleanup();
    }

    // ── scheduled_wait (pure) ───────────────────────────────────────────────

    #[test]
    fn a_closer_change_shortens_the_wait() {
        assert_eq!(
            scheduled_wait(Duration::from_secs(600), Some(Duration::from_secs(300))),
            Duration::from_secs(300)
        );
    }

    #[test]
    fn a_change_beyond_the_interval_cannot_extend_the_wait() {
        assert_eq!(
            scheduled_wait(Duration::from_secs(60), Some(Duration::from_secs(600))),
            Duration::from_secs(60)
        );
    }

    #[test]
    fn a_change_that_is_already_due_does_not_produce_a_zero_wait() {
        assert_eq!(
            scheduled_wait(Duration::from_secs(600), Some(Duration::ZERO)),
            crate::schedule::MIN_WAIT
        );
    }

    #[test]
    fn no_scheduled_change_means_the_plain_interval() {
        assert_eq!(
            scheduled_wait(Duration::from_secs(600), None),
            Duration::from_secs(600)
        );
    }

    /// The frame boundary of `MIDDAY_FRAMES` sits at 71°, which Beijing's summer sun
    /// crosses again a bit over an hour after local noon — inside the longest interval
    /// the configuration allows, so this is the case the schedule exists for.
    #[test]
    fn an_event_inside_a_short_horizon_is_reported() {
        let fixture = Fixture::new_with_frames(
            "schedule_inside",
            &["MON-A"],
            individual(&[("MON-A", "Midday")]),
            &[("Midday", &[1, 2])],
            MIDDAY_FRAMES,
        )
        .without_color_scheme();
        let engine = fixture.engine();
        let now = summer_noon();

        engine
            .process_theme_cycle_at(&fixture.position, &now)
            .unwrap();

        let horizon = Duration::from_secs(90 * 60);
        let scheduled = engine.next_event_within(&now, horizon);

        assert_eq!(
            scheduled,
            first_change_second_by_second(&engine, &fixture.position, &now, horizon)
                .map(Duration::from_secs),
            "the schedule has to agree with the scan inside a short horizon too"
        );
        assert!(
            scheduled.is_some_and(|event| event < horizon),
            "the sun crosses this theme's frame boundary within ninety minutes of noon"
        );

        fixture.cleanup();
    }
}
