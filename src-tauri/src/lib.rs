use std::borrow::Cow;
use std::path::PathBuf;
use std::process::Command;

use dwall::config::Config;
use dwall::{setup_logging, ThemeValidator};
use tauri::{AppHandle, Manager, RunEvent};
use tokio::sync::OnceCell;

use crate::auto_start::{check_auto_start, disable_auto_start, enable_auto_start};
use crate::download::download_theme_and_extract;
use crate::error::DwallSettingsResult;
use crate::process_manager::{find_daemon_process, kill_daemon};
use crate::setup::setup_app;
use crate::window::new_main_window;

mod auto_start;
mod download;
mod error;
mod process_manager;
mod setup;
#[cfg(not(debug_assertions))]
mod update;
mod window;

#[macro_use]
extern crate tracing;

pub static DAEMON_EXE_PATH: OnceCell<PathBuf> = OnceCell::const_new();

#[tauri::command]
fn show_window<'a>(app: AppHandle, label: &'a str) -> DwallSettingsResult<()> {
    debug!("Showing window: {}", label);

    if let Some(window) = app.get_webview_window(label) {
        window.show()?;
        window.set_focus()?;
    }

    Ok(())
}

#[tauri::command]
async fn check_theme_exists<'a>(theme_id: &'a str) -> DwallSettingsResult<()> {
    ThemeValidator::validate_theme(theme_id)
        .await
        .map_err(Into::into)
}

#[tauri::command]
async fn get_applied_theme_id<'a>() -> DwallSettingsResult<Option<Cow<'a, str>>> {
    if find_daemon_process()?.is_none() {
        return Ok(None);
    }

    let config = dwall::config::read_config_file().await?;

    Ok(config.theme_id())
}

#[tauri::command]
async fn read_config_file<'a>() -> DwallSettingsResult<Config<'a>> {
    dwall::config::read_config_file().await.map_err(Into::into)
}

#[tauri::command]
async fn write_config_file<'a>(config: Config<'a>) -> DwallSettingsResult<()> {
    dwall::config::write_config_file(config.into())
        .await
        .map_err(Into::into)
}

#[tauri::command]
async fn apply_theme(config: Config<'_>) -> DwallSettingsResult<()> {
    trace!("Spawning daemon...");
    kill_daemon()?;
    write_config_file(config).await?;

    let daemon_path = DAEMON_EXE_PATH.get().unwrap().to_str().unwrap();

    let handle = Command::new(daemon_path).spawn()?;
    info!(pid = handle.id(), "Spawned daemon using subprocess");

    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() -> DwallSettingsResult<()> {
    setup_logging("dwall_settings_lib");
    let builder = tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _argv, _cwd| {
            if let Some(w) = app.get_webview_window("main") {
                info!("Application instance already running, focusing existing window");
                w.set_focus().unwrap();
            } else {
                new_main_window(app).unwrap();
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
        ]);

    if cfg!(debug_assertions) {
        builder.build(tauri::generate_context!())?.run(|_, event| {
            if let RunEvent::Exit = event {
                kill_daemon().unwrap();
            }
        })
    } else {
        builder.run(tauri::generate_context!())?
    }
    Ok(())
}
