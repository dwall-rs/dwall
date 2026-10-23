use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::{env, io::Write, str::FromStr};

// Only the debug formatter asks whether stderr is a terminal.
#[cfg(debug_assertions)]
use std::io::IsTerminal;

use log::{LevelFilter, Log, SetLoggerError, set_boxed_logger, set_max_level};
use time::{OffsetDateTime, UtcDateTime};

mod macros;
#[cfg(debug_assertions)]
mod rich;

/// The `log` crate, re-exported so the macros in this crate can be expanded
/// without the caller having to depend on the exact same `log` version.
#[doc(hidden)]
pub use log;

#[doc(hidden)]
pub const LOG_MAX_LEVEL_INFO: bool = cfg!(feature = "max-level-info");

pub struct Logger {
    level: LevelFilter,
    targets: Option<Vec<(String, LevelFilter)>>,
    #[cfg(not(debug_assertions))]
    output: ProductionOutput,
}

/// The size at which the live log is rotated at runtime.
///
/// Rotation at process start alone leaves a daemon that is never restarted growing its
/// log without bound. Together with [`MAX_BACKUP_LOGS`] this bounds the log
/// directory at six files.
#[cfg_attr(debug_assertions, allow(dead_code))]
const MAX_LOG_BYTES: u64 = 5 * 1024 * 1024;

/// The live log file, rotated once it grows past [`MAX_LOG_BYTES`].
///
/// The handle lives behind a mutex because [`Log::log`] takes `&self`, and it is an
/// `Option` so a rotation can close the file before renaming it: Windows refuses
/// to rename a file that is still open. Every failure drops the line instead of panicking.
#[cfg_attr(debug_assertions, allow(dead_code))]
struct LogFile {
    path: PathBuf,
    file: std::sync::Mutex<Option<std::fs::File>>,
    /// Bytes written to the current file, so the size check needs no metadata call.
    written: AtomicU64,
}

#[cfg_attr(debug_assertions, allow(dead_code))]
impl LogFile {
    /// Wraps an already-open file, counting what it already holds.
    fn new(path: PathBuf, file: std::fs::File) -> Self {
        let written = file.metadata().map(|metadata| metadata.len()).unwrap_or(0);

        Self {
            path,
            file: std::sync::Mutex::new(Some(file)),
            written: AtomicU64::new(written),
        }
    }

    /// Opens the log for appending, creating it when a rotation removed it.
    fn open_append(path: &Path) -> std::io::Result<std::fs::File> {
        std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
    }

    /// Opens a fresh append handle for `path`, for the runtime-rotation tests.
    #[cfg(test)]
    fn open(path: &Path) -> std::io::Result<Self> {
        let file = Self::open_append(path)?;

        Ok(Self::new(path.to_path_buf(), file))
    }

    /// Appends one already-formatted line, rotating first once the file is too large.
    ///
    /// Never panics: a poisoned mutex or an unusable path drops the line.
    fn write_line(&self, line: &str) {
        let Ok(mut file) = self.file.lock() else {
            return;
        };

        if self.written.load(Ordering::Relaxed) >= MAX_LOG_BYTES {
            self.rotate(&mut file);
        }

        if let Some(file) = file.as_mut() {
            let _ = writeln!(file, "{line}");
            self.written
                .fetch_add(line.len() as u64 + 1, Ordering::Relaxed);
        }
    }

    /// Moves the live file into `backup/` and reopens a fresh one.
    ///
    /// The handle is closed first, then [`rotate_log_file`] applies the same
    /// five-file cap the startup path uses. Failures are ignored: a log directory that
    /// cannot be rotated must not stop the daemon from logging.
    fn rotate(&self, slot: &mut Option<std::fs::File>) {
        if let Some(mut file) = slot.take() {
            let _ = file.flush();
        }

        let _ = rotate_log_file(&self.path);

        // The counter restarts even when the reopen fails, so an unusable directory is
        // retried once per limit instead of once per line.
        self.written.store(0, Ordering::Relaxed);

        if let Ok(file) = Self::open_append(&self.path) {
            *slot = Some(file);
        }
    }

    fn flush(&self) {
        if let Ok(mut file) = self.file.lock()
            && let Some(file) = file.as_mut()
        {
            let _ = file.flush();
        }
    }
}

#[cfg(not(debug_assertions))]
enum ProductionOutput {
    Stderr,
    File(LogFile),
}

