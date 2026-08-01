//! Daemon log parsing utilities

use std::io::Read;

use serde::Deserialize;
use serde_json::Value;

#[derive(Deserialize)]
struct DaemonLogEntry {
    fields: Value,
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
        Err(_) => return None,
    };

    let mut content = Vec::new();
    let mut reader = std::io::BufReader::new(file);
    if reader.read_to_end(&mut content).is_err() {
        return None;
    }

    let content = match String::from_utf8(content) {
        Ok(content) => content,
        Err(_) => return None,
    };

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
                    continue;
                }
            }
        }
    }

    None
}
