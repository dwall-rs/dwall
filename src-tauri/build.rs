use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();

    let is_special_mode = args
        .iter()
        .any(|arg| arg == "clippy" || arg == "test" || arg == "doc")
        || env::var("CLIPPY_ARGS").is_ok()
        || env::var("CARGO_TARGET_TMPDIR").is_ok();

    if !is_special_mode {
        tauri_build::build();
    }
}
