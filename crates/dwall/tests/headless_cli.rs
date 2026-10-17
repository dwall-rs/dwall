//! Integration tests for the daemon's command-line exit codes.
//!
//! `docs/headless.md` documents the exit codes as a public interface and the
//! parser lives inside the binary, so these tests run the real `dwall`
//! executable and assert only on its exit code and its output. Every invocation
//! that touches a configuration passes `--config` with a scratch file under the
//! temporary directory, so no test reads the user's configuration, applies a
//! wallpaper or leaves a daemon behind.

use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use windows::Win32::Foundation::{CloseHandle, HANDLE};
use windows::Win32::System::Threading::CreateMutexW;
use windows::core::PCWSTR;

/// The daemon binary cargo built for this test target.
const DWALL: &str = env!("CARGO_BIN_EXE_dwall");

/// Name of the per-session mutex a daemon holds for its whole lifetime, as
/// declared in `src/platform/windows/single_instance.rs`.
const DAEMON_MUTEX_NAME: &str = "Local\\com.thep0y.dwall.daemon";

// ── Helpers ─────────────────────────────────────────────────────────────────

/// Runs the daemon with `args` and captures its output.
fn run_dwall<I, S>(args: I) -> Output
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    Command::new(DWALL)
        .args(args)
        .output()
        .unwrap_or_else(|e| panic!("failed to run {DWALL}: {e}"))
}

/// Asserts that `output` exited with `expected`, showing both streams otherwise.
fn assert_exit(output: &Output, expected: i32) {
    assert_eq!(
        output.status.code(),
        Some(expected),
        "unexpected exit code\nstdout:\n{}\nstderr:\n{}",
        text(&output.stdout),
        text(&output.stderr),
    );
}

/// Decodes a captured stream.
fn text(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}

