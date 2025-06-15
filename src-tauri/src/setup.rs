use std::{env, path::PathBuf, str::FromStr};

use tauri::Manager;

use crate::{
    download::ThemeDownloader, error::DwallSettingsError, process_manager::find_daemon_process,
    read_config_file, theme::launch_daemon, window::create_main_window, DAEMON_EXE_PATH,
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

    //#[cfg(all(desktop, not(debug_assertions)))]
    setup_updater(app)?;

    // Process launch arguments
    let args: Vec<String> = env::args().collect();
    debug!(arguments = ?args, "Launch arguments");

    let settings_exe_path = PathBuf::from_str(&args[0])?;
    let daemon_exe_path = settings_exe_path
        .parent()
        .ok_or(DwallSettingsError::Io(std::io::ErrorKind::NotFound.into()))?
        .join("dwall.exe");
    if !daemon_exe_path.exists() || !daemon_exe_path.is_file() {
        error!("Daemon executable does not exist");
        return Err(Box::new(std::io::Error::from(std::io::ErrorKind::NotFound)));
    }
    info!(path = %daemon_exe_path.display(), "Found daemon exe");
    DAEMON_EXE_PATH.set(daemon_exe_path)?;

    let theme_downloader = ThemeDownloader::new();
    app.manage(theme_downloader);

    create_main_window(app.app_handle())?;

    // If a theme is configured in the configuration file but the background process is not detected,
    // then run the background process when this program starts.
    tauri::async_runtime::spawn(async move {
        let _ = read_config_file()
            .await
            .and_then(|_| find_daemon_process())
            .and_then(|pid| pid.map_or_else(|| launch_daemon().map(|_| ()), |_| Ok(())));
    });

    info!("Application setup completed successfully");

    Ok(())
}

//#[cfg(all(desktop, not(debug_assertions)))]
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
