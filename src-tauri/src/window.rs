use std::error::Error;

use tauri::{WebviewUrl, WebviewWindowBuilder};

pub fn new_main_window(app: &tauri::AppHandle) -> Result<(), Box<dyn Error>> {
    let win_builder = WebviewWindowBuilder::new(app, "main", WebviewUrl::default())
        .title("Dwall")
        .transparent(true)
        .resizable(false)
        .maximizable(false)
        .visible(false)
        .inner_size(660., 600.);

    let window = win_builder.build().unwrap();

    #[cfg(target_os = "windows")]
    {
        info!("Applying acrylic effect on Windows");
        use window_vibrancy::apply_acrylic;
        apply_acrylic(&window, Some((18, 18, 18, 125)))?;
    }
    //window.set_focus().unwrap();
    //window.hide().unwrap();
    Ok(())
}
