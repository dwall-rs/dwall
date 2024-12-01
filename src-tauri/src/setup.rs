use std::env;

use tauri::Manager;

use crate::window::new_main_window;

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

    // Conditionally create main window
    if !args.contains(&"--auto-start".to_string()) {
        info!("Auto-start not enabled, creating main window");
        new_main_window(app.app_handle())?;
    } else {
        info!("Auto-start enabled, skipping main window creation");
    }

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
