//! Daemon process lifecycle: launching the daemon and reading its log.

use std::os::windows::process::CommandExt;
use std::path::Path;
use std::process::Command;

use windows::Win32::System::Threading::CREATE_NO_WINDOW;

use crate::DAEMON_EXE_PATH;
use crate::error::{DwallSettingsError, DwallSettingsResult};

/// The configured daemon executable path.
///
/// Returns an error instead of panicking: `dwall.exe` may be missing (partial
/// install, anti-virus quarantine, portable copy), and the caller is expected to
/// surface that to the user rather than aborting the settings process.
pub(crate) fn daemon_path() -> DwallSettingsResult<&'static Path> {
    DAEMON_EXE_PATH
        .get()
        .map(std::path::PathBuf::as_path)
        .ok_or_else(|| DwallSettingsError::Daemon("Daemon executable path not configured".into()))
}

/// Whether the configuration asks for a running daemon.
///
/// Random mode always needs one (the daemon draws the daily theme); fixed mode only
/// needs one when at least one monitor is configured. `setup_app` and
/// `ThemeApplier::apply` must agree on this, otherwise "stop engine" does not stick
/// and saving a random pool leaves the engine dead.
pub(crate) fn should_run(config: &dwall::Config) -> bool {
    match config.wallpaper_mode() {
        dwall::config::WallpaperMode::Random { .. } => true,
        dwall::config::WallpaperMode::Fixed {
            monitor_specific_wallpapers,
        } => !monitor_specific_wallpapers.is_empty(),
    }
}

/// Whether a daemon is running.
///
/// Authoritative and cheap: the daemon holds a named mutex for its whole lifetime,
/// so this is a single `OpenMutex` call instead of a process snapshot, and it is
/// true for a daemon installed anywhere (the installer's copy next to this
/// executable, a portable one, or a headless one in `C:\tools`). Use
/// [`crate::process::find_daemon_process`] when the PID is needed as well.
pub fn is_daemon_running() -> DwallSettingsResult<bool> {
    Ok(dwall::daemon_is_running())
}

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

/// Bytes of the log read when looking for the newest error.
///
/// The daemon logs one `ERROR` line per failed cycle and backs off to at most one
/// attempt every five minutes, so this window covers far more than a week of
/// failures — and it keeps the settings process from allocating a log that grows
/// for as long as the daemon runs.
const LOG_TAIL_BYTES: u64 = 64 * 1024;

/// Attempts to read the most recent error from the daemon log file.
///
/// `log_dir` is the daemon's log directory (`dwall::DWALL_LOG_DIR`); the daemon
/// writes plain-text lines (`"{timestamp} {LEVEL} [target] {message}"`), so the
/// last `ERROR` line in the tail of the file is returned as-is.
pub fn get_last_daemon_error(log_dir: &Path) -> Option<String> {
    let log_file_path = log_dir.join("dwall.log");
    if !log_file_path.is_file() {
        return None;
    }

    let content = read_log_tail(&log_file_path)?;
    let content = String::from_utf8_lossy(&content);

    content
        .lines()
        .rev()
        .map(str::trim)
        .find(|line| !line.is_empty() && line.contains(" ERROR "))
        .map(str::to_owned)
}

