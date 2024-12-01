use std::error::Error;

use tauri::{WebviewUrl, WebviewWindowBuilder};
use window_vibrancy::apply_acrylic;

pub fn new_main_window(app: &tauri::AppHandle) -> Result<(), Box<dyn Error>> {
    let win_builder = WebviewWindowBuilder::new(app, "main", WebviewUrl::default())
        .title("Dwall Settings")
        .transparent(true)
        .resizable(false)
        .maximizable(false)
        .visible(false)
        .inner_size(660., 600.);

    let window = win_builder.build().unwrap();

    info!("Applying acrylic effect on Windows");
    apply_acrylic(&window, Some((18, 18, 18, 125))).map_err(|e| {
        error!("Failed to apply acrylic: {}", e);
        e
    })?;

    Ok(())
}