impl Logger {
    pub fn with_level(mut self, level: LevelFilter) -> Self {
        self.level = level;
        self
    }

    /// Adds a target filter using the current log level set on this `Logger`.
    ///
    /// # ⚠️ Order-Dependent
    ///
    /// This method captures `self.level` **at the time of the call**.
    /// If you want the target to use a custom level, you **must** call
    /// [`with_level`](Logger::with_level) **before** calling this method.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// // ✅ Correct: level is set before adding the target
    /// Logger::default()
    ///     .with_level(LevelFilter::Info)
    ///     .with_target("my_crate");
    ///
    /// // ❌ Wrong: target is added before level is set,
    /// //    so it captures the default level instead of Info
    /// Logger::default()
    ///     .with_target("my_crate")
    ///     .with_level(LevelFilter::Info);
    /// ```
    ///
    /// To set a specific level for a target without this ordering constraint,
    /// use [`with_target_level`](Logger::with_target_level) instead.
    pub fn with_target(mut self, target: &str) -> Self {
        if let Some(targets) = &mut self.targets {
            targets.push((target.to_string(), self.level));
        } else {
            self.targets = Some(vec![(target.to_string(), self.level)]);
        }
        self
    }

    pub fn with_target_level(mut self, target: &str, level: LevelFilter) -> Self {
        if let Some(targets) = &mut self.targets {
            targets.push((target.to_string(), level));
        } else {
            self.targets = Some(vec![(target.to_string(), level)]);
        }
        self
    }

    #[cfg(not(debug_assertions))]
    pub fn with_file_path(mut self, path: std::path::PathBuf) -> std::io::Result<Self> {
        use std::fs::OpenOptions;

        rotate_log_file(&path)?;

        // The previous file was just rotated into `backup/`, so this starts a fresh
        // log; `truncate` states that intent (without it, a file that survived the
        // rename for any reason would be overwritten from offset 0 and keep the
        // tail of the previous run).
        let file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&path)?;

        self.output = ProductionOutput::File(LogFile::new(path, file));
        Ok(self)
    }

    pub fn init(self) -> std::result::Result<(), SetLoggerError> {
        set_max_level(self.effective_max_level());
        set_boxed_logger(Box::new(self))
    }

    /// The level handed to `log`'s global filter.
    ///
    /// That filter runs *before* [`Log::enabled`], so it has to allow the highest
    /// configured level — otherwise a per-target level above the global one could
    /// never take effect. Per-target filtering then narrows the records down again.
    fn effective_max_level(&self) -> LevelFilter {
        let highest_target = self
            .targets
            .as_ref()
            .and_then(|targets| targets.iter().map(|(_, level)| *level).max());

        highest_target.map_or(self.level, |level| level.max(self.level))
    }
}

