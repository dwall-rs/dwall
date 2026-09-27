//! Daemon process lifecycle: launching the daemon and reading its log.

use std::io::Read;
use std::os::windows::process::CommandExt;
use std::process::Command;

use serde::Deserialize;
use serde_json::Value;
use windows::Win32::System::Threading::CREATE_NO_WINDOW;

use crate::DAEMON_EXE_PATH;
use crate::error::{DwallSettingsError, DwallSettingsResult};

/// Launches the daemon process
pub struct DaemonLauncher;

impl DaemonLauncher {
    /// Launch the daemon executable
    pub fn launch() -> DwallSettingsResult<()> {
        let daemon_path = match DAEMON_EXE_PATH.get() {
            Some(path) => path.as_os_str(),
            None => {
                return Err(DwallSettingsError::Daemon(
                    "Daemon executable path not configured".into(),
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

        cmd.spawn()?;
        Ok(())
    }
}

#[derive(Deserialize)]
struct DaemonLogEntry {
    fields: Value,
}

/// Attempts to read the most recent error from the daemon log file.
///
/// Returns the error message if found, or `None` if no error was found or the
/// log file couldn't be read.
pub fn get_last_daemon_error(dwall_config_dir: &std::path::Path) -> Option<String> {
    let log_file_path = dwall_config_dir.join("dwall.log");
    if !log_file_path.exists() {
        return None;
    }

    let file = std::fs::File::open(&log_file_path).ok()?;

    let mut content = Vec::new();
    let mut reader = std::io::BufReader::new(file);
    reader.read_to_end(&mut content).ok()?;

    let content = String::from_utf8(content).ok()?;

    for line in content.lines().rev() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        if line.to_lowercase().contains("error")
            && let Ok(log_line) = serde_json::from_str::<DaemonLogEntry>(line)
        {
            return Some(log_line.fields.to_string());
        }
    }

    None
}
