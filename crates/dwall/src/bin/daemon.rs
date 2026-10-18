#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

//! Dwall daemon entry point.
//!
//! Besides running the wallpaper engine this binary is the standalone control
//! surface: `--check-config` validates a configuration, `--once` applies a single
//! cycle without leaving a process behind, and `--install-autostart` /
//! `--uninstall-autostart` manage the per-user auto-start entry. See
//! `docs/headless.md`.

use std::ffi::OsString;
use std::path::PathBuf;
use std::process::ExitCode;

use dwall::config::{Config, ConfigReader, config_path};
use dwall::{DaemonApplication, DaemonInstance};
use logging::Logger;

/// Exit codes, also documented in `--help` and `docs/headless.md`.
mod exit {
    /// Success. Also returned by `run` when another daemon already runs.
    pub const SUCCESS: u8 = 0;
    /// The daemon stopped because of a fatal error.
    pub const FATAL: u8 = 1;
    /// The command line was not understood.
    pub const USAGE: u8 = 2;
    /// The configuration is invalid or unreadable.
    pub const INVALID_CONFIG: u8 = 3;
    /// Auto-start registration failed.
    pub const AUTOSTART: u8 = 4;
    /// `--once` refused because a daemon already runs.
    pub const ALREADY_RUNNING: u8 = 5;
    /// The position could not be resolved because the user has location switched off.
    pub const LOCATION_UNAVAILABLE: u8 = 6;
}

const USAGE: &str = "\
Dwall — solar-position dynamic wallpaper daemon

USAGE:
    dwall [OPTIONS]

OPTIONS:
    --config <PATH>        Use a configuration file other than
                           %APPDATA%\\dwall\\config.toml
    --once                 Apply one update cycle and exit
    --check-config         Validate the configuration and exit
    --install-autostart    Start automatically at sign-in
    --uninstall-autostart  Remove the auto-start entry
    -h, --help             Print this help
    -V, --version          Print the version

EXIT CODES:
    0  success (also when a daemon is already running)
    1  the daemon stopped because of a fatal error
    2  the command line was not understood
    3  the configuration is invalid or unreadable
    4  auto-start registration failed
    5  --once refused because a daemon is already running
    6  no location: automatic coordinates are on but Windows has location switched off
       or denied it for this app (the log line names the setting that fixes it)

FILES:
    config  %APPDATA%\\dwall\\config.toml
    logs    %LOCALAPPDATA%\\com.thep0y.dwall\\log\\dwall.log  (DWALL_LOG=debug)
";

/// What the command line asked for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Command {
    Run,
    Once,
    CheckConfig,
    InstallAutostart,
    UninstallAutostart,
    Help,
    Version,
}

#[derive(Debug, PartialEq, Eq)]
struct Options {
    command: Command,
    config_path: Option<PathBuf>,
}

/// Parses the command line.
///
/// Hand-rolled: the daemon is size-sensitive, and the surface is seven flags.
fn parse_args(args: impl IntoIterator<Item = OsString>) -> Result<Options, String> {
    let mut command = Command::Run;
    let mut command_given = false;
    let mut config_path = None;
    let mut args = args.into_iter();

    while let Some(arg) = args.next() {
        let arg = arg.to_string_lossy().into_owned();

        match arg.as_str() {
            "-h" | "--help" => {
                return Ok(Options {
                    command: Command::Help,
                    config_path: None,
                });
            }
            "-V" | "--version" => {
                return Ok(Options {
                    command: Command::Version,
                    config_path: None,
                });
            }
            "--once" | "--check-config" | "--install-autostart" | "--uninstall-autostart" => {
                if command_given {
                    return Err(format!("only one command may be given (got {arg})"));
                }

                command = match arg.as_str() {
                    "--once" => Command::Once,
                    "--check-config" => Command::CheckConfig,
                    "--install-autostart" => Command::InstallAutostart,
                    _ => Command::UninstallAutostart,
                };
                command_given = true;
            }
            "--config" => {
                let value = args
                    .next()
                    .ok_or_else(|| "--config needs a path".to_string())?;
                config_path = Some(PathBuf::from(value));
            }
            other if other.starts_with("--config=") => {
                config_path = Some(PathBuf::from(&other["--config=".len()..]));
            }
            other => return Err(format!("unknown argument: {other}")),
        }
    }

    if config_path.is_some()
        && matches!(
            command,
            Command::InstallAutostart | Command::UninstallAutostart
        )
    {
        return Err("--config cannot be combined with an auto-start command".to_string());
    }

    Ok(Options {
        command,
        config_path,
    })
}

