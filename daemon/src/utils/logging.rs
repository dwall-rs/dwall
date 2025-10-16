//! Logging utilities - moved from log.rs

use std::{env, str::FromStr, sync::Mutex};

use time::{format_description::BorrowedFormatItem, macros::format_description};
use tracing::Level;
use tracing_subscriber::{
    fmt::{time::LocalTime, writer::BoxMakeWriter},
    EnvFilter,
};

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

/// Get time format based on build configuration
fn get_time_format<'a>() -> &'a [BorrowedFormatItem<'a>] {
    if cfg!(debug_assertions) {
        format_description!("[hour]:[minute]:[second].[subsecond digits:3]")
    } else {
        format_description!("[year]-[month]-[day] [hour]:[minute]:[second].[subsecond digits:3]")
    }
}

/// Setup logging with given configuration
pub fn setup_logging<S: AsRef<str>>(pkg_names: &[S]) {
    let timer = LocalTime::new(get_time_format());
    let level = get_log_level();

    let writer = if cfg!(debug_assertions) {
        BoxMakeWriter::new(Mutex::new(std::io::stderr()))
    } else {
        use crate::lazy::DWALL_LOG_DIR;
        use std::fs::File;

        let log_file = File::create(DWALL_LOG_DIR.join(format!("{}.log", pkg_names[0].as_ref())))
            .expect("Failed to create the log file");
        BoxMakeWriter::new(Mutex::new(log_file))
    };

    let builder = tracing_subscriber::fmt()
        .with_file(cfg!(debug_assertions))
        .with_target(!cfg!(debug_assertions))
        .with_line_number(true)
        .with_env_filter(create_env_filter(pkg_names, level))
        .with_timer(timer)
        .with_writer(writer);

    if cfg!(debug_assertions) {
        builder.with_ansi(true).init();
    } else {
        builder.json().init();
    }
}
