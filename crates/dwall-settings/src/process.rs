//! Windows process management: find, terminate and wait for the daemon.

use std::path::Path;
use std::time::Duration;

use windows::Win32::Foundation::{HANDLE, MAX_PATH, WAIT_OBJECT_0, WAIT_TIMEOUT};
use windows::Win32::System::Diagnostics::ToolHelp::{
    CreateToolhelp32Snapshot, PROCESSENTRY32, Process32First, Process32Next, TH32CS_SNAPPROCESS,
};
use windows::Win32::System::RemoteDesktop::ProcessIdToSessionId;
use windows::Win32::System::Threading::{
    OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION, PROCESS_SYNCHRONIZE, PROCESS_TERMINATE,
    QueryFullProcessImageNameW, TerminateProcess, WaitForSingleObject,
};
use windows::core::{Free, PWSTR};

use crate::error::{DwallSettingsError, DwallSettingsResult};

/// Executable name of the daemon, as built in `crates/dwall`.
const DAEMON_EXE_NAME: &str = "dwall.exe";

/// How long [`stop_daemon`] waits for the daemon to really exit.
///
/// `TerminateProcess` is not a request the daemon can refuse, so this only has to
/// cover the time the kernel needs to tear the process down; the ceiling exists so
/// that a wedged process cannot hang the window.
pub const DAEMON_EXIT_TIMEOUT: Duration = Duration::from_secs(5);

/// How long to wait for a freshly launched daemon to claim its mutex.
pub const START_TIMEOUT: Duration = Duration::from_secs(5);

/// How often to check while waiting for a freshly launched daemon.
pub const START_POLL_INTERVAL: Duration = Duration::from_millis(50);

/// Safe wrapper for a Windows HANDLE
#[derive(Debug)]
struct HandleWrapper(HANDLE);

impl HandleWrapper {
    fn new(handle: HANDLE) -> Self {
        Self(handle)
    }

    fn as_raw(&self) -> HANDLE {
        self.0
    }
}

impl Drop for HandleWrapper {
    fn drop(&mut self) {
        if !self.0.is_invalid() {
            unsafe { self.0.free() };
        }
    }
}

fn get_process_exe_name(process_entry: &PROCESSENTRY32) -> String {
    unsafe {
        let name_ptr = process_entry.szExeFile.as_ptr();
        let mut len = 0;
        while len < process_entry.szExeFile.len() && *name_ptr.add(len) != 0 {
            len += 1;
        }
        String::from_utf8_lossy(std::slice::from_raw_parts(name_ptr as *const u8, len)).to_string()
    }
}

/// Whether a process is a daemon this session may stop.
///
/// The daemon's mutex is session-local (`Local\`), so the daemon that belongs to
/// this window is the one in this session. Matching the executable name alone —
/// which is what this did — stopped an RDP or fast-user-switch session's daemon
/// while [`crate::daemon::is_daemon_running`] reported "not running" for this one.
fn is_our_daemon(image_name: &str, process_session: u32, our_session: u32) -> bool {
    image_name.eq_ignore_ascii_case(DAEMON_EXE_NAME) && process_session == our_session
}

/// The session this process runs in, or `None` when Windows will not say.
fn current_session_id() -> Option<u32> {
    session_id_of(std::process::id())
}

/// The session a process belongs to.
///
/// `None` covers both a process that has already exited (`ERROR_INVALID_PARAMETER`)
/// and one Windows refuses to describe; either way the caller must not act on it.
fn session_id_of(pid: u32) -> Option<u32> {
    let mut session = 0u32;

    unsafe { ProcessIdToSessionId(pid, &mut session) }.ok()?;

    Some(session)
}

/// Whether a process snapshot entry describes this session's daemon.
///
/// Fails closed: a candidate whose session Windows will not report — a process that
/// has exited, or one it refuses to describe — is not matched on the strength of its
/// name alone. Matching it used to hand [`terminate_process`] a PID that could have
/// been recycled by then.
fn is_daemon_entry(
    own_pid: u32,
    pid: u32,
    image_name: &str,
    our_session: Option<u32>,
    process_session: impl FnOnce() -> Option<u32>,
) -> bool {
    if pid == own_pid || !image_name.eq_ignore_ascii_case(DAEMON_EXE_NAME) {
        return false;
    }

    match (our_session, process_session()) {
        (Some(ours), Some(theirs)) => is_our_daemon(image_name, theirs, ours),
        // This process's own session could not be read, so the session filter cannot
        // be applied at all; the warning in `find_daemon_process` says as much.
        (None, _) => true,
        (Some(_), None) => false,
    }
}

