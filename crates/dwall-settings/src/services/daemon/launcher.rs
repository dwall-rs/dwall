//! Daemon launcher service (use case orchestration)

use std::{os::windows::process::CommandExt, process::Command};

use windows::Win32::System::Threading::CREATE_NO_WINDOW;

use crate::{
    DAEMON_EXE_PATH,
    error::{DwallSettingsError, DwallSettingsResult},
};

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
