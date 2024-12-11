use std::str::FromStr;

use time::macros::format_description;
use tracing::Level;
use tracing_subscriber::{fmt::time::LocalTime, EnvFilter};

fn get_log_level() -> Level {
    std::env::var("DWALL_LOG")
        .map(|level| Level::from_str(&level).unwrap_or(default_log_level()))
        .unwrap_or_else(|_| default_log_level())
}

fn default_log_level() -> Level {
    if cfg!(debug_assertions) {
        Level::TRACE
    } else {
        Level::WARN
    }
}

pub fn setup_logging(pkg_name: &str) {
    let fmt = if cfg!(debug_assertions) {
        format_description!("[hour]:[minute]:[second].[subsecond digits:3]")
    } else {
        format_description!("[year]-[month]-[day] [hour]:[minute]:[second].[subsecond digits:3]")
    };

    let timer = LocalTime::new(fmt);

    #[cfg(not(debug_assertions))]
    let writer = {
        use crate::lazy::APP_CONFIG_DIR;
        use std::{fs::File, sync::Mutex};

        let log_file = File::create(APP_CONFIG_DIR.join(format!("{pkg_name}.log")))
            .expect("Failed to create the log file");
        Mutex::new(log_file)
    };

    #[cfg(debug_assertions)]
    let writer = std::io::stderr;

    let level = get_log_level();

    let builder = tracing_subscriber::fmt()
        .with_line_number(true)
        .with_env_filter(EnvFilter::new(format!("{pkg_name}={level}")))
        .with_timer(timer)
        .with_writer(writer);

    if cfg!(debug_assertions) {
        builder
            .with_file(true)
            .with_target(false)
            .with_ansi(true)
            .init();
    } else {
        builder.with_file(false).with_target(true).json().init();
    }
}
