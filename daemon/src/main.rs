#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[tokio::main]
async fn main() -> dwall::DwallResult<()> {
    dwall::setup_logging(&env!("CARGO_PKG_NAME").replace("-", "_"));

    let config = dwall::config::read_config_file().await?;
    dwall::apply_theme(config).await?;

    Ok(())
}