impl Log for Logger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        match self.targets {
            None => metadata.level() <= self.level,
            Some(ref targets) => targets
                .iter()
                .find(|(name, _level)| metadata.target().starts_with(name))
                .is_some_and(|(_name, level)| metadata.level() <= *level),
        }
    }

    fn log(&self, record: &log::Record) {
        if !self.enabled(record.metadata()) {
            return;
        }

        let timestamp = OffsetDateTime::now_local()
            .map(|dt| dt.to_rfc3339())
            .unwrap_or_else(|_| UtcDateTime::now().to_rfc3339());

        #[cfg(debug_assertions)]
        {
            let file = record.file().unwrap_or("<unknown>");
            let line = record.line().unwrap_or(0);
            let msg = record.args().to_string();

            //   2026-06-01T01:27:42+08:00 ERROR  Reading configuration file, path: C:\Users\…\config.toml
            //     at src\config.rs:79

            if std::io::stderr().is_terminal() {
                use crate::rich::{
                    BOLD, CYAN, DIM, ITALIC, MAGENTA, RED, RESET, WHITE, YELLOW, restore_color,
                };

                let (level_color, level_str) = match record.level() {
                    log::Level::Error => (RED, "ERROR"),
                    log::Level::Warn => (YELLOW, "WARN "),
                    log::Level::Info => (CYAN, "INFO "),
                    log::Level::Debug => (WHITE, "DEBUG"),
                    log::Level::Trace => (MAGENTA, "TRACE"),
                };

                let msg = restore_color(&msg, level_color);
                let _ = writeln!(
                    std::io::stderr(),
                    "  {DIM}{timestamp}{RESET} \
                     {BOLD}{level_color}{level_str}{RESET} {level_color}{msg}{RESET}\n    \
                     {DIM}{ITALIC}at{RESET} {file}:{line}\n",
                );
            } else {
                // Redirected output (file, pipe, CI) must not receive escape codes.
                let level_str = match record.level() {
                    log::Level::Error => "ERROR",
                    log::Level::Warn => "WARN",
                    log::Level::Info => "INFO",
                    log::Level::Debug => "DEBUG",
                    log::Level::Trace => "TRACE",
                };

                let _ = writeln!(
                    std::io::stderr(),
                    "{timestamp} {level_str} [{target}] {msg} at {file}:{line}",
                    target = record.target(),
                );
            }
        }

        #[cfg(not(debug_assertions))]
        {
            let level_str = match record.level() {
                log::Level::Error => "ERROR",
                log::Level::Warn => "WARN",
                log::Level::Info => "INFO",
                log::Level::Debug => "DEBUG",
                log::Level::Trace => "TRACE",
            };

            let location = match (record.file(), record.line()) {
                (Some(file), Some(line)) => format!(" at {file}:{line}"),
                _ => String::new(),
            };

            match &self.output {
                ProductionOutput::Stderr => {
                    let mut stderr = std::io::stderr().lock();
                    let _ = writeln!(
                        stderr,
                        "{timestamp} {level_str} [{}] {}{location}",
                        record.target(),
                        record.args(),
                    );
                }
                ProductionOutput::File(log_file) => {
                    log_file.write_line(&format!(
                        "{timestamp} {level_str} [{}] {}{location}",
                        record.target(),
                        record.args(),
                    ));
                }
            }
        }
    }

    fn flush(&self) {
        #[cfg(debug_assertions)]
        {
            let _ = std::io::stderr().flush();
        }

        #[cfg(not(debug_assertions))]
        {
            match &self.output {
                ProductionOutput::Stderr => {
                    let _ = std::io::stderr().flush();
                }
                ProductionOutput::File(log_file) => log_file.flush(),
            }
        }
    }
}

impl Default for Logger {
    fn default() -> Self {
        Self {
            level: get_log_level(),
            targets: None,
            #[cfg(not(debug_assertions))]
            output: ProductionOutput::Stderr,
        }
    }
}

/// The level used when neither the runtime nor the build-time environment names one.
///
/// `build.rs` embeds `DWALL_LOG=debug` for every build without `max-level-info`
/// (debug and release alike), so this default is only reached by a `max-level-info`
/// build with no environment override — and `Info` is the level the release pipeline
/// is meant to surface. A `Trace` default for debug builds used to live here, but the
/// injected variable made that branch unreachable.
const fn default_log_level() -> LevelFilter {
    LevelFilter::Info
}

/// Get log level from environment variable
///
/// Accepts a plain level (`debug`) as well as the `env_logger`-style forms people
/// reach for out of habit (`dwall=debug`, `dwall=debug,tauri=info`), but only the
/// first directive is honoured: it supplies the logger's global level.
///
/// This is a documented limitation, not an accident. The [`Logger`] built from this
/// level does support per-target levels, added with [`with_target_level`], and the
/// environment's remaining directives are deliberately not turned into those filters:
/// a target filter makes [`Log::enabled` match by prefix only, which would drop
/// every other target. A value like `dwall=info,tauri=warn` therefore still means
/// info for everything.
fn get_log_level() -> LevelFilter {
    let from_env = env::var("DWALL_LOG").or_else(|_| env::var("RUST_LOG")).ok();

    from_env
        .as_deref()
        .or(option_env!("DWALL_LOG"))
        .and_then(parse_log_level)
        .unwrap_or_else(default_log_level)
}

/// Parses `debug`, `dwall=debug` or `dwall=debug,tauri=info` into the global level.
///
/// Only the first directive is read; the rest are dropped, for the reason documented
/// on [`get_log_level`]. A leading `target=` prefix on that directive is ignored, so
/// its level applies to every target.
fn parse_log_level(value: &str) -> Option<LevelFilter> {
    // Use the first directive, then the part after its optional `target=` prefix.
    let directive = value.trim().split(',').next()?.trim();
    let level = directive.rsplit('=').next()?.trim();

    LevelFilter::from_str(level).ok()
}