/// Attaches to the console the command was typed into, if there is one.
///
/// Release builds are GUI-subsystem executables, so without this a command run
/// from a terminal produces no output at all. `std` resolves its stdio handles
/// lazily, so attaching before the first write is what makes `println!` land in
/// that console.
#[cfg(windows)]
fn attach_parent_console() {
    use windows::Win32::System::Console::{ATTACH_PARENT_PROCESS, AttachConsole};

    unsafe {
        let _ = AttachConsole(ATTACH_PARENT_PROCESS);
    }
}

#[cfg(not(windows))]
fn attach_parent_console() {}

fn main() -> ExitCode {
    let options = match parse_args(std::env::args_os().skip(1)) {
        Ok(options) => options,
        Err(message) => {
            attach_parent_console();
            eprintln!("error: {message}\n\n{USAGE}");
            return ExitCode::from(exit::USAGE);
        }
    };

    attach_parent_console();

    match options.command {
        Command::Help => {
            println!("{USAGE}");
            ExitCode::from(exit::SUCCESS)
        }
        Command::Version => {
            println!("{}", version_string());
            ExitCode::from(exit::SUCCESS)
        }
        Command::CheckConfig => check_config(options.config_path),
        Command::InstallAutostart => install_autostart(),
        Command::UninstallAutostart => uninstall_autostart(),
        Command::Once => run(options, true),
        Command::Run => run(options, false),
    }
}

/// Runs the daemon (or a single cycle), claiming the per-session instance mutex.
fn run(options: Options, once: bool) -> ExitCode {
    if let Err(e) = init_logger() {
        eprintln!("error: failed to initialise logging: {e}");
        return ExitCode::from(exit::FATAL);
    }

    // The guard has to stay alive for the whole run; it is what marks this process
    // as *the* daemon of this session.
    let _instance: DaemonInstance = match dwall::acquire_daemon_instance() {
        Ok(Some(instance)) => instance,
        Ok(None) if once => {
            // Applying a second cycle while a daemon is running would fight over
            // the same wallpaper, so refuse loudly instead of doing nothing quietly.
            eprintln!("error: a dwall daemon is already running; not applying a second cycle");
            return ExitCode::from(exit::ALREADY_RUNNING);
        }
        Ok(None) => {
            // Normal for a double auto-start (Task Scheduler plus the Run key, or a
            // second copy started by hand): the other instance owns the desktop.
            println!("dwall is already running; this instance exits");
            return ExitCode::from(exit::SUCCESS);
        }
        Err(e) => {
            eprintln!("error: failed to claim the single-instance mutex: {e}");
            return ExitCode::from(exit::FATAL);
        }
    };

    let mut app = match &options.config_path {
        Some(path) => DaemonApplication::with_config_path(path.clone()),
        None => DaemonApplication::new(),
    };

    let result = if once { app.run_once() } else { app.run() };

    match result {
        Ok(()) => ExitCode::from(exit::SUCCESS),
        Err(e) => {
            let code = exit_code_for(&e);

            if code == exit::LOCATION_UNAVAILABLE {
                // Not a crash: the user's own setting leaves the daemon without a
                // position, and it deliberately does not guess one. The message already
                // names the setting that fixes it, so it goes to the durable log and to
                // the console.
                logging::error!(error = %e, "Stopping: automatic location is unavailable");
                eprintln!("error: {e}");
            } else {
                // The release binary has no console of its own, so the log file is the
                // durable record; the message is also printed for `--once` from a shell.
                logging::error!(error = %e, "Daemon stopped because of a fatal error");
                eprintln!("error: dwall stopped because of a fatal error: {e}");
            }

            ExitCode::from(code)
        }
    }
}

