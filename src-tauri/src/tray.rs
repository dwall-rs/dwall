use tauri::{
    menu::{Menu, MenuItem},
    tray::{TrayIconBuilder, TrayIconEvent},
    Manager,
};

use crate::{error::DwallResult, window::new_main_window};

pub fn build_tray(app: &mut tauri::App) -> DwallResult<()> {
    let quit_i = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&quit_i])?;

    let tray = TrayIconBuilder::new()
        .icon(app.default_window_icon().unwrap().clone())
        .menu(&menu)
        .menu_on_left_click(true)
        .on_menu_event(|app, event| match event.id.as_ref() {
            "quit" => {
                println!("quit menu item was clicked");
                std::process::exit(0);
            }
            _ => {
                println!("menu item {:?} not handled", event.id);
            }
        })
        .on_tray_icon_event(|tray, event| match event {
            //TrayIconEvent::Click {
            //    button: MouseButton::Left,
            //    button_state: MouseButtonState::Up,
            //    ..
            //} => {
            //    println!("left click pressed and released");
            //    // in this example, let's show and focus the main window when the tray is clicked
            //    let app = tray.app_handle();
            //    if let Some(window) = app.get_webview_window("main") {
            //        let _ = window.show();
            //        let _ = window.set_focus();
            //    }
            //}
            TrayIconEvent::DoubleClick { .. } => {
                println!("double click");
                let app = tray.app_handle();
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.show();
                    let _ = window.set_focus();
                } else {
                    new_main_window(app).unwrap();
                }
            }
            _ => {}
        })
        .tooltip(&app.package_info().name)
        .build(app)?;
    Ok(())
}
