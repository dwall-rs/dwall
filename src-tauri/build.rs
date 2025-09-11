use std::env;

fn main() {
    let is_clippy = env::var("CLIPPY_ARGS").is_ok()
        || env::args().any(|arg| arg.contains("clippy"))
        || env::var("CARGO_CFG_FEATURE")
            .map(|f| f.contains("clippy"))
            .unwrap_or(false);

    if !is_clippy {
        tauri_build::build()
    }
}
