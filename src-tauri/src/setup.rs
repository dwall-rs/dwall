use std::sync::Arc;

use tauri::Manager;
use time::macros::{format_description, offset};
use tokio::sync::Mutex;
use tracing::Level;
use tracing_subscriber::fmt::time::OffsetTime;

use crate::{theme::CloseTaskSender, tray::build_tray, window::new_main_window};

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
        "Setting up application: version {}",
        app.package_info().version
    );

    build_tray(app)?;

    #[cfg(all(desktop, not(debug_assertions)))]
    setup_updater(app)?;

    new_main_window(app.app_handle())?;

    let channel: CloseTaskSender = Arc::new(Mutex::new(None));
    app.manage(channel);

    info!("Application setup completed");

    Ok(())
}

#[cfg(all(desktop, not(debug_assertions)))]
fn setup_updater(app: &mut tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    info!("Initializing update plugin");
    app.handle()
        .plugin(tauri_plugin_updater::Builder::new().build())?;

    info!("Spawning update check task");
    let handle = app.handle().clone();
    tauri::async_runtime::spawn(async move {
        if let Err(e) = crate::update::update(handle).await {
            error!("Failed to check for updates: {:?}", e);
        }
    });

    Ok(())
}
