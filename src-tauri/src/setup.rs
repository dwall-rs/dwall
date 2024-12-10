use std::{env, path::PathBuf, str::FromStr};

use tauri::Manager;

use crate::{
    auto_start::AutoStartManager, error::DwallSettingsError, process_manager::find_daemon_process,
    read_config_file, theme::spawn_apply_daemon, window::create_main_window, DAEMON_EXE_PATH,
};

pub fn setup_app(app: &mut tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    info!(
        "Starting application: version {}, build mode: {}",
        app.package_info().version,
        if cfg!(debug_assertions) {
            "debug"
        } else {
            "release"
        }
    );

    //#[cfg(all(desktop, not(debug_assertions)))]
    setup_updater(app)?;

    // Process launch arguments
    let args: Vec<String> = env::args().collect();
    debug!("Launch arguments: {:?}", args);

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

    let auto_start_manager = AutoStartManager::new();
    app.manage(auto_start_manager);

    info!("Creating main window");
    create_main_window(app.app_handle())?;

    // If a theme is configured in the configuration file but the background process is not detected,
    // then run the background process when this program starts.
    tauri::async_runtime::spawn(async move {
        let _ = read_config_file()
            .await
            .and_then(|config| {
                config
                    .theme_id()
                    .map_or(Ok(None), |_| find_daemon_process())
            })
            .and_then(|pid| pid.map_or_else(|| spawn_apply_daemon().map(|_| ()), |_| Ok(())));
    });

    info!("Application setup completed successfully");

    Ok(())
}

//#[cfg(all(desktop, not(debug_assertions)))]
fn setup_updater(app: &mut tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    info!("Initializing update plugin");

    // Initialize update plugin
    app.handle()
        .plugin(tauri_plugin_updater::Builder::new().build())
        .map_err(|e| {
            error!("Failed to initialize update plugin: {}", e);
            e
        })?;

    Ok(())
}
