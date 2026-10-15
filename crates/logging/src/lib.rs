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

#[cfg(not(debug_assertions))]
enum ProductionOutput {
    Stderr,
    File(std::sync::Mutex<std::fs::File>),
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

        if path.exists() {
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

            let bak_name = format!("{stem}_{timestamp}.log");
            std::fs::rename(&path, backup_dir.join(bak_name))?;
        }

        // The previous file was just rotated into `backup/`, so this starts a fresh
        // log; `truncate` states that intent (without it, a file that survived the
        // rename for any reason would be overwritten from offset 0 and keep the
        // tail of the previous run).
        let file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&path)?;

        self.output = ProductionOutput::File(std::sync::Mutex::new(file));
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
                ProductionOutput::File(mutex) => {
                    if let Ok(mut file) = mutex.lock() {
                        let _ = writeln!(
                            file,
                            "{timestamp} {level_str} [{}] {}{location}",
                            record.target(),
                            record.args(),
                        );
                    }
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
                ProductionOutput::File(mutex) => {
                    if let Ok(mut file) = mutex.lock() {
                        let _ = file.flush();
                    }
                }
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

/// Get default log level
///
/// Release builds built with `max-level-info` keep `Info` (the level the release
/// pipeline is supposed to surface); other release builds embed
/// `DWALL_LOG=debug` from `build.rs` and therefore never reach this default.
const fn default_log_level() -> LevelFilter {
    if cfg!(debug_assertions) {
        LevelFilter::Trace
    } else {
        LevelFilter::Info
    }
}

/// Get log level from environment variable
///
/// Accepts a plain level (`debug`) as well as the `env_logger`-style forms people
/// reach for out of habit (`dwall=debug`, `dwall=debug,tauri=info`).
fn get_log_level() -> LevelFilter {
    let from_env = env::var("DWALL_LOG").or_else(|_| env::var("RUST_LOG")).ok();

    from_env
        .as_deref()
        .or(option_env!("DWALL_LOG"))
        .and_then(parse_log_level)
        .unwrap_or_else(default_log_level)
}

/// Parses `debug`, `dwall=debug` or `dwall=debug,tauri=info` into a level.
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
}
