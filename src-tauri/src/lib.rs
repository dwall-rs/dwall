use tauri::{AppHandle, Manager};

use crate::config::{read_config_file, write_config_file};
use crate::error::DwallResult;
use crate::event::run_callback;
use crate::setup::{setup_app, setup_logging};
use crate::theme::{apply_theme, close_last_theme_task, CloseTaskSender, ThemeValidator};

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
mod window;

#[macro_use]
extern crate tracing;

#[tauri::command]
async fn show_main_window(app: AppHandle) -> DwallResult<()> {
    debug!("Showing main window");

    if let Some(main_window) = app.get_webview_window("main") {
        main_window.show()?;
        main_window.set_focus()?;
    }

    Ok(())
}

#[tauri::command]
async fn check_theme_exists(theme_id: String) -> DwallResult<()> {
    ThemeValidator::validate_theme(&theme_id).await
}

#[tauri::command]
async fn get_applied_theme_id(
    sender: tauri::State<'_, CloseTaskSender>,
) -> DwallResult<Option<String>> {
    let sender = sender.clone();
    let sender = sender.lock().await;
    if sender.is_none() {
        return Ok(None);
    }

    let config = read_config_file().await?;

    Ok(config.theme_id())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() -> DwallResult<()> {
    setup_logging();
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
            close_last_theme_task,
            get_applied_theme_id
        ])
        .build(tauri::generate_context!())?
        .run(run_callback);
    Ok(())
}
