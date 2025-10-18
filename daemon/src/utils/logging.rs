//! Logging utilities - moved from log.rs

use std::{env, str::FromStr};

use tracing::Level;
use tracing_subscriber::EnvFilter;

#[cfg(debug_assertions)]
struct MyTimer;

#[cfg(debug_assertions)]
impl tracing_subscriber::fmt::time::FormatTime for MyTimer {
    fn format_time(&self, w: &mut tracing_subscriber::fmt::format::Writer<'_>) -> std::fmt::Result {
        write!(
            w,
            "{}",
            crate::utils::datetime::UtcDateTime::now().to_rfc3339()
        )
    }
}

/// Get default log level
fn default_log_level() -> Level {
    if cfg!(debug_assertions) {
        Level::TRACE
    } else {
        Level::WARN
    }
}

/// Get log level from environment variable
fn get_log_level() -> Level {
    env::var("DWALL_LOG")
        .ok()
        .as_deref()
        .or(option_env!("DWALL_LOG"))
        .and_then(|level| Level::from_str(level).ok())
        .unwrap_or_else(default_log_level)
}

/// Create environment filter for logging
fn create_env_filter<S: AsRef<str>>(pkg_names: &[S], level: Level) -> EnvFilter {
    let filter_str = pkg_names
        .iter()
        .map(|s| format!("{}={}", s.as_ref(), level))
        .collect::<Vec<String>>()
        .join(",");

    EnvFilter::builder()
        .with_default_directive(level.into())
        .parse_lossy(filter_str)
}

/// Setup logging with given configuration
pub fn setup_logging<S: AsRef<str>>(pkg_names: &[S]) {
    let level = get_log_level();

    #[cfg(debug_assertions)]
    let writer = std::io::stderr;

    #[cfg(not(debug_assertions))]
    let writer = {
        use crate::lazy::DWALL_LOG_DIR;
        use std::fs::File;

        let log_file = File::create(DWALL_LOG_DIR.join(format!("{}.log", pkg_names[0].as_ref())))
            .expect("Failed to create the log file");
        log_file
    };

    let builder = tracing_subscriber::fmt()
        .with_file(cfg!(debug_assertions))
        .with_target(!cfg!(debug_assertions))
        .with_line_number(cfg!(debug_assertions))
        .with_env_filter(create_env_filter(pkg_names, level))
        .with_writer(writer);

    #[cfg(debug_assertions)]
    builder.with_timer(MyTimer).with_ansi(true).init();
    #[cfg(not(debug_assertions))]
    builder.without_time().init();
}
