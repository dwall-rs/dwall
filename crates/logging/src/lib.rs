use std::{env, io::Write, str::FromStr};

use log::{LevelFilter, Log, SetLoggerError, set_boxed_logger, set_max_level};
use time::{OffsetDateTime, UtcDateTime};

mod macros;
#[cfg(debug_assertions)]
mod rich;

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

        let file = OpenOptions::new().create(true).write(true).open(&path)?;

        self.output = ProductionOutput::File(std::sync::Mutex::new(file));
        Ok(self)
    }

    pub fn init(self) -> std::result::Result<(), SetLoggerError> {
        set_max_level(self.level);
        set_boxed_logger(Box::new(self))
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

            let file = record.file().unwrap_or("<unknown>");
            let line = record.line().unwrap_or(0);

            //   2026-06-01T01:27:42.340486Z DEBUG  Reading configuration file, path: C:\Users\wzl03\AppData\Roaming\dwall\config.toml
            //     at src\config.rs:79

            let msg = record.args().to_string();
            let msg = restore_color(&msg, level_color);
            eprintln!(
                "  {DIM}{timestamp}{RESET} \
                 {BOLD}{level_color}{level_str}{RESET} {level_color}{}{RESET}\n    \
                 {DIM}{ITALIC}at{RESET} {file}:{line}\n",
                msg,
            );
        }

        #[cfg(not(debug_assertions))]
        {
            use std::io::Write;

            let level_str = match record.level() {
                log::Level::Error => "ERROR",
                log::Level::Warn => "WARN",
                log::Level::Info => "INFO",
                log::Level::Debug => "DEBUG",
                log::Level::Trace => "TRACE",
            };

            let line = format!(
                "{} {} [{}] {}\n",
                timestamp,
                level_str,
                record.target(),
                record.args(),
            );

            match &self.output {
                ProductionOutput::Stderr => {
                    eprint!("{line}");
                }
                ProductionOutput::File(mutex) => {
                    if let Ok(mut file) = mutex.lock() {
                        let _ = file.write_all(line.as_bytes());
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
const fn default_log_level() -> LevelFilter {
    if cfg!(debug_assertions) {
        LevelFilter::Trace
    } else {
        LevelFilter::Warn
    }
}

/// Get log level from environment variable
fn get_log_level() -> LevelFilter {
    let from_env = env::var("DWALL_LOG").or_else(|_| env::var("RUST_LOG")).ok();

    from_env
        .as_deref()
        .or(option_env!("DWALL_LOG"))
        .and_then(|s| LevelFilter::from_str(s).ok())
        .unwrap_or_else(default_log_level)
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
