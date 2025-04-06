use std::borrow::Cow;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use dwall::config::write_config_file as dwall_write_config;
use dwall::{config::Config, setup_logging, ThemeValidator};
use dwall::{ColorMode, DWALL_CONFIG_DIR};
use tauri::{AppHandle, Manager, RunEvent, WebviewWindow};
use tokio::sync::OnceCell;

use crate::auto_start::{check_auto_start, disable_auto_start, enable_auto_start};
use crate::cache::get_or_save_cached_thumbnails;
use crate::download::{cancel_theme_download, download_theme_and_extract};
use crate::error::DwallSettingsResult;
use crate::fs::move_themes_directory;
use crate::i18n::get_translations;
use crate::postion::request_location_permission;
use crate::process_manager::{find_daemon_process, kill_daemon};
use crate::setup::setup_app;
use crate::theme::spawn_apply_daemon;
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
            error!(error = ?e, "Failed to show window");
            e
        })?;
        window.set_focus().map_err(|e| {
            error!(error = ?e, "Failed to set window focus");
            e
        })?;
    } else {
        warn!(label = label, "No window found with label");
    }

    Ok(())
}

#[tauri::command]
async fn check_theme_exists(themes_direcotry: &Path, theme_id: &str) -> DwallSettingsResult<()> {
    trace!(id = theme_id, "Checking theme existence for theme");
    match ThemeValidator::validate_theme(themes_direcotry, theme_id).await {
        Ok(_) => {
            info!(id = theme_id, "Theme exists and is valid");
            Ok(())
        }
        Err(e) => {
            error!(theme_id = %theme_id, error = ?e, "Theme validation failed");
            Err(e.into())
        }
    }
}

#[tauri::command]
async fn get_applied_theme_id(monitor_id: &str) -> DwallSettingsResult<Option<String>> {
    debug!(monitor_id, "Attempting to get currently applied theme ID");

    let daemon_process = find_daemon_process()?;
    if daemon_process.is_none() {
        debug!("No daemon process found");
        return Ok(None);
    }

    match dwall::config::read_config_file().await {
        Ok(config) => {
            let monitor_themes = config.monitor_specific_wallpapers();
            let theme_id = if monitor_id == "all" {
                let mut iter = monitor_themes.values();
                let first_value = iter.next();
                let theme_id = if iter.all(|value| Some(value) == first_value) {
                    first_value
                } else {
                    None
                };
                info!(theme_id = ?theme_id, "Retrieved all theme ID");
                theme_id
            } else {
                let theme_id = monitor_themes.get(monitor_id);
                info!(monitor_id, theme_id =?theme_id, "Retrieved current theme ID");
                theme_id
            };

            Ok(theme_id.map(|s| s.to_string()))
        }
        Err(e) => {
            error!(error = ?e, "Failed to read config file while getting theme ID");
            Err(e.into())
        }
    }
}

#[tauri::command]
async fn read_config_file() -> DwallSettingsResult<Config> {
    trace!("Reading configuration file");
    match dwall::config::read_config_file().await {
        Ok(config) => {
            debug!("Configuration file read successfully");
            Ok(config)
        }
        Err(e) => {
            error!(error = ?e, "Failed to read configuration file");
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
            error!(config = ?config, error = ?e, "Failed to write configuration file");
            Err(e.into())
        }
    }
}

#[tauri::command]
async fn apply_theme(config: Config) -> DwallSettingsResult<()> {
    trace!("Starting theme application process");

    match kill_daemon() {
        Ok(_) => debug!("Successfully killed existing daemon process"),
        Err(e) => warn!(error = ?e, "Failed to kill existing daemon process"),
    }

    dwall_write_config(&config).await?;

    match spawn_apply_daemon() {
        Ok(_) => {
            info!("Successfully spawned theme daemon");
            Ok(())
        }
        Err(e) => {
            error!(error = ?e, "Failed to spawn theme daemon");
            Err(e)
        }
    }
}

#[tauri::command]
async fn open_dir(dir_path: Cow<'_, Path>) -> DwallSettingsResult<()> {
    open::that(dir_path.as_os_str()).map_err(|e| {
        error!(error = ?e, "Failed to open app config directory");
        e.into()
    })
}

#[tauri::command]
async fn open_config_dir() -> DwallSettingsResult<()> {
    open::that(DWALL_CONFIG_DIR.as_os_str()).map_err(|e| {
        error!(error = ?e, "Failed to open app config directory");
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
async fn get_monitors() -> DwallSettingsResult<HashMap<String, dwall::monitor::Monitor>> {
    let monitors = monitor::get_monitors().await?;

    Ok(monitors)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() -> DwallSettingsResult<()> {
    setup_logging(&["dwall_settings_lib".to_string(), "dwall".to_string()]);
    let builder = tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_single_instance::init(|app, _argv, _cwd| {
            trace!("Handling single instance application launch");
            if let Some(w) = app.get_webview_window("main") {
                info!("Application instance already running, focusing existing window");
                if let Err(e) = w.set_focus() {
                    error!(error = ?e, "Failed to set focus on existing window");
                }
            } else {
                match create_main_window(app) {
                    Ok(_) => debug!("New main window created"),
                    Err(e) => error!(error = ?e, "Failed to create new main window"),
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
            cancel_theme_download,
            request_location_permission,
            open_dir,
            open_config_dir,
            set_titlebar_color_mode,
            move_themes_directory,
            kill_daemon,
            get_or_save_cached_thumbnails,
            get_translations,
            get_monitors,
        ]);

    if cfg!(debug_assertions) {
        builder.build(tauri::generate_context!())?.run(|_, event| {
            if let RunEvent::Exit = event {
                trace!("Application exit event received");
                match kill_daemon() {
                    Ok(_) => debug!("Daemon process killed on exit"),
                    Err(e) => error!(error = ?e, "Failed to kill daemon process on exit"),
                }
            }
        })
    } else {
        builder.run(tauri::generate_context!())?
    }

    info!("DWall settings application run completed");
    Ok(())
}
