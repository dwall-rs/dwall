// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::path::PathBuf;

use tauri::{Manager, RunEvent};
use tokio::sync::OnceCell;

use crate::error::DwallSettingsResult;
use crate::process::stop_daemon;
use crate::setup::setup_app;
use crate::window::build_window;

mod archive;
mod autostart;
mod commands;
mod daemon;
mod error;
mod filesystem;
mod monitor_wallpapers;
mod network;
mod process;
mod setup;
#[cfg(test)]
mod testing;
mod theme;
mod thumbnail;
mod update;
mod window;

#[macro_use]
extern crate logging;

pub static DAEMON_EXE_PATH: OnceCell<PathBuf> = OnceCell::const_new();

/// Starts the settings window.
///
/// Deliberately not `#[tokio::main]`: nothing in this function awaits, and Tauri creates
/// the tokio runtime its async commands run on the first time it needs one, so the
/// attribute only added a second runtime.
///
/// Nothing reached from the main thread may call `tokio::spawn`, `tokio::task::spawn_blocking`
/// or anything else that needs a runtime to be current — without the attribute there is
/// none, and such a call panics instead of failing. Use `tauri::async_runtime` there.
fn main() -> DwallSettingsResult<()> {
    let builder = tauri::Builder::default()
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_single_instance::init(|app, _argv, _cwd| {
            if let Some(w) = app.get_webview_window("main") {
                info!("Application instance already running, focusing existing window");
                if let Err(e) = w.set_focus() {
                    error!(error = %e, "Failed to set focus on existing window");
                }
            } else if let Err(e) = build_window(app, "main", "Dwall Settings") {
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
            commands::window::open_url,
            commands::window::open_config_dir,
            commands::window::open_log_dir,
            commands::window::set_titlebar_color_mode,
            commands::system::move_directory_cmd,
            commands::thumbnail::get_or_save_cached_thumbnails_cmd,
            commands::thumbnail::clear_thumbnail_cache_cmd,
            commands::monitor::get_monitors_cmd,
            commands::monitor::resolve_monitor_themes_cmd,
            commands::system::open_privacy_location_settings,
            commands::system::check_for_updates_cmd,
            commands::theme::get_customized_themes_cmd,
            commands::solar::get_solar_position,
            commands::solar::get_solar_path,
            commands::app::get_app_info,
            commands::engine::get_engine_status,
            commands::engine::get_daemon_last_error,
            commands::engine::start_engine,
            commands::engine::stop_engine,
            commands::mirror::mirror_url,
            commands::random::get_random_selection,
            commands::catalog::get_theme_catalog,
            commands::catalog::get_theme_wallpapers,
            commands::catalog::match_wallpaper,
        ]);

    if cfg!(debug_assertions) {
        builder.build(tauri::generate_context!())?.run(|_, event| {
            if let RunEvent::Exit = event {
                match stop_daemon() {
                    Ok(Some(pid)) => debug!(pid = pid, "Daemon process killed on exit"),
                    Ok(None) => debug!("No daemon process to kill on exit"),
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
