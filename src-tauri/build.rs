use std::env;

fn main() {
    let skip_build = env::var("DWALL_SKIP_BUILD").is_ok();

    if !skip_build {
        tauri_build::build();
    }
}
