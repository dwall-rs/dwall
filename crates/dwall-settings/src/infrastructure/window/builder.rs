//! Window builder (pure technical)

use std::error::Error;

use tauri::{WebviewUrl, WebviewWindowBuilder};

/// Creates a window with the given parameters
pub fn build_window(
    app: &tauri::AppHandle,
    label: &str,
    title: &str,
    width: f64,
    height: f64,
) -> Result<(), Box<dyn Error>> {
    let window_builder = WebviewWindowBuilder::new(app, label, WebviewUrl::default())
        .title(title)
        .resizable(false)
        .maximizable(false)
        .transparent(true)
        .visible(cfg!(debug_assertions))
        .inner_size(width, height);

    match window_builder.build() {
        Ok(_) => Ok(()),
        Err(build_error) => Err(build_error.into()),
    }
}
