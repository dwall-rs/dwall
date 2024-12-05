use std::{os::windows::process::CommandExt, process::Command};

use windows::Win32::System::Threading::CREATE_NO_WINDOW;

use crate::{error::DwallSettingsResult, DAEMON_EXE_PATH};

pub fn spawn_apply_daemon() -> DwallSettingsResult<()> {
    let daemon_path = DAEMON_EXE_PATH.get().unwrap().to_str().unwrap();
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
