use std::borrow::Cow;

use download::download_theme_and_extract;
use dwall::config::Config;
use dwall::{setup_logging, ThemeValidator};
use tauri::{AppHandle, Manager};
use window::new_main_window;

use crate::auto_start::{check_auto_start, disable_auto_start, enable_auto_start};
use crate::error::DwallSettingsResult;
use crate::setup::setup_app;

mod auto_start;
mod download;
mod error;
mod setup;
#[cfg(not(debug_assertions))]
mod update;
mod window;

#[macro_use]
extern crate tracing;

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
    // TODO: 判断 dwall.exe 进程是否运行

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
async fn apply_theme<'a>() {
    // TODO: 创建 daemon 子进程
}

#[tauri::command]
async fn close_daemon<'a>() {
    // TODO: 关闭 daemon 子进程
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() -> DwallSettingsResult<()> {
    setup_logging(&env!("CARGO_PKG_NAME").replace("-", "_"));
    tauri::Builder::default()
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
            close_daemon,
            get_applied_theme_id,
            check_auto_start,
            disable_auto_start,
            enable_auto_start,
            download_theme_and_extract
        ])
        .run(tauri::generate_context!())?;
    Ok(())
}
