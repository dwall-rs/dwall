use std::{
    fs::File,
    io::{BufReader, Read},
    os::windows::process::CommandExt,
    process::Command,
};

use serde::Deserialize;
use serde_json::Value;
use windows::Win32::System::Threading::CREATE_NO_WINDOW;

use crate::{
    error::{DwallSettingsError, DwallSettingsResult},
    DAEMON_EXE_PATH, DWALL_CONFIG_DIR,
};

#[derive(Deserialize)]
struct DaemonErrorLog {
    fields: Value,
}

pub fn read_daemon_error_log() -> Option<String> {
    let log_file_path = DWALL_CONFIG_DIR.join("dwall.log");
    if !log_file_path.exists() {
        debug!("Daemon log file does not exist yet");
        return None;
    }

    let file = match File::open(&log_file_path) {
        Ok(file) => file,
        Err(e) => {
            warn!(error = ?e, "Failed to open daemon log file");
            return None;
        }
    };

    let mut content = String::new();
    let mut reader = BufReader::new(file);
    if reader.read_to_string(&mut content).is_err() {
        warn!("Failed to read daemon log file");
        return None;
    }

    // Split by lines and collect into a vector
    let lines: Vec<&str> = content.lines().collect();

    // Search backwards for the first line containing error message
    // Usually the last line is empty, so start from the second last line
    for line in lines.iter().rev() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        if line.to_lowercase().contains("error") {
            match serde_json::from_str::<DaemonErrorLog>(line) {
                Ok(log_line) => {
                    debug!("Found last error log entry");
                    return Some(log_line.fields.to_string());
                }
                Err(e) => {
                    warn!(error = ?e, "Failed to parse log line as JSON");
                }
            }
        }
    }

    debug!("No error logs found in daemon log file");
    None
}

pub fn spawn_apply_daemon() -> DwallSettingsResult<()> {
    let daemon_path = match DAEMON_EXE_PATH.get() {
        Some(path) => path.as_os_str(),
        None => {
            error!("DAEMON_EXE_PATH is not set");
            return Err(DwallSettingsError::Daemon(
                "DAEMON_EXE_PATH is not set".into(),
            ));
        }
    };

    let mut cmd = Command::new(daemon_path);

    if cfg!(debug_assertions) {
        use std::process::Stdio;

        cmd.creation_flags(CREATE_NO_WINDOW.0)
            .stderr(Stdio::inherit())
            .stdout(Stdio::inherit());
    } else {
        cmd.creation_flags(CREATE_NO_WINDOW.0);
    }

    match cmd.spawn() {
        Ok(handle) => {
            info!(pid = handle.id(), "Spawned daemon using subprocess");
            Ok(())
        }
        Err(e) => {
            error!(error = ?e, path = ?daemon_path, "Failed to spawn daemon");
            Err(e.into())
        }
    }
}