/// Creates an empty scratch directory unique to this test binary.
fn scratch_dir(name: &str) -> PathBuf {
    let dir =
        std::env::temp_dir().join(format!("dwall_headless_cli_{name}_{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("failed to create the scratch directory");

    dir
}

/// Renders `path` for a TOML basic string, where `\` is an escape character.
fn toml_path(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

/// A valid configuration whose theme paths all live inside `dir`.
///
/// It also asks the engine to do nothing visible: the safety net for the
/// `--once` test is the mutex, but even an unexpected cycle must not change the
/// session's wallpaper or colour scheme.
fn inert_config(dir: &Path) -> String {
    format!(
        "interval = 30\n\
         themes_directory = \"{themes}\"\n\
         customized_themes_directory = \"{customized}\"\n\
         auto_detect_color_scheme = false\n\
         lock_screen_wallpaper_enabled = false\n\
         \n\
         [position_source]\n\
         type = \"MANUAL\"\n\
         latitude = 0.0\n\
         longitude = 0.0\n\
         altitude = 0.0\n",
        themes = toml_path(&dir.join("themes")),
        customized = toml_path(&dir.join("customize")),
    )
}

/// Holds the daemon's per-session mutex for as long as the value lives.
#[must_use = "the mutex has to stay held while the child runs"]
struct HeldDaemonMutex(HANDLE);

impl Drop for HeldDaemonMutex {
    fn drop(&mut self) {
        // Safety: the handle came from `CreateMutexW` and is closed exactly once.
        unsafe {
            let _ = CloseHandle(self.0);
        }
    }
}

/// Creates the mutex a running daemon would hold and keeps it open.
///
/// The kernel destroys the object only when the last handle to it is closed, so
/// owning this handle is enough for `dwall --once` to see a daemon. If a real
/// daemon already owns the mutex, `CreateMutexW` returns a second handle to the
/// same object (`ERROR_ALREADY_EXISTS`), which is just as good here.
fn hold_daemon_mutex() -> HeldDaemonMutex {
    let name: Vec<u16> = DAEMON_MUTEX_NAME.encode_utf16().chain([0]).collect();

    // Safety: `name` is NUL-terminated and outlives the call.
    let handle = unsafe { CreateMutexW(None, false, PCWSTR(name.as_ptr())) }
        .expect("failed to create the daemon mutex");

    HeldDaemonMutex(handle)
}

// ── Tests ───────────────────────────────────────────────────────────────────

/// `--help` exits 0 and names every documented command-line flag.
#[test]
fn help_exits_zero_and_prints_the_usage() {
    let output = run_dwall(["--help"]);

    assert_exit(&output, 0);

    let stdout = text(&output.stdout);
    for flag in [
        "--config <PATH>",
        "--once",
        "--check-config",
        "--install-autostart",
        "--uninstall-autostart",
        "--help",
        "--version",
    ] {
        assert!(
            stdout.contains(flag),
            "usage does not name {flag}:\n{stdout}"
        );
    }

    assert!(
        stdout.contains("EXIT CODES"),
        "usage does not document the exit codes:\n{stdout}"
    );
}

/// An unusable command line is exit code 2, and the rejection explains itself.
#[test]
fn an_unknown_flag_exits_two() {
    let output = run_dwall(["--definitely-not-a-flag"]);

    assert_exit(&output, 2);

    let stderr = text(&output.stderr);
    assert!(
        stderr.contains("unknown argument: --definitely-not-a-flag"),
        "the error does not name the offending argument:\n{stderr}"
    );
    assert!(
        stderr.contains("USAGE:"),
        "the usage is not printed after the error:\n{stderr}"
    );
}

/// A configuration that cannot be parsed is exit code 3, and says which file.
#[test]
fn a_broken_config_exits_three() {
    let dir = scratch_dir("broken_config");
    let config_path = dir.join("config.toml");
    fs::write(&config_path, "interval = \"soon\"\n").expect("failed to write the broken config");

    let output = run_dwall([
        OsStr::new("--check-config"),
        OsStr::new("--config"),
        config_path.as_os_str(),
    ]);

    assert_exit(&output, 3);

    let stderr = text(&output.stderr);
    assert!(
        stderr.contains("configuration invalid:"),
        "the error does not say the configuration is invalid:\n{stderr}"
    );
    assert!(
        stderr.contains("config.toml"),
        "the error does not name the offending file:\n{stderr}"
    );

    let _ = fs::remove_dir_all(&dir);
}

/// `--check-config` treats a missing file as the defaults, not as an error.
#[test]
fn check_config_on_a_missing_file_reports_defaults_and_exits_zero() {
    let dir = scratch_dir("missing_config");
    let config_path = dir.join("config.toml");
    assert!(!config_path.exists());

    let output = run_dwall([
        OsStr::new("--check-config"),
        OsStr::new("--config"),
        config_path.as_os_str(),
    ]);

    assert_exit(&output, 0);

    let stdout = text(&output.stdout);
    assert!(
        stdout.contains("configuration OK:"),
        "the summary does not confirm the configuration:\n{stdout}"
    );
    assert!(
        stdout.contains("the file does not exist yet"),
        "a missing file is not reported as such:\n{stdout}"
    );

    // The documented defaults, asserted without the theme-directory lines,
    // which describe the real profile and are not this test's business.
    assert!(
        stdout.contains("interval: 15s"),
        "the default interval is not reported:\n{stdout}"
    );
    assert!(
        stdout.contains("image format: Jpeg"),
        "the default image format is not reported:\n{stdout}"
    );
    assert!(
        stdout.contains("auto-detect colour scheme: true"),
        "the default colour-scheme setting is not reported:\n{stdout}"
    );

    let _ = fs::remove_dir_all(&dir);
}

/// `--once` refuses to apply a second cycle while a daemon runs: exit code 5.
///
/// The daemon's guard is its named mutex, so the test holds that mutex itself
/// instead of starting a real daemon or relying on one being installed. The
/// child then refuses before it reads a configuration or touches the desktop.
#[test]
fn once_refuses_with_five_while_a_daemon_runs() {
    let dir = scratch_dir("once_refused");
    let config_path = dir.join("config.toml");
    fs::write(&config_path, inert_config(&dir)).expect("failed to write the scratch config");

    let instance = hold_daemon_mutex();
    assert!(
        dwall::daemon_is_running(),
        "the test must hold the daemon's own mutex for this test to mean anything"
    );

    let output = run_dwall([
        OsStr::new("--once"),
        OsStr::new("--config"),
        config_path.as_os_str(),
    ]);

    assert_exit(&output, 5);

    let stderr = text(&output.stderr);
    assert!(
        stderr.contains("already running"),
        "--once did not explain the refusal:\n{stderr}"
    );

    drop(instance);
    let _ = fs::remove_dir_all(&dir);
}
