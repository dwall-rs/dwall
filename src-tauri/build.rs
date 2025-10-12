use std::env;

fn main() {
    // Check if we're running in a special mode where we should skip tauri_build
    // CARGO_CFG_TEST is set when building in test mode
    // CLIPPY_ARGS is set when running clippy
    // CARGO_TARGET_TMPDIR is set during certain Cargo operations
    let is_special_mode = env::var("CARGO_CFG_TEST").is_ok()
        || env::var("CLIPPY_ARGS").is_ok()
        || env::var("CARGO_TARGET_TMPDIR").is_ok()
        || env::var("RUSTDOC").is_ok(); // Set during cargo doc

    if !is_special_mode {
        tauri_build::build();
    }
}
