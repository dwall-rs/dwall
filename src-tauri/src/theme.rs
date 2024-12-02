use std::{os::windows::process::CommandExt, process::Command};

use windows::Win32::System::Threading::CREATE_NO_WINDOW;

use crate::{error::DwallSettingsResult, DAEMON_EXE_PATH};

pub fn spawn_apply_daemon() -> DwallSettingsResult<()> {
    let daemon_path = DAEMON_EXE_PATH.get().unwrap().to_str().unwrap();

    let handle = Command::new(daemon_path)
        .creation_flags(CREATE_NO_WINDOW.0)
        .spawn()?;
    info!(pid = handle.id(), "Spawned daemon using subprocess");

    Ok(())
}
