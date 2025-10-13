#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use dwall::core::daemon::DaemonApplication;

#[tokio::main(flavor = "current_thread")]
async fn main() -> dwall::DwallResult<()> {
    let mut app = DaemonApplication::new().await?;
    app.run().await
}