/// In debug builds, wraps the key name with ANSI bold-italic escape codes.
/// In release builds, returns the key as-is with zero overhead.
#[cfg(debug_assertions)]
#[doc(hidden)]
#[inline(always)]
pub fn __fmt_key(key: &'static str) -> String {
    use crate::rich::{BOLD, ITALIC, RESET};

    format!("{BOLD}{ITALIC}{key}{RESET}")
}

#[cfg(not(debug_assertions))]
#[doc(hidden)]
#[inline(always)]
pub fn __fmt_key(key: &'static str) -> &'static str {
    key
}

/// How many rotated logs are kept next to the current one.
///
/// Every process start rotates the live file into `backup/`, and the settings app
/// restarts the daemon on every theme application, so an uncapped directory grows
/// without bound — a scheduled `--once` every minute would add 1,440 files a day.
///
/// Only release builds rotate (see [`Logger::with_file_path`]), hence the lint
/// allowance in debug builds; the tests below use it in both profiles.
#[cfg_attr(debug_assertions, allow(dead_code))]
const MAX_BACKUP_LOGS: usize = 5;

/// Moves the live log into `backup/` and prunes the oldest backups.
///
/// Split out of [`Logger::with_file_path`] (which release builds alone compile) so
/// the rotation policy is testable.
#[cfg_attr(debug_assertions, allow(dead_code))]
fn rotate_log_file(path: &std::path::Path) -> std::io::Result<()> {
    if !path.exists() {
        return Ok(());
    }

    let backup_dir = path
        .parent()
        .unwrap_or_else(|| std::path::Path::new("."))
        .join("backup");

    std::fs::create_dir_all(&backup_dir)?;

    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("app");

    // Two starts within the same second would otherwise pick the same name and the
    // rename would overwrite the earlier log; bumping keeps the name parseable by
    // the prune below and the file unique.
    let mut timestamp = timestamp;
    let mut backup_path = backup_dir.join(format!("{stem}_{timestamp}.log"));
    while backup_path.exists() {
        timestamp += 1;
        backup_path = backup_dir.join(format!("{stem}_{timestamp}.log"));
    }

    std::fs::rename(path, backup_path)?;

    // Best effort: a log directory that cannot be pruned must not stop the daemon
    // from logging at all. `eprintln` because this runs before `init()` installs the
    // logger, so a `warn!` would go nowhere.
    if let Err(e) = prune_backups(&backup_dir, stem) {
        eprintln!(
            "warning: failed to prune old logs in {}: {e}",
            backup_dir.display()
        );
    }

    Ok(())
}

/// Keeps the newest [`MAX_BACKUP_LOGS`] backups for `stem`, deleting the rest.
#[cfg_attr(debug_assertions, allow(dead_code))]
fn prune_backups(backup_dir: &std::path::Path, stem: &str) -> std::io::Result<usize> {
    let prefix = format!("{stem}_");
    let mut backups: Vec<(u64, std::path::PathBuf)> = Vec::new();

    for entry in std::fs::read_dir(backup_dir)? {
        let entry = entry?;
        let name = entry.file_name();
        let name = name.to_string_lossy();

        // `<stem>_<unix seconds>.log`, so the timestamps sort as numbers and a file
        // from another program is left alone.
        let Some(timestamp) = name
            .strip_prefix(&prefix)
            .and_then(|rest| rest.strip_suffix(".log"))
            .and_then(|stamp| stamp.parse::<u64>().ok())
        else {
            continue;
        };

        backups.push((timestamp, entry.path()));
    }

    if backups.len() <= MAX_BACKUP_LOGS {
        return Ok(0);
    }

    backups.sort_by_key(|(timestamp, _)| *timestamp);

    let mut pruned = 0;
    for (_, path) in backups.iter().take(backups.len() - MAX_BACKUP_LOGS) {
        match std::fs::remove_file(path) {
            Ok(()) => pruned += 1,
            // A locked or already removed file is not worth failing over.
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(e) => {
                return Err(std::io::Error::new(
                    e.kind(),
                    format!("failed to remove {}: {e}", path.display()),
                ));
            }
        }
    }

    Ok(pruned)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_plain_levels() {
        assert_eq!(parse_log_level("debug"), Some(LevelFilter::Debug));
        assert_eq!(parse_log_level("WARN"), Some(LevelFilter::Warn));
        assert_eq!(parse_log_level("  info  "), Some(LevelFilter::Info));
        assert_eq!(parse_log_level("nonsense"), None);
    }

    #[test]
    fn parses_env_logger_style_filters() {
        // The forms people habitually type; they used to fall back silently.
        assert_eq!(parse_log_level("dwall=debug"), Some(LevelFilter::Debug));
        assert_eq!(
            parse_log_level("dwall=info,tauri=warn"),
            Some(LevelFilter::Info)
        );
        assert_eq!(
            parse_log_level("dwall::config=trace"),
            Some(LevelFilter::Trace)
        );
    }

    #[test]
    fn effective_max_level_keeps_per_target_levels_reachable() {
        let logger = Logger::default()
            .with_level(LevelFilter::Warn)
            .with_target_level("dwall::config", LevelFilter::Debug);

        // Without this, `log`'s global filter would drop DEBUG before the
        // per-target decision could allow it.
        assert_eq!(logger.effective_max_level(), LevelFilter::Debug);

        let plain = Logger::default().with_level(LevelFilter::Error);
        assert_eq!(plain.effective_max_level(), LevelFilter::Error);
    }

    // ── Log rotation ─────────────────────────────────────────────────────────

    fn scratch(name: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!("dwall_logging_{name}"));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();

        dir
    }

    fn backup_names(log_path: &std::path::Path) -> Vec<String> {
        let backup_dir = log_path.parent().unwrap().join("backup");
        let mut names: Vec<String> = std::fs::read_dir(&backup_dir)
            .unwrap()
            .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
            .collect();
        names.sort();

        names
    }

    #[test]
    fn a_missing_log_file_is_not_rotated() {
        let dir = scratch("missing");
        let log_path = dir.join("dwall.log");

        rotate_log_file(&log_path).unwrap();

        assert!(!log_path.parent().unwrap().join("backup").exists());
        assert!(!log_path.exists());

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn the_previous_log_is_moved_into_the_backup_directory() {
        let dir = scratch("rotate");
        let log_path = dir.join("dwall.log");
        std::fs::write(&log_path, "previous run").unwrap();

        rotate_log_file(&log_path).unwrap();

        let names = backup_names(&log_path);
        assert_eq!(names.len(), 1, "unexpected backups: {names:?}");
        assert!(names[0].starts_with("dwall_") && names[0].ends_with(".log"));
        assert!(!log_path.exists(), "the live file is moved, not copied");

        let _ = std::fs::remove_dir_all(&dir);
    }

    /// The policy this adds: the directory used to grow by one file per process
    /// start, forever.
    #[test]
    fn only_the_newest_backups_are_kept() {
        let dir = scratch("prune");
        let log_path = dir.join("dwall.log");
        let backup_dir = dir.join("backup");
        std::fs::create_dir_all(&backup_dir).unwrap();

        for timestamp in 1..=(MAX_BACKUP_LOGS as u64 + 4) {
            std::fs::write(backup_dir.join(format!("dwall_{timestamp}.log")), "old").unwrap();
        }

        // A different stem, and a file that only looks similar, must survive.
        std::fs::write(backup_dir.join("dwall-settings_1.log"), "other").unwrap();
        std::fs::write(backup_dir.join("notes.txt"), "keep me").unwrap();

        let pruned = prune_backups(&backup_dir, "dwall").unwrap();

        assert_eq!(pruned, 4, "the four oldest dwall backups are removed");
        assert_eq!(
            backup_names(&log_path),
            vec![
                "dwall-settings_1.log".to_string(),
                "dwall_5.log".to_string(),
                "dwall_6.log".to_string(),
                "dwall_7.log".to_string(),
                "dwall_8.log".to_string(),
                "dwall_9.log".to_string(),
                "notes.txt".to_string(),
            ],
            "only the newest dwall backups survive, and nothing else is touched"
        );

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_directory_of_backups_is_left_alone_when_it_fits() {
        let dir = scratch("fits");
        let backup_dir = dir.join("backup");
        std::fs::create_dir_all(&backup_dir).unwrap();

        for timestamp in 1..=MAX_BACKUP_LOGS as u64 {
            std::fs::write(backup_dir.join(format!("dwall_{timestamp}.log")), "old").unwrap();
        }

        assert_eq!(prune_backups(&backup_dir, "dwall").unwrap(), 0);
        assert_eq!(backup_names(&dir.join("dwall.log")).len(), MAX_BACKUP_LOGS);

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn rotation_keeps_the_backup_count_bounded_across_restarts() {
        let dir = scratch("repeated");
        let log_path = dir.join("dwall.log");

        for run in 0..(MAX_BACKUP_LOGS * 3) {
            std::fs::write(&log_path, format!("run {run}")).unwrap();
            rotate_log_file(&log_path).unwrap();
        }

        let names = backup_names(&log_path);
        assert_eq!(
            names.len(),
            MAX_BACKUP_LOGS,
            "rotation must not grow the directory without bound: {names:?}"
        );

        let _ = std::fs::remove_dir_all(&dir);
    }

    // ── Runtime size rotation ───────────────────────────────────────────────

    /// A live log already at the limit; the next line written rotates it.
    fn log_file_at_the_limit(log_path: &Path) -> LogFile {
        std::fs::File::create(log_path)
            .unwrap()
            .set_len(MAX_LOG_BYTES)
            .unwrap();

        LogFile::open(log_path).unwrap()
    }

    #[test]
    fn the_live_log_is_rotated_once_it_passes_the_size_limit() {
        let dir = scratch("runtime_rotate");
        let log_path = dir.join("dwall.log");
        let log_file = log_file_at_the_limit(&log_path);

        assert!(
            !dir.join("backup").exists(),
            "opening a full log does not rotate it by itself"
        );

        log_file.write_line("first line past the limit");

        let names = backup_names(&log_path);
        assert_eq!(names.len(), 1, "the full log moved into backup: {names:?}");
        assert!(names[0].starts_with("dwall_") && names[0].ends_with(".log"));
        assert_eq!(
            std::fs::metadata(&log_path).unwrap().len(),
            "first line past the limit".len() as u64 + 1,
            "the fresh log holds only what was written after the rotation"
        );

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn runtime_rotation_keeps_the_backup_count_bounded() {
        let dir = scratch("runtime_cap");
        let log_path = dir.join("dwall.log");
        let log_file = LogFile::open(&log_path).unwrap();

        for run in 0..(MAX_BACKUP_LOGS * 2) {
            // Pretend the file already reached the limit, then write one line to
            // trigger the rotation the size check performs.
            log_file.written.store(MAX_LOG_BYTES, Ordering::Relaxed);
            log_file.write_line(&format!("run {run}"));
        }

        let names = backup_names(&log_path);
        assert_eq!(
            names.len(),
            MAX_BACKUP_LOGS,
            "runtime rotation has to keep the same five-file cap: {names:?}"
        );

        let _ = std::fs::remove_dir_all(&dir);
    }

    // ── Level parsing and target filters ────────────────────────────────────

    /// Only the first directive is read; anything after the first comma is ignored,
    /// including a valid level that could otherwise mask a typo.
    #[test]
    fn parse_log_level_takes_the_first_directive_only() {
        assert_eq!(parse_log_level("warn,debug"), Some(LevelFilter::Warn));
        assert_eq!(parse_log_level("nonsense,debug"), None);
        assert_eq!(
            parse_log_level("dwall=warn,dwall=debug"),
            Some(LevelFilter::Warn)
        );
    }

    /// with_target captures the level in effect at the call, which is the documented
    /// ordering trap: setting the level afterwards does not reach the target.
    #[test]
    fn with_target_captures_the_level_in_effect_at_the_call() {
        let _lock = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let default_level = Logger::default().level;

        let ordered = Logger::default()
            .with_level(LevelFilter::Error)
            .with_target("dwall");
        assert_eq!(ordered.targets.as_ref().unwrap()[0].1, LevelFilter::Error);

        let misordered = Logger::default()
            .with_target("dwall")
            .with_level(LevelFilter::Error);
        assert_eq!(misordered.level, LevelFilter::Error);
        assert_eq!(
            misordered.targets.as_ref().unwrap()[0].1,
            default_level,
            "the target kept the default level, not the one set afterwards"
        );
        assert_ne!(default_level, LevelFilter::Error);
    }

    /// with_target_level exists precisely to avoid that trap.
    #[test]
    fn with_target_level_does_not_depend_on_the_order() {
        let logger = Logger::default()
            .with_target_level("dwall::config", LevelFilter::Debug)
            .with_level(LevelFilter::Error);

        assert_eq!(logger.targets.as_ref().unwrap()[0].1, LevelFilter::Debug);
        assert_eq!(logger.effective_max_level(), LevelFilter::Debug);
    }

    fn metadata(target: &'static str, level: log::Level) -> log::Metadata<'static> {
        log::MetadataBuilder::new()
            .level(level)
            .target(target)
            .build()
    }

    /// A target filter covers the target itself and everything below it, and nothing
    /// above it.
    #[test]
    fn enabled_matches_targets_by_prefix() {
        let logger = Logger::default().with_target_level("dwall::config", LevelFilter::Debug);

        assert!(logger.enabled(&metadata("dwall::config", log::Level::Debug)));
        assert!(
            logger.enabled(&metadata("dwall::config::inner", log::Level::Debug)),
            "a child module matches the target prefix"
        );
        assert!(
            !logger.enabled(&metadata("dwall::config", log::Level::Trace)),
            "a level above the target own is filtered out"
        );
        assert!(
            !logger.enabled(&metadata("dwall::other", log::Level::Debug)),
            "a sibling target is not covered"
        );
        assert!(
            !logger.enabled(&metadata("dwall", log::Level::Debug)),
            "the parent does not match the child prefix"
        );
    }

    // ── Environment ─────────────────────────────────────────────────────────

    /// Serialises the tests that mutate the process environment; the test harness runs
    /// tests in parallel threads.
    static ENV_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

    /// Restores DWALL_LOG and RUST_LOG when the test ends, panic included.
    struct EnvGuard {
        d_wall: Option<std::ffi::OsString>,
        rust_log: Option<std::ffi::OsString>,
    }

    impl EnvGuard {
        fn set(d_wall: Option<&str>, rust_log: Option<&str>) -> Self {
            let guard = Self {
                d_wall: env::var_os("DWALL_LOG"),
                rust_log: env::var_os("RUST_LOG"),
            };

            Self::apply(
                d_wall.map(std::ffi::OsStr::new),
                rust_log.map(std::ffi::OsStr::new),
            );
            guard
        }

        fn apply(d_wall: Option<&std::ffi::OsStr>, rust_log: Option<&std::ffi::OsStr>) {
            unsafe {
                match d_wall {
                    Some(value) => env::set_var("DWALL_LOG", value),
                    None => env::remove_var("DWALL_LOG"),
                }
                match rust_log {
                    Some(value) => env::set_var("RUST_LOG", value),
                    None => env::remove_var("RUST_LOG"),
                }
            }
        }
    }

    impl Drop for EnvGuard {
        fn drop(&mut self) {
            Self::apply(self.d_wall.as_deref(), self.rust_log.as_deref());
        }
    }

    #[test]
    fn get_log_level_prefers_dwall_log_over_rust_log() {
        let _lock = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());

        {
            let _guard = EnvGuard::set(Some("warn"), Some("trace"));
            assert_eq!(get_log_level(), LevelFilter::Warn);
        }
        {
            let _guard = EnvGuard::set(None, Some("error"));
            assert_eq!(get_log_level(), LevelFilter::Error);
        }
        {
            let _guard = EnvGuard::set(Some("debug"), None);
            assert_eq!(get_log_level(), LevelFilter::Debug);
        }
    }

    // ── Formatting helpers ──────────────────────────────────────────────────

    #[cfg(debug_assertions)]
    #[test]
    fn restore_color_reapplies_the_base_colour_after_a_reset() {
        use crate::rich::{CYAN, RESET, restore_color};

        assert_eq!(restore_color("plain", CYAN), "plain");
        assert_eq!(
            restore_color(&format!("a{RESET}b"), CYAN),
            format!("a{RESET}{CYAN}b"),
            "every reset has to be followed by the base colour again"
        );
    }

    #[test]
    fn fmt_key_is_wrapped_only_in_debug_builds() {
        #[cfg(debug_assertions)]
        {
            use crate::rich::{BOLD, ITALIC, RESET};

            assert_eq!(
                __fmt_key("theme_id"),
                format!("{BOLD}{ITALIC}theme_id{RESET}")
            );
        }

        #[cfg(not(debug_assertions))]
        {
            assert_eq!(__fmt_key("theme_id"), "theme_id");
        }
    }
}
