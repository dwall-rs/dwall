// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::path::PathBuf;

use dwall::setup_logging;
use tauri::{Manager, RunEvent};
use tokio::sync::OnceCell;

use crate::app::{commands, setup::setup_app};
use crate::error::DwallSettingsResult;
use crate::infrastructure::process::kill_daemon;

mod app;
mod domain;
mod error;
mod i18n;
mod infrastructure;
mod services;
mod utils;

#[macro_use]
extern crate tracing;

pub static DAEMON_EXE_PATH: OnceCell<PathBuf> = OnceCell::const_new();

#[tokio::main]
async fn main() -> DwallSettingsResult<()> {
    if cfg!(not(debug_assertions)) && cfg!(not(feature = "log-max-level-info")) {
        std::env::set_var("DWALL_LOG", "debug");
    }

    setup_logging(&["dwall_settings", "dwall"]);

    let builder = tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_single_instance::init(|app, _argv, _cwd| {
            if let Some(w) = app.get_webview_window("main") {
                info!("Application instance already running, focusing existing window");
                if let Err(e) = w.set_focus() {
                    error!(error = %e, "Failed to set focus on existing window");
                }
            } else {
                match crate::infrastructure::window::create_main_window(app) {
                    Ok(_) => debug!("New main window created"),
                    Err(e) => error!(error = %e, "Failed to create new main window"),
                }
            }
        }))
        .setup(setup_app)
        .invoke_handler(tauri::generate_handler![
            commands::show_window,
            commands::read_config_file,
            commands::write_config_file,
            commands::validate_theme_cmd,
            commands::apply_theme_cmd,
            commands::get_applied_theme_id_cmd,
            commands::check_auto_start,
            commands::disable_auto_start,
            commands::enable_auto_start,
            commands::download_theme_cmd,
            commands::cancel_theme_download_cmd,
            commands::request_location_permission,
            commands::open_dir,
            commands::open_config_dir,
            commands::open_log_dir,
            commands::set_titlebar_color_mode,
            commands::move_themes_directory_cmd,
            commands::kill_daemon_cmd,
            commands::get_or_save_cached_thumbnails_cmd,
            commands::clear_thumbnail_cache_cmd,
            commands::get_translations_cmd,
            commands::get_monitors_cmd,
            commands::open_privacy_location_settings
        ]);

    if cfg!(debug_assertions) {
        builder.build(tauri::generate_context!())?.run(|_, event| {
            if let RunEvent::Exit = event {
                match kill_daemon() {
                    Ok(_) => debug!("Daemon process killed on exit"),
                    Err(e) => error!(error = %e, "Failed to kill daemon process on exit"),
                }
            }
        })
    } else {
        builder.run(tauri::generate_context!())?
    }

    info!("Dwall Settings application run completed");
    Ok(())
}
