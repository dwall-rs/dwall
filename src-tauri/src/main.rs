// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::borrow::Cow;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use dwall::DisplayMonitor;
use dwall::{
    config::Config, read_config_file as dwall_read_config, setup_logging,
    write_config_file as dwall_write_config, ColorMode, DWALL_CONFIG_DIR,
};
use tauri::{AppHandle, Manager, RunEvent, WebviewWindow};
use tokio::sync::OnceCell;

use crate::auto_start::{check_auto_start, disable_auto_start, enable_auto_start};
use crate::cache::{clear_thumbnail_cache, get_or_save_cached_thumbnails};
use crate::download::{cancel_theme_download, download_theme_and_extract};
use crate::error::DwallSettingsResult;
use crate::fs::move_themes_directory;
use crate::i18n::get_translations;
use crate::postion::request_location_permission;
use crate::process_manager::kill_daemon;
use crate::setup::setup_app;
use crate::theme::{apply_theme, get_applied_theme_id, validate_theme};
use crate::window::create_main_window;

mod auto_start;
mod cache;
mod download;
mod error;
mod fs;
mod i18n;
mod monitor;
mod postion;
mod process_manager;
mod setup;
mod theme;
mod tracker;
mod window;

#[macro_use]
extern crate tracing;

pub static DAEMON_EXE_PATH: OnceCell<PathBuf> = OnceCell::const_new();

#[tauri::command]
fn show_window(app: AppHandle, label: &str) -> DwallSettingsResult<()> {
    debug!(label = label, "Showing window");

    if let Some(window) = app.get_webview_window(label) {
        trace!(label = label, "Window found for label");
        window.show().map_err(|e| {
            error!(error = %e, "Failed to show window");
            e
        })?;
        window.set_focus().map_err(|e| {
            error!(error = %e, "Failed to set window focus");
            e
        })?;
    } else {
        warn!(label = label, "No window found with label");
    }

    Ok(())
}

#[tauri::command]
async fn read_config_file() -> DwallSettingsResult<Config> {
    trace!("Reading configuration file");
    match dwall_read_config().await {
        Ok(config) => {
            debug!("Configuration file read successfully");
            Ok(config)
        }
        Err(e) => {
            error!(error = %e, "Failed to read configuration file");
            Err(e.into())
        }
    }
}

#[tauri::command]
async fn write_config_file(config: Config) -> DwallSettingsResult<()> {
    trace!("Writing configuration file");
    match dwall_write_config(&config).await {
        Ok(_) => {
            info!(config = ?config, "Configuration file written successfully");
            Ok(())
        }
        Err(e) => {
            error!(config = ?config, error = %e, "Failed to write configuration file");
            Err(e.into())
        }
    }
}

#[tauri::command]
async fn open_dir(dir_path: Cow<'_, Path>) -> DwallSettingsResult<()> {
    open::that(dir_path.as_os_str()).map_err(|e| {
        error!(error = %e, "Failed to open app config directory");
        e.into()
    })
}

#[tauri::command]
async fn open_config_dir() -> DwallSettingsResult<()> {
    open::that(DWALL_CONFIG_DIR.as_os_str()).map_err(|e| {
        error!(error = %e, "Failed to open app config directory");
        e.into()
    })
}

#[tauri::command]
async fn set_titlebar_color_mode(
    window: WebviewWindow,
    color_mode: ColorMode,
) -> DwallSettingsResult<()> {
    let hwnd = window.hwnd()?;

    crate::window::set_window_color_mode(hwnd, color_mode)?;
    Ok(())
}

#[tauri::command]
async fn get_monitors() -> DwallSettingsResult<HashMap<String, DisplayMonitor>> {
    let monitors = monitor::get_monitors().await?;

    Ok(monitors)
}

#[tauri::command]
async fn open_privacy_location_settings() -> DwallSettingsResult<()> {
    open::that("ms-settings:privacy-location").map_err(|e| e.into())
}

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
            trace!("Handling single instance application launch");
            if let Some(w) = app.get_webview_window("main") {
                info!("Application instance already running, focusing existing window");
                if let Err(e) = w.set_focus() {
                    error!(error = %e, "Failed to set focus on existing window");
                }
            } else {
                match create_main_window(app) {
                    Ok(_) => debug!("New main window created"),
                    Err(e) => error!(error = %e, "Failed to create new main window"),
                }
            }
        }))
        .setup(setup_app)
        .invoke_handler(tauri::generate_handler![
            show_window,
            read_config_file,
            write_config_file,
            validate_theme,
            apply_theme,
            get_applied_theme_id,
            check_auto_start,
            disable_auto_start,
            enable_auto_start,
            download_theme_and_extract,
            cancel_theme_download,
            request_location_permission,
            open_dir,
            open_config_dir,
            set_titlebar_color_mode,
            move_themes_directory,
            kill_daemon,
            get_or_save_cached_thumbnails,
            clear_thumbnail_cache,
            get_translations,
            get_monitors,
            open_privacy_location_settings
        ]);

    if cfg!(debug_assertions) {
        builder.build(tauri::generate_context!())?.run(|_, event| {
            if let RunEvent::Exit = event {
                trace!("Application exit event received");
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