/// Finds the running daemon process of this session, wherever it was installed.
///
/// Matching on the executable name — rather than on the path the settings app
/// expects next to itself — is what makes a portable or headless `dwall.exe`
/// visible: the installer's copy, one in `C:\tools` and a debug build are all the
/// same product, and the window has to be able to see and stop whichever one
/// actually runs. For a plain "is it running?" check prefer
/// [`crate::daemon::is_daemon_running`], which reads the daemon's mutex instead of
/// walking the process list.
pub fn find_daemon_process() -> DwallSettingsResult<Option<u32>> {
    let our_session = current_session_id();
    if our_session.is_none() {
        // Better to find the daemon without the session filter than not at all; the
        // log line is what makes the difference discoverable.
        warn!("Could not determine this session, matching the daemon by name only");
    }

    let snapshot = unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) }
        .map_err(DwallSettingsError::Windows)?;

    let snapshot = HandleWrapper::new(snapshot);
    let own_pid = std::process::id();

    let mut process_entry = PROCESSENTRY32 {
        dwSize: std::mem::size_of::<PROCESSENTRY32>() as u32,
        ..Default::default()
    };

    unsafe {
        if Process32First(snapshot.as_raw(), &mut process_entry).is_ok() {
            loop {
                let pid = process_entry.th32ProcessID;
                let image_name = get_process_exe_name(&process_entry);

                if is_daemon_entry(own_pid, pid, &image_name, our_session, || {
                    session_id_of(pid)
                }) {
                    return Ok(Some(pid));
                }

                if Process32Next(snapshot.as_raw(), &mut process_entry).is_err() {
                    break;
                }
            }
        }
    }

    Ok(None)
}

/// Terminates the daemon at `pid`, returning whether it was still the daemon.
///
/// The PID comes from a process snapshot, and a PID can be reused before the
/// handle is opened: the image name is therefore checked again on the *handle*
/// that is about to be terminated, so a recycled PID cannot take an unrelated
/// process down with it. `Ok(false)` means the process at that PID is no longer
/// the daemon — nothing left to stop rather than a failure.
pub fn terminate_process(pid: u32) -> DwallSettingsResult<bool> {
    let process_handle = unsafe {
        OpenProcess(
            PROCESS_TERMINATE | PROCESS_QUERY_LIMITED_INFORMATION,
            false,
            pid,
        )
    }
    .map_err(DwallSettingsError::Windows)?;

    let process_handle = HandleWrapper::new(process_handle);

    if !image_name_matches(&process_handle, DAEMON_EXE_NAME) {
        warn!(
            pid = pid,
            "The process at that PID is no longer the daemon, leaving it alone"
        );
        return Ok(false);
    }

    unsafe {
        TerminateProcess(process_handle.as_raw(), 0).map_err(DwallSettingsError::Windows)?;
    }

    Ok(true)
}

/// Stops the daemon of this session, returning the PID it stopped.
///
/// One place for the whole sequence, because every caller needs all of it: find the
/// daemon, verify it on the handle before terminating it, and wait for it to really
/// exit. `TerminateProcess` only asks — it returns while the process still holds the
/// mutex the next start has to claim — so launching a replacement right after it was
/// an intermittent, and misleadingly reported, "Daemon process failed to start".
pub fn stop_daemon() -> DwallSettingsResult<Option<u32>> {
    let Some(pid) = find_daemon_process()? else {
        return Ok(None);
    };

    info!(pid = pid, "Stopping the daemon process");

    if !terminate_process(pid)? {
        return Ok(None);
    }

    if !wait_for_exit(pid, DAEMON_EXIT_TIMEOUT)? {
        warn!(
            pid = pid,
            timeout_seconds = DAEMON_EXIT_TIMEOUT.as_secs(),
            "The daemon is still running after being terminated"
        );
    }

    Ok(Some(pid))
}

/// Waits for a process to really exit, up to `timeout`.
///
/// `false` means the wait ran out; a PID that cannot be opened at all counts as
/// exited, which is also what a PID that has already been reaped looks like.
pub fn wait_for_exit(pid: u32, timeout: Duration) -> DwallSettingsResult<bool> {
    let handle = unsafe { OpenProcess(PROCESS_SYNCHRONIZE, false, pid) };

    let handle = match handle {
        Ok(handle) => HandleWrapper::new(handle),
        Err(e) => {
            // Cannot be opened: either the PID is gone or Windows will not describe
            // it. Ask the only question the caller cares about instead of guessing.
            debug!(pid = pid, error = %e, "Could not open the process to wait for it");
            return Ok(find_daemon_process()?.is_none());
        }
    };

    let timeout_ms = timeout.as_millis().min(u128::from(u32::MAX)) as u32;

    match unsafe { WaitForSingleObject(handle.as_raw(), timeout_ms) } {
        WAIT_OBJECT_0 => Ok(true),
        WAIT_TIMEOUT => Ok(false),
        other => Err(DwallSettingsError::Other(format!(
            "Waiting for process {pid} failed: {other:?}"
        ))),
    }
}

/// Whether the process behind `handle` is running the expected executable.
fn image_name_matches(handle: &HandleWrapper, expected: &str) -> bool {
    let Some(path) = process_image_path(handle) else {
        // Windows would not describe the process, so it cannot be confirmed as the
        // daemon: the caller asked to stop the daemon, and this is not provably it.
        return false;
    };

    Path::new(&path)
        .file_name()
        .is_some_and(|name| name.to_string_lossy().eq_ignore_ascii_case(expected))
}

