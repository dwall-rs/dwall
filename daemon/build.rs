use std::io;

fn main() -> io::Result<()> {
    #[cfg(feature = "build-script")]
    {
        use {std::env, winresource::WindowsResource};

        if env::var_os("CARGO_CFG_WINDOWS").is_some() {
            WindowsResource::new()
                .set_icon("../src-tauri/icons/icon.ico")
                .compile()?;
        }
    }
    Ok(())
}
