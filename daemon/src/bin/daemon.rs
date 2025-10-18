#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use dwall::{core::daemon::DaemonApplication, setup_logging};

#[tokio::main(flavor = "current_thread")]
async fn main() -> dwall::DwallResult<()> {
    setup_logging(&[env!("CARGO_PKG_NAME").replace("-", "_")]);

    let mut app = DaemonApplication::new();
    app.run().await
}
