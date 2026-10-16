//! Per-user auto-start registration (`HKCU\Software\Microsoft\Windows\CurrentVersion\Run`).
//!
//! One implementation for both binaries: the daemon exposes it as
//! `--install-autostart` / `--uninstall-autostart`, and the settings app's toggle
//! calls the same functions, so the command-line quoting and the "is it
//! registered?" comparison cannot drift apart.

use std::path::Path;

use windows::Win32::{
    Foundation::ERROR_FILE_NOT_FOUND,
    System::Registry::{KEY_QUERY_VALUE, KEY_WRITE, REG_SZ},
};

use crate::error::{DwallError, DwallResult};
use crate::platform::RegistryKey;
use crate::utils::{WideStringExt, WideStringRead};

/// Registry value name under the `Run` key.
const APP_NAME: &str = "Dwall";

/// `Run` key under `HKEY_CURRENT_USER`.
const KEY_PATH: &str = "Software\\Microsoft\\Windows\\CurrentVersion\\Run";

/// Command line stored for `executable`.
///
/// The install path contains spaces ("Dwall Settings", "Dwall"), so the value has
/// to be quoted; unquoted, Windows parses `C:\Program Files\Dwall\dwall.exe` as
/// `C:\Program` plus arguments, which is the classic unquoted-path hijack.
fn command_line(executable: &Path) -> String {
    format!("\"{}\"", executable.display())
}

/// Whether a stored command line refers to `executable`.
///
/// Accepts the quoted form written by current versions and the bare path written
/// by older ones.
fn command_matches(command: &str, executable: &Path) -> bool {
    let command = command.trim();
    let expected = executable.to_string_lossy();

    command == expected
        || command == command_line(executable)
        || command == expected.trim_matches('"')
}

/// The command line currently registered, if any.
pub fn registered_autostart_command() -> DwallResult<Option<String>> {
    let registry_key = RegistryKey::open(KEY_PATH, KEY_QUERY_VALUE)?;

    let mut data_size = 0;
    match registry_key.query(APP_NAME, None, None, Some(&mut data_size)) {
        Ok(()) => {}
        Err(crate::RegistryError::Query(error)) if error == ERROR_FILE_NOT_FOUND => {
            return Ok(None);
        }
        Err(error) => return Err(error.into()),
    }

    let mut data: Vec<u16> = vec![0; data_size as usize];
    let mut data_type = REG_SZ;
    match registry_key.query(
        APP_NAME,
        Some(std::ptr::addr_of_mut!(data_type)),
        Some(data.as_mut_ptr() as *mut u8),
        Some(&mut data_size),
    ) {
        Ok(()) => Ok(Some(data.to_string())),
        Err(crate::RegistryError::Query(error)) if error == ERROR_FILE_NOT_FOUND => Ok(None),
        Err(error) => Err(error.into()),
    }
}

/// Whether auto-start is registered for `executable`.
pub fn is_autostart_installed(executable: &Path) -> DwallResult<bool> {
    let Some(command) = registered_autostart_command()? else {
        return Ok(false);
    };

    Ok(command_matches(&command, executable))
}

/// Registers `executable` to start at sign-in.
pub fn install_autostart(executable: &Path) -> DwallResult<()> {
    let registry_key = RegistryKey::open(KEY_PATH, KEY_WRITE)?;

    let wide_command = Vec::from_str(&command_line(executable));
    let bytes = unsafe {
        std::slice::from_raw_parts(
            wide_command.as_ptr() as *const u8,
            wide_command.len() * std::mem::size_of::<u16>(),
        )
    };
    registry_key.set(APP_NAME, REG_SZ, bytes)?;

    Ok(())
}

/// Removes the auto-start registration.
///
/// Succeeds when nothing was registered, so it can be used as a cleanup step.
pub fn uninstall_autostart() -> DwallResult<()> {
    let registry_key = RegistryKey::open(KEY_PATH, KEY_WRITE)?;

    match registry_key.delete(APP_NAME) {
        Ok(()) => Ok(()),
        Err(crate::RegistryError::Delete(error)) if error == ERROR_FILE_NOT_FOUND => Ok(()),
        Err(error) => Err(DwallError::from(error)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    // Deliberately no test that touches `HKCU\...\Run`: the value name is shared
    // with the installed copy and with the settings app, so a test run could
    // overwrite (or delete) the user's real auto-start entry. The parsing and
    // comparison rules above are the parts worth pinning down; the registry calls
    // themselves are exercised by hand with `dwall.exe --install-autostart`.

    #[test]
    fn the_stored_command_line_is_quoted() {
        let executable = PathBuf::from(r"C:\Program Files\Dwall\dwall.exe");

        assert_eq!(
            command_line(&executable),
            r#""C:\Program Files\Dwall\dwall.exe""#
        );
    }

    #[test]
    fn matching_accepts_quoted_and_legacy_bare_values() {
        let executable = PathBuf::from(r"C:\Dwall\dwall.exe");

        assert!(command_matches(r#""C:\Dwall\dwall.exe""#, &executable));
        assert!(command_matches(r"C:\Dwall\dwall.exe", &executable));
        assert!(command_matches(r#"  "C:\Dwall\dwall.exe"  "#, &executable));
    }

    #[test]
    fn matching_rejects_another_executable() {
        let executable = PathBuf::from(r"C:\Dwall\dwall.exe");

        assert!(!command_matches(r#""C:\Other\dwall.exe""#, &executable));
        assert!(!command_matches("", &executable));
    }
}
