use std::io;

fn main() -> io::Result<()> {
    if cfg!(not(feature = "max-level-info")) {
        println!("cargo:rustc-env=DWALL_LOG=debug");
    };

    Ok(())
}
