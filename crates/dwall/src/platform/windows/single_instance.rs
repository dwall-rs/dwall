//! Named-mutex single-instance guard for the daemon.
//!
//! The daemon owns the wallpaper, colour scheme and lock screen of the session it
//! runs in, so the rule is one daemon per session; a second session (an RDP login,
//! a fast user switch) legitimately has its own desktop and therefore its own
//! daemon, which is why the name is `Local\` and not `Global\`.
//!
//! A path is not a usable identity here: the installer's copy, a portable copy and
//! a hand-placed `dwall.exe` are all the same product, possibly in directories the
//! settings app has never heard of. A named mutex is enforced by the kernel and is
//! independent of where the process was launched from — two copies of the daemon
//! would otherwise write the same config-driven wallpaper and rotate the same log
//! file against each other.

use windows::Win32::Foundation::{CloseHandle, ERROR_ALREADY_EXISTS, GetLastError, HANDLE};
use windows::Win32::System::Threading::{CreateMutexW, OpenMutexW, SYNCHRONIZATION_SYNCHRONIZE};
use windows::core::PCWSTR;

use crate::DwallResult;
use crate::utils::WideStringExt;

/// Object name of the daemon's mutex, scoped to the current session.
const MUTEX_NAME: &str = "Local\\com.thep0y.dwall.daemon";

/// Holds the daemon mutex for as long as it is alive.
///
/// The mutex exists while any handle to it is open, so simply keeping the guard
/// alive is what marks the daemon as running; there is no ownership to release.
#[must_use = "dropping the guard lets a second daemon start"]
pub struct DaemonInstance(HANDLE);

impl Drop for DaemonInstance {
    fn drop(&mut self) {
        unsafe {
            let _ = CloseHandle(self.0);
        }
    }
}

/// Whether a daemon already runs in this session.
///
/// Cheap enough to poll: unlike enumerating the process list, this is a single
/// `OpenMutex` call.
pub fn daemon_is_running() -> bool {
    is_held(MUTEX_NAME)
}

/// Claims the daemon mutex, or reports that another daemon already holds it.
pub fn acquire_daemon_instance() -> DwallResult<Option<DaemonInstance>> {
    acquire_named(MUTEX_NAME)
}

/// Whether `name` currently exists as a mutex object.
fn is_held(name: &str) -> bool {
    let name = Vec::from_str(name);
    unsafe { OpenMutexW(SYNCHRONIZATION_SYNCHRONIZE, false, PCWSTR(name.as_ptr())) }
        .map(|handle| {
            unsafe {
                let _ = CloseHandle(handle);
            }
            true
        })
        .unwrap_or(false)
}

/// Claims the mutex called `name`.
fn acquire_named(name: &str) -> DwallResult<Option<DaemonInstance>> {
    let name = Vec::from_str(name);
    let handle = unsafe { CreateMutexW(None, false, PCWSTR(name.as_ptr())) }?;

    // `CreateMutexW` succeeds for an existing object too, so the outcome has to be
    // read from the thread's last error — and only immediately after the call.
    if unsafe { GetLastError() } == ERROR_ALREADY_EXISTS {
        unsafe {
            let _ = CloseHandle(handle);
        }
        return Ok(None);
    }

    Ok(Some(DaemonInstance(handle)))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A name unique to this process, so the test never collides with a daemon
    /// that happens to be running on the machine.
    fn test_name() -> String {
        format!("Local\\com.thep0y.dwall.daemon.test{}", std::process::id())
    }

    #[test]
    fn the_second_claim_loses_and_dropping_the_guard_releases_it() {
        let name = test_name();

        let first = acquire_named(&name).expect("first acquire failed");
        assert!(first.is_some(), "the first claim must win the mutex");
        assert!(is_held(&name));

        // Re-opening an existing mutex is the same situation a second daemon
        // process would see.
        assert!(
            acquire_named(&name)
                .expect("second acquire failed")
                .is_none(),
            "a second claim must not win the mutex"
        );

        drop(first);
        assert!(!is_held(&name), "dropping the guard must release the mutex");
    }
}