/// The exit code for a failed run, so callers can tell a missing location apart from a
/// crash without parsing the message. Both are documented in `docs/headless.md`.
fn exit_code_for(error: &dwall::error::DwallError) -> u8 {
    if error.location_unavailable() {
        return exit::LOCATION_UNAVAILABLE;
    }

    exit::FATAL
}

/// Initialises logging: stderr for debug builds, the log file for release builds.
fn init_logger() -> dwall::DwallResult<()> {
    let package_name = env!("CARGO_PKG_NAME");

    #[cfg(debug_assertions)]
    {
        Logger::default().with_target(package_name).init()?;
    }

    #[cfg(not(debug_assertions))]
    {
        use dwall::lazy::DWALL_LOG_DIR;

        Logger::default()
            .with_target(package_name)
            .with_file_path(DWALL_LOG_DIR.join(format!("{package_name}.log")))?
            .init()?;
    }

    Ok(())
}

/// Validates a configuration file and reports what it would do.
fn check_config(path: Option<PathBuf>) -> ExitCode {
    let path = path.unwrap_or_else(config_path);

    match ConfigReader::read_from_path(&path) {
        Ok(config) => {
            println!("configuration OK: {}", path.display());

            if !path.is_file() {
                println!(
                    "  note: the file does not exist yet; defaults are in effect and a \
                     default file is written on first run"
                );
            }

            print_summary(&config);
            ExitCode::from(exit::SUCCESS)
        }
        Err(e) => {
            eprintln!("configuration invalid: {}", path.display());
            eprintln!("  {e}");
            ExitCode::from(exit::INVALID_CONFIG)
        }
    }
}

/// Prints the non-sensitive configuration summary (never the coordinates).
fn print_summary(config: &Config) {
    println!("  interval: {}s", config.interval());
    println!("  image format: {:?}", config.image_format());
    println!(
        "  auto-detect colour scheme: {}",
        config.auto_detect_color_scheme()
    );

    for (label, directory) in [
        ("themes", config.themes_directory()),
        ("customized themes", config.customized_themes_directory()),
    ] {
        println!(
            "  {label} directory: {} ({})",
            directory.display(),
            if directory.is_dir() { "ok" } else { "missing" }
        );
    }
}

/// Registers this executable to start at sign-in.
fn install_autostart() -> ExitCode {
    let executable = match current_executable() {
        Ok(executable) => executable,
        Err(e) => {
            eprintln!("error: cannot determine the path of this executable: {e}");
            return ExitCode::from(exit::AUTOSTART);
        }
    };

    // The Run value is shared with the settings app and with every other copy, so
    // replacing an entry that points somewhere else is worth saying out loud
    // rather than overwriting it silently.
    if !dwall::is_autostart_installed(&executable).unwrap_or(false)
        && let Ok(Some(existing)) = dwall::registered_autostart_command()
    {
        println!("replacing the existing auto-start entry: {existing}");
    }

    // A debug build is a console application, so the `Run` key would give it a
    // console window at every logon; release builds are GUI-subsystem executables
    // and stay silent. Say so rather than surprising someone at sign-in.
    if dwall::executable_opens_console().unwrap_or(false) {
        eprintln!(
            "warning: this is a console build, so auto-starting it opens a console window at sign-in"
        );
        eprintln!(
            "         use a release build (bun run build:daemon) for a windowless auto-start"
        );
    }

    match dwall::install_autostart(&executable) {
        Ok(()) => {
            println!("auto-start registered: \"{}\"", executable.display());
            ExitCode::from(exit::SUCCESS)
        }
        Err(e) => {
            eprintln!("error: failed to register auto-start: {e}");
            ExitCode::from(exit::AUTOSTART)
        }
    }
}

/// Removes the auto-start entry.
fn uninstall_autostart() -> ExitCode {
    match dwall::uninstall_autostart() {
        Ok(()) => {
            println!("auto-start entry removed");
            ExitCode::from(exit::SUCCESS)
        }
        Err(e) => {
            eprintln!("error: failed to remove the auto-start entry: {e}");
            ExitCode::from(exit::AUTOSTART)
        }
    }
}

/// Path of the running executable, as Windows reports it.
///
/// Deliberately not canonicalised: the settings app compares the stored `Run`
/// value against the plain path it derives from its own directory, and a
/// `\\?\`-prefixed verbatim path would never match it.
fn current_executable() -> std::io::Result<PathBuf> {
    std::env::current_exe()
}

