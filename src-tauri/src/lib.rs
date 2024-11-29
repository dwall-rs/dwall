use std::borrow::Cow;

use download::download_theme_and_extract;
use tauri::{AppHandle, Manager};
use window::new_main_window;

use crate::auto_start::{check_auto_start, disable_auto_start, enable_auto_start};
use crate::config::{read_config_file, write_config_file};
use crate::error::DwallResult;
use crate::event::run_callback;
use crate::setup::{setup_app, setup_logging};
use crate::theme::{apply_theme, close_last_theme_task, CloseTaskSender, ThemeValidator};

mod auto_start;
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
#[cfg(not(debug_assertions))]
mod update;
mod window;

#[macro_use]
extern crate tracing;

#[tauri::command]
fn show_window<'a>(app: AppHandle, label: &'a str) -> DwallResult<()> {
    debug!("Showing window: {}", label);

    if let Some(window) = app.get_webview_window(label) {
        window.show()?;
        window.set_focus()?;
    }

    Ok(())
}

#[tauri::command]
async fn check_theme_exists<'a>(theme_id: &'a str) -> DwallResult<()> {
    ThemeValidator::validate_theme(theme_id).await
}

#[tauri::command]
async fn get_applied_theme_id(
    sender: tauri::State<'_, CloseTaskSender>,
) -> DwallResult<Option<Cow<'_, str>>> {
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
            } else {
                new_main_window(app).unwrap();
            }
        }))
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(setup_app)
        .invoke_handler(tauri::generate_handler![
            show_window,
            read_config_file,
            write_config_file,
            check_theme_exists,
            apply_theme,
            close_last_theme_task,
            get_applied_theme_id,
            check_auto_start,
            disable_auto_start,
            enable_auto_start,
            download_theme_and_extract
        ])
        .build(tauri::generate_context!())?
        .run(run_callback);
    Ok(())
}
