use error::DwallResult;
use event::run_callback;
use tauri::{AppHandle, Manager};

use crate::config::{read_config_file, write_config_file};
use crate::geo::get_geo_postion;
use crate::setup::setup_app;
use crate::theme::{apply_theme, check_theme_exists, close_last_theme_task};

mod color_mode;
mod config;
mod download;
mod error;
mod event;
mod geo;
mod lazy;
mod setup;
mod solar;
mod theme;
mod tray;
mod update;

#[macro_use]
extern crate tracing;

/// 防止启动时闪白屏
#[tauri::command]
async fn show_main_window(app: AppHandle) {
    debug!("Showing main window");

    let main_window = app.get_webview_window("main").unwrap();

    main_window.show().unwrap();
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() -> DwallResult<()> {
    get_geo_postion().unwrap();
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _argv, _cwd| {
            if let Some(w) = app.get_webview_window("main") {
                info!("Application instance already running, focusing existing window");
                w.set_focus().unwrap();
            }
        }))
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(setup_app)
        .invoke_handler(tauri::generate_handler![
            show_main_window,
            read_config_file,
            write_config_file,
            check_theme_exists,
            apply_theme,
            close_last_theme_task
        ])
        .build(tauri::generate_context!())?
        .run(run_callback);
    Ok(())
}
