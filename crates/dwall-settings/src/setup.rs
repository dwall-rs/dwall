//! Application setup

use std::{env, path::PathBuf};

use tauri::Manager;

use crate::{
    DAEMON_EXE_PATH,
    daemon::{DaemonLauncher, DaemonStartup, daemon_path, plan_daemon_startup},
    network::HttpClient,
    process::stop_daemon,
    theme::downloader::ThemeDownloader,
    thumbnail::ThumbnailCache,
    window::build_window,
};

pub fn setup_app(app: &mut tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    info!(
        version = app.package_info().version.to_string(),
        build_mode = if cfg!(debug_assertions) {
            "debug"
        } else {
            "release"
        },
        "Starting application"
    );

    setup_updater(app)?;

    // Resolve the daemon path synchronously so engine commands work as soon as
    // the webview loads. Previously this ran in a spawned task and could race
    // with the frontend's first `get_engine_status` call.
    let args: Vec<String> = env::args().collect();
    debug!(arguments = ?args, "Launch arguments");

    match resolve_daemon_path(&args) {
        Ok(path) => {
            info!(path = %path.display(), "Found daemon exe");
            if DAEMON_EXE_PATH.set(path).is_err() {
                error!("Failed to set daemon exe path");
            }
        }
        Err(e) => error!(error = %e, "Failed to resolve daemon executable path"),
    }

    let config_path = dwall::DWALL_CONFIG_DIR.join("config.toml");
    let config = match dwall::config::ConfigReader::read_from_path(&config_path) {
        Ok(config) => config,
        Err(e) => {
            // A corrupt config must not stop the settings window from opening: the
            // user may have opened it precisely to fix the configuration.
            error!(error = %e, "Failed to read the configuration, falling back to defaults");
            dwall::Config::default()
        }
    };
    // Before anything can ask the webview to show an image.
    allow_asset_directories(app.handle(), &config);

    let should_run_daemon = crate::daemon::should_run(&config);
    // A hand-edited or stale configuration can still name a proxy this machine
    // cannot build a client for. The window has to open anyway: it is the only place
    // the user can correct the value.
    let http_client = match HttpClient::new(config.network()) {
        Ok(client) => client,
        Err(e) => {
            warn!(
                error = %e,
                "Failed to build the HTTP client from the configured proxy; using a direct connection"
            );
            HttpClient::new(None)?
        }
    };

    let theme_downloader = ThemeDownloader::new(http_client.clone());
    app.manage(theme_downloader);

    let thumbnail_cache = ThumbnailCache::new(http_client.clone());
    app.manage(thumbnail_cache);

    // The update check needs the same client, so that the configured proxy applies to
    // the release list it reads for pre-release builds.
    app.manage(http_client);

    build_window(app.app_handle(), "main", "Dwall Settings")?;

    // Only start the daemon when the configuration asks for it; otherwise "stop
    // engine" would be undone every time the settings window opens.
    if !should_run_daemon {
        debug!("Configuration does not require the daemon; not starting it");
        info!("Application setup completed successfully");
        return Ok(());
    }

    // `tauri::async_runtime::spawn`, not `tokio::spawn`: this runs from the setup hook on
    // the main thread, where no Tokio runtime is current. Tauri creates its own runtime on
    // first use, so this works from here; `tokio::spawn` panicked with "there is no reactor
    // running".
    tauri::async_runtime::spawn(async move {
        // A daemon installed anywhere in this session counts, not only the copy
        // next to this executable.
        let already_running = crate::daemon::is_daemon_running().unwrap_or(false);

        match plan_daemon_startup(cfg!(debug_assertions), already_running) {
            DaemonStartup::Start => {
                if let Err(e) = DaemonLauncher::launch() {
                    error!(error = %e, "Failed to start the daemon");
                }
            }
            DaemonStartup::Adopt => {
                info!("A daemon is already running, leaving it alone");
            }
            DaemonStartup::Replace => {
                // A debug build must run the daemon it was built alongside, so the
                // installed one is stopped first rather than adopted: adopting it
                // would leave the dev session exercising a release binary.
                // `stop_daemon` waits on the process handle (up to five seconds), so it
                // runs off the async runtime thread that drives the window.
                match tauri::async_runtime::spawn_blocking(stop_daemon).await {
                    Ok(Ok(Some(pid))) => info!(
                        pid = pid,
                        "Stopped the running daemon to start this debug build's daemon"
                    ),
                    Ok(Ok(None)) => info!("No daemon process found to stop"),
                    Ok(Err(e)) => error!(error = %e, "Failed to stop the running daemon"),
                    Err(e) => error!(error = %e, "Stopping the running daemon failed"),
                }

                match daemon_path() {
                    Ok(path) => info!(path = %path.display(), "Starting the debug daemon"),
                    Err(e) => error!(error = %e, "Daemon executable path not configured"),
                }

                if let Err(e) = DaemonLauncher::launch() {
                    error!(error = %e, "Failed to start the debug daemon");
                }
            }
        }
    });

    info!("Application setup completed successfully");

    Ok(())
}

