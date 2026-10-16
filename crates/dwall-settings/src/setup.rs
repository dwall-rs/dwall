//! Application setup

use std::{env, path::PathBuf};

use tauri::Manager;

use crate::{
    DAEMON_EXE_PATH, daemon::DaemonLauncher, network::HttpClient,
    theme::downloader::ThemeDownloader, thumbnail::ThumbnailCache, window::build_window,
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
    let should_run_daemon = crate::daemon::should_run(&config);
    let http_client = HttpClient::new(config.network())?;

    let theme_downloader = ThemeDownloader::new(http_client.clone());
    app.manage(theme_downloader);

    let thumbnail_cache = ThumbnailCache::new(http_client.clone());
    app.manage(thumbnail_cache);

    build_window(app.app_handle(), "main", "Dwall Settings")?;

    tokio::spawn(async move { crate::tracker::track().await });

    // Only start the daemon when the configuration asks for it; otherwise "stop
    // engine" would be undone every time the settings window opens.
    if !should_run_daemon {
        debug!("Configuration does not require the daemon; not starting it");
        info!("Application setup completed successfully");
        return Ok(());
    }

    tokio::spawn(async move {
        // A daemon installed anywhere in this session counts, not only the copy
        // next to this executable.
        let already_running = crate::daemon::is_daemon_running().unwrap_or(false);
        if !already_running && let Err(e) = DaemonLauncher::launch() {
            error!(error = %e, "Failed to start the daemon");
        }
    });

    info!("Application setup completed successfully");

    Ok(())
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