/// The last [`LOG_TAIL_BYTES`] of `path`, or `None` when it cannot be read.
///
/// A tail that starts mid-line loses that line, which only matters if the very line
/// the caller is looking for happens to be the one cut in half; the first (partial)
/// line is dropped for that reason.
fn read_log_tail(path: &Path) -> Option<Vec<u8>> {
    use std::io::{Read, Seek, SeekFrom};

    let mut file = std::fs::File::open(path).ok()?;
    let length = file.metadata().ok()?.len();

    if length <= LOG_TAIL_BYTES {
        let mut content = Vec::with_capacity(length as usize);
        file.read_to_end(&mut content).ok()?;

        return Some(content);
    }

    file.seek(SeekFrom::End(-(LOG_TAIL_BYTES as i64))).ok()?;

    let mut content = Vec::with_capacity(LOG_TAIL_BYTES as usize);
    file.read_to_end(&mut content).ok()?;

    // A truncated record is never reported as the daemon's last error.
    match content.iter().position(|byte| *byte == b'\n') {
        Some(newline) => Some(content.split_off(newline + 1)),
        None => Some(content),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn scratch(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("dwall_daemon_log_{name}"));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();

        dir
    }

    #[test]
    fn a_missing_log_has_no_error() {
        let dir = scratch("missing");

        assert_eq!(get_last_daemon_error(&dir), None);

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_log_without_errors_has_no_error() {
        let dir = scratch("clean");
        std::fs::write(
            dir.join("dwall.log"),
            "2026-10-17T10:00:00+08:00 INFO [dwall] Starting daemon\n\
             2026-10-17T10:00:01+08:00 DEBUG [dwall] Theme cycle completed\n",
        )
        .unwrap();

        assert_eq!(get_last_daemon_error(&dir), None);

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn the_newest_error_line_is_returned() {
        let dir = scratch("error");
        std::fs::write(
            dir.join("dwall.log"),
            "2026-10-17T10:00:00+08:00 INFO [dwall] Starting daemon\n\
             2026-10-17T10:00:05+08:00 ERROR [dwall] Theme update cycle failed, retrying\n\
             2026-10-17T10:00:20+08:00 ERROR [dwall] Every configured monitor failed\n",
        )
        .unwrap();

        let error = get_last_daemon_error(&dir).expect("an error line");

        assert!(error.contains("Every configured monitor failed"), "{error}");

        let _ = std::fs::remove_dir_all(&dir);
    }

    /// The tail is what is read, so an error older than the window is not reported
    /// as the daemon's *last* error.
    #[test]
    fn only_the_tail_of_a_long_log_is_searched() {
        let dir = scratch("long");
        let path = dir.join("dwall.log");
        let filler = "2026-10-17T10:00:00+08:00 INFO [dwall] a line\n".repeat(8192);

        std::fs::write(
            &path,
            format!("2026-10-17T09:00:00+08:00 ERROR [dwall] ancient failure\n{filler}"),
        )
        .unwrap();

        assert!(path.metadata().unwrap().len() > LOG_TAIL_BYTES);
        assert_eq!(get_last_daemon_error(&dir), None);

        // The same log with the error inside the window is found.
        std::fs::write(
            &path,
            format!(
                "2026-10-17T09:00:00+08:00 ERROR [dwall] ancient failure\n{filler}\
                 2026-10-17T11:00:00+08:00 ERROR [dwall] recent failure\n"
            ),
        )
        .unwrap();

        assert!(
            get_last_daemon_error(&dir)
                .expect("an error line")
                .contains("recent failure")
        );

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn the_tail_starts_on_a_line_boundary() {
        let dir = scratch("boundary");
        let path = dir.join("dwall.log");

        // A line long enough that the window lands in the middle of it.
        let long_line = format!("{} tail-marker", "y".repeat(LOG_TAIL_BYTES as usize + 1));
        std::fs::write(&path, format!("first line\n{long_line}\nlast line\n")).unwrap();
        assert!(path.metadata().unwrap().len() > LOG_TAIL_BYTES);

        let tail = read_log_tail(&path).unwrap();
        let tail = String::from_utf8_lossy(&tail);

        assert!(
            !tail.starts_with('y'),
            "the window must not start in the middle of a line"
        );
        assert!(tail.contains("last line"));
        assert!(!tail.contains("first line"));

        let _ = std::fs::remove_dir_all(&dir);
    }

    /// The line the window cuts in half is dropped, so a truncated record is never
    /// reported as the daemon's last error.
    #[test]
    fn a_truncated_line_is_not_reported_as_an_error() {
        let dir = scratch("truncated");
        let path = dir.join("dwall.log");

        let long_line = format!(
            "{} ERROR [dwall] a line that is cut in half",
            "y".repeat(LOG_TAIL_BYTES as usize + 1)
        );
        std::fs::write(
            &path,
            format!("first line\n{long_line}\n2026-10-17T10:00:00+08:00 INFO [dwall] ok\n"),
        )
        .unwrap();

        assert_eq!(get_last_daemon_error(&dir), None);

        let _ = std::fs::remove_dir_all(&dir);
    }
}
