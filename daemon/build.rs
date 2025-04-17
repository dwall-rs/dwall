use std::io;

fn main() -> io::Result<()> {
    #[cfg(feature = "build-script")]
    {
        use {std::env, winresource::WindowsResource};

        if cfg!(not(feature = "log-max-level-info")) {
            println!("cargo:rustc-env=DWALL_LOG=debug");
        };

        if env::var_os("CARGO_CFG_WINDOWS").is_some() {
            WindowsResource::new()
                .set_icon("../src-tauri/icons/icon.ico")
                .set(
                    "LegalCopyright",
                    "Copyright (C) 2025 thep0y. All rights reserved.",
                )
                .compile()?;
        }
    }
    Ok(())
}
