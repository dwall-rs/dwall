//! Theme domain module
//!
//! This module contains the core business logic related to themes.

use std::io::Read;
use std::path::Path;

use dwall::SolarThemeValidator;
use serde::Deserialize;
use serde_json::Value;

#[derive(Deserialize)]
struct DaemonLogEntry {
    fields: Value,
}

/// Validates a theme against the solar theme specification
pub fn validate_solar_theme(
    themes_directory: &Path,
    theme_id: &str,
) -> Result<(), dwall::error::DwallError> {
    SolarThemeValidator::validate_solar_theme(themes_directory, theme_id)
}

/// Attempts to read the most recent error from the daemon log file
///
/// Returns the error message if found, or None if no error was found or the log file couldn't be read
pub fn get_last_daemon_error(dwall_config_dir: &std::path::Path) -> Option<String> {
    let log_file_path = dwall_config_dir.join("dwall.log");
    if !log_file_path.exists() {
        return None;
    }

    let file = match std::fs::File::open(&log_file_path) {
        Ok(file) => file,
        Err(_) => {
            return None;
        }
    };

    let mut content = Vec::new();
    let mut reader = std::io::BufReader::new(file);
    if reader.read_to_end(&mut content).is_err() {
        return None;
    }

    // Convert bytes to string
    let content = match String::from_utf8(content) {
        Ok(content) => content,
        Err(_) => return None,
    };

    // Get lines and search from newest to oldest
    let lines: Vec<&str> = content.lines().collect();

    for line in lines.iter().rev() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        if line.to_lowercase().contains("error") {
            match serde_json::from_str::<DaemonLogEntry>(line) {
                Ok(log_line) => {
                    return Some(log_line.fields.to_string());
                }
                Err(_) => {
                    // Just continue searching if this line couldn't be parsed
                    continue;
                }
            }
        }
    }

    None
}
