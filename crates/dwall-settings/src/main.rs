// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::path::PathBuf;

use logging::Logger;
use tauri::{Manager, RunEvent};
use tokio::sync::OnceCell;

use crate::app::commands;
use crate::app::setup::setup_app;
use crate::error::DwallSettingsResult;
use crate::infrastructure::process::lifecycle::terminate_process;

mod app;
mod domain;
mod error;
mod infrastructure;
mod services;

#[macro_use]
extern crate logging;

pub static DAEMON_EXE_PATH: OnceCell<PathBuf> = OnceCell::const_new();

#[tokio::main]
async fn main() -> DwallSettingsResult<()> {
    #[cfg(debug_assertions)]
    Logger::default().with_target("dwall").init()?;
    #[cfg(not(debug_assertions))]
    {
        use dwall::lazy::DWALL_LOG_DIR;

        let package_name = env!("CARGO_PKG_NAME");
        Logger::default()
            .with_target("dwall")
            .with_file_path(DWALL_LOG_DIR.join(format!("{}.log", package_name)))?
            .init()?;
    }

    let builder = tauri::Builder::default()
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_single_instance::init(|app, _argv, _cwd| {
            if let Some(w) = app.get_webview_window("main") {
                info!("Application instance already running, focusing existing window");
                if let Err(e) = w.set_focus() {
                    error!(error = %e, "Failed to set focus on existing window");
                }
            } else if let Err(e) = crate::infrastructure::window::builder::build_window(
                app,
                "main",
                "Dwall Settings",
                660.0,
                600.0,
            ) {
                error!(error = %e, "Failed to create new main window");
            } else {
                debug!("New main window created");
            }
        }))
        .setup(setup_app)
        .invoke_handler(tauri::generate_handler![
            commands::window::show_window,
            commands::config::read_config_file,
            commands::config::write_config_file,
            commands::theme::validate_theme_cmd,
            commands::theme::apply_theme_cmd,
            commands::theme::get_applied_theme_id_cmd,
            commands::system::check_auto_start,
            commands::system::disable_auto_start,
            commands::system::enable_auto_start,
            commands::download::download_theme_cmd,
            commands::download::cancel_theme_download_cmd,
            commands::system::request_location_permission,
            commands::window::open_dir,
            commands::window::open_config_dir,
            commands::window::open_log_dir,
            commands::window::set_titlebar_color_mode,
            commands::system::move_directory_cmd,
            commands::system::kill_daemon_cmd,
            commands::thumbnail::get_or_save_cached_thumbnails_cmd,
            commands::thumbnail::clear_thumbnail_cache_cmd,
            commands::monitor::get_monitors_cmd,
            commands::system::open_privacy_location_settings,
            commands::system::check_for_updates_cmd,
            commands::theme::get_customized_themes_cmd,
        ]);

    if cfg!(debug_assertions) {
        builder.build(tauri::generate_context!())?.run(|_, event| {
            if let RunEvent::Exit = event {
                match crate::infrastructure::process::finder::find_process_by_path(
                    DAEMON_EXE_PATH.get().unwrap(),
                ) {
                    Ok(Some(pid)) => match terminate_process(pid) {
                        Ok(_) => debug!("Daemon process killed on exit"),
                        Err(e) => error!(error = %e, "Failed to kill daemon process on exit"),
                    },
                    Ok(None) => debug!("No daemon process to kill on exit"),
                    Err(e) => error!(error = %e, "Failed to find daemon process on exit"),
                }
            }
        })
    } else {
        builder.run(tauri::generate_context!())?
    }

    info!("Dwall Settings application run completed");
    Ok(())
}
