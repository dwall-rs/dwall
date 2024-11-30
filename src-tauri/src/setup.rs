use std::{env, sync::Arc};

use tauri::Manager;
use time::macros::{format_description, offset};
use tokio::sync::Mutex;
use tracing::Level;
use tracing_subscriber::fmt::time::OffsetTime;

use crate::{
    config::read_config_file,
    theme::{apply_theme, CloseTaskSender},
    tray::build_tray,
    window::new_main_window,
};

pub fn setup_logging() {
    let fmt = if cfg!(debug_assertions) {
        format_description!("[hour]:[minute]:[second].[subsecond digits:3]")
    } else {
        format_description!("[year]-[month]-[day] [hour]:[minute]:[second].[subsecond digits:3]")
    };

    let timer = OffsetTime::new(offset!(+8), fmt);

    #[cfg(all(desktop, not(debug_assertions)))]
    let writer = {
        use crate::lazy::APP_CONFIG_DIR;
        use std::{fs::File, sync::Mutex};

        let log_file =
            File::create(APP_CONFIG_DIR.join("dwall.log")).expect("Failed to create the log file");
        Mutex::new(log_file)
    };

    #[cfg(any(debug_assertions, mobile))]
    let writer = std::io::stderr;

    let builder = tracing_subscriber::fmt()
        .with_max_level(Level::TRACE)
        .with_file(true)
        .with_line_number(true)
        .with_env_filter("dwall_lib")
        .with_target(false)
        .with_timer(timer)
        .with_writer(writer);

    if cfg!(debug_assertions) {
        builder.init();
    } else {
        builder.json().init();
    }
}

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

    build_tray(app)?;

    #[cfg(all(desktop, not(debug_assertions)))]
    setup_updater(app)?;

    let channel: CloseTaskSender = Arc::new(Mutex::new(None));
    app.manage(channel);

    // Apply theme asynchronously
    info!("Preparing to apply theme on launch");
    let handle = app.handle().clone();
    tauri::async_runtime::spawn(async move {
        match read_config_file().await {
            Ok(config) => {
                let sender = handle.state::<CloseTaskSender>();
                if let Err(e) = apply_theme(sender, config).await {
                    error!("Failed to apply theme: {}", e);
                }
            }
            Err(e) => error!("Failed to read config file: {}", e),
        }
    });

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