/// Lets the asset protocol read the directories the webview shows images from.
///
/// The scope in `tauri.conf.json` used to be `**/*`, which let the renderer ask for any
/// file on the disk. A pattern there cannot be narrowed to what is actually needed either:
/// the themes directory is whatever the configuration says, and it can be moved. So the
/// configuration is read instead, and only these three directories are reachable through
/// `asset:` — a theme's frames live in the first two, and the library's thumbnails in the
/// cache.
pub fn allow_asset_directories(app: &tauri::AppHandle, config: &dwall::Config) {
    let scope = app.asset_protocol_scope();
    let directories = [
        config.themes_directory().to_path_buf(),
        config.customized_themes_directory().to_path_buf(),
        dwall::DWALL_CACHE_DIR.join("thumbnails"),
    ];

    for directory in directories {
        match scope.allow_directory(&directory, true) {
            Ok(()) => debug!(
                path = %directory.display(),
                "Allowed the asset protocol to read a directory"
            ),
            Err(e) => error!(
                error = %e,
                path = %directory.display(),
                "Failed to allow the asset protocol to read a directory"
            ),
        }
    }
}

/// Locate `dwall.exe` next to the settings executable.
fn resolve_daemon_path(args: &[String]) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let settings_exe_path = args.first().ok_or("Missing executable path in arguments")?;
    let daemon_exe_path = PathBuf::from(settings_exe_path)
        .parent()
        .ok_or("Failed to find parent directory of settings exe")?
        .join("dwall.exe");

    if !daemon_exe_path.is_file() {
        return Err(format!(
            "Daemon executable does not exist at {}",
            daemon_exe_path.display()
        )
        .into());
    }

    Ok(daemon_exe_path)
}

fn setup_updater(app: &mut tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    debug!("Initializing update plugin");

    app.handle()
        .plugin(tauri_plugin_updater::Builder::new().build())
        .map_err(|e| {
            error!(error = %e, "Failed to initialize update plugin");
            e
        })?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;

    /// A directory holding a settings executable, and optionally a daemon next to it.
    fn scratch(name: &str, with_daemon: bool) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("dwall_setup_{name}"));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();

        let settings = dir.join("dwall-settings.exe");
        fs::write(&settings, b"").unwrap();

        if with_daemon {
            fs::write(dir.join("dwall.exe"), b"").unwrap();
        }

        settings
    }

    #[test]
    fn the_daemon_is_looked_for_next_to_the_settings_executable() {
        let settings = scratch("beside", true);

        let resolved = resolve_daemon_path(&[settings.display().to_string()]).unwrap();

        assert_eq!(resolved, settings.parent().unwrap().join("dwall.exe"));
    }

    #[test]
    fn a_missing_daemon_is_an_error_rather_than_a_guess() {
        let settings = scratch("missing", false);

        let error = resolve_daemon_path(&[settings.display().to_string()]).unwrap_err();

        assert!(
            error.to_string().contains("does not exist"),
            "the message names what is missing: {error}"
        );
    }

    #[test]
    fn a_directory_named_like_the_daemon_is_not_one() {
        let settings = scratch("directory", false);
        fs::create_dir(settings.parent().unwrap().join("dwall.exe")).unwrap();

        assert!(
            resolve_daemon_path(&[settings.display().to_string()]).is_err(),
            "an executable is a file, not a directory with that name"
        );
    }

    #[test]
    fn no_arguments_is_an_error() {
        assert!(resolve_daemon_path(&[]).is_err());
    }
}
