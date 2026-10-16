//! Daemon process lifecycle: launching the daemon and reading its log.

use std::io::Read;
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

/// Attempts to read the most recent error from the daemon log file.
///
/// `log_dir` is the daemon's log directory (`dwall::DWALL_LOG_DIR`); the daemon
/// writes plain-text lines (`"{timestamp} {LEVEL} [target] {message}"`), so the
/// last `ERROR` line is returned as-is.
pub fn get_last_daemon_error(log_dir: &Path) -> Option<String> {
    let log_file_path = log_dir.join("dwall.log");
    if !log_file_path.exists() {
        return None;
    }

    let file = std::fs::File::open(&log_file_path).ok()?;

    let mut content = Vec::new();
    let mut reader = std::io::BufReader::new(file);
    reader.read_to_end(&mut content).ok()?;

    let content = String::from_utf8_lossy(&content);

    content
        .lines()
        .rev()
        .map(str::trim)
        .find(|line| !line.is_empty() && line.contains(" ERROR "))
        .map(str::to_owned)
}
