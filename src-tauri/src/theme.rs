use std::{
    fs::File,
    io::{BufReader, Read},
    process::Command,
};

use serde::Deserialize;
use serde_json::Value;

use crate::{error::DwallSettingsResult, DAEMON_EXE_PATH, DWALL_CONFIG_DIR};

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

    match File::open(&log_file_path) {
        Ok(file) => {
            // Read all file content into a string
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
                            // return Some(line.to_string());
                        }
                    }
                }
            }

            debug!("No error logs found in daemon log file");
            None
        }
        Err(e) => {
            warn!(error = ?e, "Failed to open daemon log file");
            None
        }
    }
}

pub fn spawn_apply_daemon() -> DwallSettingsResult<()> {
    let daemon_path = DAEMON_EXE_PATH.get().unwrap().to_str().unwrap();

    #[cfg(debug_assertions)]
    {
        use std::io::BufRead;
        use std::process::Stdio;

        info!("Debug mode: spawning daemon with visible output");
        let stdout = Command::new(daemon_path)
            .stdout(Stdio::piped())
            .spawn()
            .map_err(|e| {
                error!(error = ?e, path = %daemon_path, "Failed to spawn daemon");
                e
            })?
            .stdout
            .expect("Could not capture standard output.");

        let reader = BufReader::new(stdout);

        for line in reader.lines() {
            let line = line?;
            println!("{}", line);
        }

        Ok(())
    }

    #[cfg(not(debug_assertions))]
    {
        use std::os::windows::process::CommandExt;
        use windows::Win32::System::Threading::CREATE_NO_WINDOW;

        match Command::new(daemon_path)
            .creation_flags(CREATE_NO_WINDOW.0)
            .spawn()
        {
            Ok(handle) => {
                info!(pid = handle.id(), "Spawned daemon using subprocess");
                Ok(())
            }
            Err(e) => {
                error!(error = ?e, path = %daemon_path, "Failed to spawn daemon");
                Err(e.into())
            }
        }
    }
}