fn version_string() -> String {
    format!(
        "dwall {} ({} {}, {})",
        env!("CARGO_PKG_VERSION"),
        std::env::consts::OS,
        std::env::consts::ARCH,
        if cfg!(debug_assertions) {
            "debug"
        } else {
            "release"
        }
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(args: &[&str]) -> Result<Options, String> {
        parse_args(args.iter().map(OsString::from))
    }

    #[test]
    fn defaults_to_running_the_daemon() {
        assert_eq!(
            parse(&[]).unwrap(),
            Options {
                command: Command::Run,
                config_path: None
            }
        );
    }

    #[test]
    fn accepts_a_config_path_in_both_forms() {
        let expected = Some(PathBuf::from(r"C:\dwall\other.toml"));

        assert_eq!(
            parse(&["--config", r"C:\dwall\other.toml"])
                .unwrap()
                .config_path,
            expected
        );
        assert_eq!(
            parse(&["--config=C:\\dwall\\other.toml"])
                .unwrap()
                .config_path,
            expected
        );
    }

    #[test]
    fn recognises_the_commands() {
        assert_eq!(parse(&["--once"]).unwrap().command, Command::Once);
        assert_eq!(
            parse(&["--check-config"]).unwrap().command,
            Command::CheckConfig
        );
        assert_eq!(
            parse(&["--install-autostart"]).unwrap().command,
            Command::InstallAutostart
        );
        assert_eq!(
            parse(&["--uninstall-autostart"]).unwrap().command,
            Command::UninstallAutostart
        );
        assert_eq!(parse(&["-h"]).unwrap().command, Command::Help);
        assert_eq!(parse(&["--version"]).unwrap().command, Command::Version);
    }

    #[test]
    fn help_and_version_win_over_other_arguments() {
        assert_eq!(parse(&["--once", "--help"]).unwrap().command, Command::Help);
        assert_eq!(
            parse(&["--config", "x", "-V"]).unwrap().command,
            Command::Version
        );
    }

    #[test]
    fn rejects_bad_input() {
        assert!(parse(&["--nope"]).is_err());
        assert!(parse(&["--config"]).is_err(), "--config needs a value");
        assert!(
            parse(&["--once", "--check-config"]).is_err(),
            "two commands at once"
        );
        assert!(
            parse(&["--install-autostart", "--config", "x"]).is_err(),
            "--config is meaningless for auto-start"
        );
        assert!(parse(&["--check-config", "--check-config"]).is_err());
    }

    // ── Exit codes ──────────────────────────────────────────────────────────

    fn denied() -> dwall::error::DwallError {
        dwall::solar::GeolocationAccessError::Denied.into()
    }

    #[test]
    fn a_missing_location_has_its_own_exit_code() {
        assert_eq!(exit_code_for(&denied()), exit::LOCATION_UNAVAILABLE);

        let switched_off: dwall::error::DwallError =
            dwall::solar::GeolocationAccessError::Unspecified.into();
        assert_eq!(exit_code_for(&switched_off), exit::LOCATION_UNAVAILABLE);
    }

    /// Everything else is the generic failure code: the distinction only exists for the
    /// case where retrying or investigating cannot help.
    #[test]
    fn other_failures_use_the_fatal_code() {
        let other: dwall::error::DwallError =
            dwall::error::DwallError::AsyncTimeout(std::time::Duration::from_secs(30));

        assert_eq!(exit_code_for(&other), exit::FATAL);
    }

    /// The codes are part of the documented interface, so the help text has to carry
    /// them — and the new one explains what to do about it.
    #[test]
    fn the_help_text_documents_the_exit_codes() {
        for code in [
            exit::SUCCESS,
            exit::FATAL,
            exit::USAGE,
            exit::INVALID_CONFIG,
            exit::AUTOSTART,
            exit::ALREADY_RUNNING,
            exit::LOCATION_UNAVAILABLE,
        ] {
            assert!(
                USAGE.contains(&format!("\n    {code}  ")),
                "the usage text has to list exit code {code}"
            );
        }
    }
}
