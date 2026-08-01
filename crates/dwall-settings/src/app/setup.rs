//! Application setup

use std::{env, path::PathBuf, str::FromStr};

use tauri::Manager;

use crate::{
    DAEMON_EXE_PATH,
    infrastructure::{
        network::http_client::HttpClient, process::finder::find_process_by_path,
        window::builder::build_window,
    },
    services::{
        daemon::launcher::DaemonLauncher, theme::downloader::ThemeDownloader,
        thumbnail::ThumbnailCache,
    },
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

    tokio::spawn(async move {
        let args: Vec<String> = env::args().collect();
        debug!(arguments = ?args, "Launch arguments");

        let settings_exe_path = match PathBuf::from_str(&args[0]) {
            Ok(path) => path,
            Err(e) => {
                error!("Failed to parse settings exe path: {}", e);
                panic!("Failed to parse settings exe path: {}", e);
            }
        };

        let daemon_exe_path = match settings_exe_path.parent() {
            Some(path) => path.join("dwall.exe"),
            None => {
                error!("Failed to find parent directory of settings exe");
                panic!("Failed to find parent directory of settings exe");
            }
        };

        if !daemon_exe_path.exists() || !daemon_exe_path.is_file() {
            error!("Daemon executable does not exist");
            panic!("Daemon executable does not exist");
        }

        info!(path = %daemon_exe_path.display(), "Found daemon exe");
        if let Err(e) = DAEMON_EXE_PATH.set(daemon_exe_path) {
            error!("Failed to set daemon exe path: {}", e);
            panic!("Failed to set daemon exe path: {}", e);
        }
    });

    let config_path = dwall::DWALL_CONFIG_DIR.join("config.toml");
    let config = dwall::infrastructure::filesystem::config_reader::ConfigReader::read_from_path(
        &config_path,
    )?;
    let http_client = HttpClient::new(config.network())?;

    let theme_downloader = ThemeDownloader::new(http_client.clone());
    app.manage(theme_downloader);

    let thumbnail_cache = ThumbnailCache::new(http_client.clone());
    app.manage(thumbnail_cache);

    build_window(app.app_handle(), "main", "Dwall Settings", 660.0, 600.0)?;

    tokio::spawn(async move { crate::app::tracker::track().await });

    tokio::spawn(async move {
        let _ = find_process_by_path(crate::DAEMON_EXE_PATH.get().unwrap())
            .and_then(|pid| pid.map_or_else(|| DaemonLauncher::launch().map(|_| ()), |_| Ok(())));
    });

    info!("Application setup completed successfully");

    Ok(())
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