/// The full image path of the process behind `handle`.
fn process_image_path(handle: &HandleWrapper) -> Option<String> {
    let mut buffer = [0u16; MAX_PATH as usize];
    let mut length = buffer.len() as u32;

    unsafe {
        QueryFullProcessImageNameW(
            handle.as_raw(),
            Default::default(),
            PWSTR(buffer.as_mut_ptr()),
            &mut length,
        )
    }
    .ok()?;

    Some(String::from_utf16_lossy(&buffer[..length as usize]))
}

/// Polls `check` until it is true or `timeout` elapses, returning whether it became
/// true.
///
/// A fixed sleep used to stand in for this: the daemon claims its mutex before doing
/// any work, so waiting for that is a handshake, while `sleep(100ms)` was a guess
/// that failed whenever the machine was slow.
pub fn wait_until(timeout: Duration, poll_interval: Duration, check: impl Fn() -> bool) -> bool {
    let deadline = std::time::Instant::now() + timeout;

    loop {
        if check() {
            return true;
        }

        match deadline.checked_duration_since(std::time::Instant::now()) {
            Some(remaining) if !remaining.is_zero() => {
                std::thread::sleep(poll_interval.min(remaining));
            }
            _ => return false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_daemon_is_matched_by_name_and_session() {
        assert!(is_our_daemon("dwall.exe", 1, 1));
        assert!(is_our_daemon("DWALL.EXE", 1, 1));
    }

    /// The bug this closes: another session's daemon was stopped while this
    /// session's status said nothing was running.
    #[test]
    fn another_sessions_daemon_is_not_ours() {
        assert!(!is_our_daemon("dwall.exe", 2, 1));
    }

    #[test]
    fn another_executable_is_not_the_daemon() {
        assert!(!is_our_daemon("dwall-settings.exe", 1, 1));
        assert!(!is_our_daemon("notdwall.exe", 1, 1));
    }

    #[test]
    fn a_daemon_entry_is_matched_by_name_and_session() {
        assert!(is_daemon_entry(10, 20, "dwall.exe", Some(1), || Some(1)));
    }

    #[test]
    fn a_daemon_entry_in_another_session_is_not_matched() {
        assert!(!is_daemon_entry(10, 20, "dwall.exe", Some(1), || Some(2)));
    }

    /// The bug this closes: an undescribable process was treated as the daemon, and a
    /// PID reused between the snapshot and the terminate call was killed for it.
    #[test]
    fn a_daemon_entry_with_an_unknown_session_is_not_matched() {
        assert!(!is_daemon_entry(10, 20, "dwall.exe", Some(1), || None));
    }

    #[test]
    fn our_own_process_and_other_names_are_never_matched() {
        assert!(!is_daemon_entry(10, 10, "dwall.exe", Some(1), || Some(1)));
        assert!(!is_daemon_entry(
            10,
            20,
            "dwall-settings.exe",
            Some(1),
            || Some(1)
        ));
    }

    #[test]
    fn this_process_knows_its_own_session() {
        let session = current_session_id().expect("the current session id");

        assert_eq!(session_id_of(std::process::id()), Some(session));
    }

    #[test]
    fn a_pid_that_does_not_exist_has_no_session() {
        // PID 0 is the idle process, which `ProcessIdToSessionId` refuses.
        assert_eq!(session_id_of(0), None);
    }

    #[test]
    fn waiting_returns_immediately_when_the_check_passes() {
        let started = std::time::Instant::now();

        assert!(wait_until(
            Duration::from_secs(5),
            Duration::from_millis(10),
            || true
        ));
        assert!(started.elapsed() < Duration::from_secs(1));
    }

    #[test]
    fn waiting_polls_until_the_check_passes() {
        use std::cell::Cell;

        let attempts = Cell::new(0);
        let started = std::time::Instant::now();

        let ready = wait_until(Duration::from_secs(5), Duration::from_millis(10), || {
            attempts.set(attempts.get() + 1);

            attempts.get() >= 3
        });

        assert!(ready);
        assert_eq!(attempts.get(), 3);
        assert!(started.elapsed() < Duration::from_secs(5));
    }

    /// A daemon that never comes up has to be reported, not waited on forever.
    #[test]
    fn waiting_gives_up_after_the_timeout() {
        let started = std::time::Instant::now();

        assert!(!wait_until(
            Duration::from_millis(150),
            Duration::from_millis(20),
            || { false }
        ));
        assert!(started.elapsed() >= Duration::from_millis(150));
    }

    /// Our own process outlives any timeout, which makes it the deterministic
    /// "still running" case.
    #[test]
    fn waiting_for_our_own_process_times_out() {
        let exited = wait_for_exit(std::process::id(), Duration::from_millis(120)).unwrap();

        assert!(!exited, "this process is obviously still running");
    }

    /// A PID whose process is long gone counts as exited: that is the path taken
    /// when the daemon died between the snapshot and the wait.
    #[test]
    fn waiting_for_a_reaped_pid_reports_that_it_exited() {
        let mut child = std::process::Command::new("cmd")
            .args(["/c", "exit"])
            .spawn()
            .expect("a short-lived child process");

        let pid = child.id();
        child.wait().unwrap();

        assert!(wait_for_exit(pid, Duration::from_secs(5)).unwrap());
    }
}
