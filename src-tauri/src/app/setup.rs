use std::{env, path::PathBuf, str::FromStr, sync::Arc};

use tauri::Manager;

use crate::{
    DAEMON_EXE_PATH,
    infrastructure::{
        network::{client::HttpClient, download::ThemeDownloader},
        process::find_daemon_process,
        window::create_main_window,
    },
    services::{cache::ThumbnailCache, theme_service::launch_daemon},
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
        // Process launch arguments
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

    let config = dwall::read_config_file()?;
    let http_client = Arc::new(HttpClient::create_client(config.network())?);

    let theme_downloader = ThemeDownloader::new(http_client.clone());
    app.manage(theme_downloader);

    let theme_cache = ThumbnailCache::new(http_client);
    app.manage(theme_cache);

    create_main_window(app.app_handle())?;

    tokio::spawn(async move { crate::app::tracker::track().await });

    // If a theme is configured in the configuration file but the background process is not detected,
    // then run the background process when this program starts.
    tokio::spawn(async move {
        let _ = find_daemon_process()
            .and_then(|pid| pid.map_or_else(|| launch_daemon().map(|_| ()), |_| Ok(())));
    });

    info!("Application setup completed successfully");

    Ok(())
}

fn setup_updater(app: &mut tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    debug!("Initializing update plugin");

    // Initialize update plugin
    app.handle()
        .plugin(tauri_plugin_updater::Builder::new().build())
        .map_err(|e| {
            error!(error = %e, "Failed to initialize update plugin");
            e
        })?;

    Ok(())
}
