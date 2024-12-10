use std::borrow::Cow;
use std::path::PathBuf;
use std::sync::Arc;

use dwall::{config::Config, setup_logging, ThemeValidator};
use dwall::{ColorMode, APP_CONFIG_DIR};
use tauri::{AppHandle, Manager, RunEvent, WebviewWindow};
use tokio::sync::OnceCell;

use crate::auto_start::{check_auto_start, disable_auto_start, enable_auto_start};
use crate::download::download_theme_and_extract;
use crate::error::DwallSettingsResult;
use crate::postion::request_location_permission;
use crate::process_manager::{find_daemon_process, kill_daemon};
use crate::setup::setup_app;
use crate::theme::spawn_apply_daemon;
use crate::window::create_main_window;

mod auto_start;
mod download;
mod error;
mod postion;
mod process_manager;
mod setup;
mod theme;
mod window;

#[macro_use]
extern crate tracing;

pub static DAEMON_EXE_PATH: OnceCell<PathBuf> = OnceCell::const_new();

#[tauri::command]
fn show_window(app: AppHandle, label: &str) -> DwallSettingsResult<()> {
    debug!("Showing window: {}", label);

    if let Some(window) = app.get_webview_window(label) {
        trace!("Window found for label: {}", label);
        window.show().map_err(|e| {
            error!(error = %e, "Failed to show window");
            e
        })?;
        window.set_focus().map_err(|e| {
            error!(error = %e, "Failed to set window focus");
            e
        })?;
    } else {
        warn!("No window found with label: {}", label);
    }

    Ok(())
}

#[tauri::command]
async fn check_theme_exists(theme_id: &str) -> DwallSettingsResult<()> {
    trace!("Checking theme existence for theme_id: {}", theme_id);
    match ThemeValidator::validate_theme(theme_id).await {
        Ok(_) => {
            info!("Theme '{}' exists and is valid", theme_id);
            Ok(())
        }
        Err(e) => {
            error!(theme_id = %theme_id, error = %e, "Theme validation failed");
            Err(e.into())
        }
    }
}

#[tauri::command]
async fn get_applied_theme_id<'a>() -> DwallSettingsResult<Option<Cow<'a, str>>> {
    trace!("Attempting to get currently applied theme ID");

    let daemon_process = find_daemon_process()?;
    if daemon_process.is_none() {
        debug!("No daemon process found");
        return Ok(None);
    }

    match dwall::config::read_config_file().await {
        Ok(config) => {
            let theme_id = config.theme_id();
            info!(theme_id = ?theme_id, "Retrieved current theme ID");
            Ok(theme_id)
        }
        Err(e) => {
            error!(error = %e, "Failed to read config file while getting theme ID");
            Err(e.into())
        }
    }
}

#[tauri::command]
async fn read_config_file<'a>() -> DwallSettingsResult<Config<'a>> {
    trace!("Reading configuration file");
    match dwall::config::read_config_file().await {
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
async fn write_config_file(config: Config<'_>) -> DwallSettingsResult<()> {
    let config = Arc::new(config);

    trace!("Writing configuration file");
    match dwall::config::write_config_file(config.clone()).await {
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
async fn apply_theme(config: Config<'_>) -> DwallSettingsResult<()> {
    trace!("Starting theme application process");

    match kill_daemon() {
        Ok(_) => debug!("Existing daemon process killed"),
        Err(e) => warn!(error = %e, "Failed to kill existing daemon process"),
    }

    write_config_file(config).await?;

    match spawn_apply_daemon() {
        Ok(_) => {
            info!("Theme daemon spawned successfully");
            Ok(())
        }
        Err(e) => {
            error!(error = %e, "Failed to spawn theme daemon");
            Err(e)
        }
    }
}

#[tauri::command]
async fn open_config_dir() -> DwallSettingsResult<()> {
    open::that(APP_CONFIG_DIR.as_os_str()).map_err(|e| {
        error!("Failed to open app config directory: {}", e);
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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() -> DwallSettingsResult<()> {
    setup_logging("dwall_settings_lib");
    let builder = tauri::Builder::default()
        .plugin(tauri_plugin_process::init())
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
            check_theme_exists,
            apply_theme,
            get_applied_theme_id,
            check_auto_start,
            disable_auto_start,
            enable_auto_start,
            download_theme_and_extract,
            request_location_permission,
            open_config_dir,
            set_titlebar_color_mode,
            kill_daemon
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

    info!("DWall settings application run completed");
    Ok(())
}
