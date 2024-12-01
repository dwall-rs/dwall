use std::{env, path::PathBuf, str::FromStr};

use tauri::Manager;

use crate::{
    auto_start::AutoStartManager, error::DwallSettingsError, window::new_main_window,
    DAEMON_EXE_PATH,
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

    #[cfg(all(desktop, not(debug_assertions)))]
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
        error!("Daemon exe is not exists");
        return Err(Box::new(std::io::Error::from(std::io::ErrorKind::NotFound)));
    }
    info!(path = %daemon_exe_path.display(), "Found daemon exe");
    DAEMON_EXE_PATH.set(daemon_exe_path)?;

    let auto_start_manager = AutoStartManager::new();
    app.manage(auto_start_manager);

    info!("Creating main window");
    new_main_window(app.app_handle())?;

    info!("Application setup completed successfully");

    Ok(())
}

#[cfg(all(desktop, not(debug_assertions)))]
fn setup_updater(app: &mut tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    info!("Initializing update plugin");

    // Initialize update plugin
    app.handle()
        .plugin(tauri_plugin_updater::Builder::new().build())
        .map_err(|e| {
            error!("Failed to initialize update plugin: {}", e);
            e
        })?;

    // Spawn update check task
    info!("Scheduling background update check");
    let handle = app.handle().clone();
    tauri::async_runtime::spawn(async move {
        match crate::update::update(handle).await {
            Ok(_) => info!("Update check completed successfully"),
            Err(e) => error!("Update check failed: {}", e),
        }
    });

    Ok(())
}
