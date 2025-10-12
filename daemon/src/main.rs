#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use dwall::{apply_theme, config::read_config_file, setup_logging};

#[tokio::main(flavor = "current_thread")]
async fn main() -> dwall::DwallResult<()> {
    setup_logging(&[env!("CARGO_PKG_NAME").replace("-", "_")]);

    let config = read_config_file().await?;
    apply_theme(config).await?;

    Ok(())
}
