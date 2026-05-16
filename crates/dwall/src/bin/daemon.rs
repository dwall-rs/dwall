#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use dwall::core::daemon::DaemonApplication;
use logging::Logger;

fn main() -> dwall::DwallResult<()> {
    let package_name = env!("CARGO_PKG_NAME");

    #[cfg(debug_assertions)]
    Logger::default().with_target(package_name).init()?;
    #[cfg(not(debug_assertions))]
    {
        use dwall::lazy::DWALL_LOG_DIR;

        Logger::default()
            .with_target(package_name)
            .with_file_path(DWALL_LOG_DIR.join(format!("{}.log", package_name)))?
            .init()?;
    }

    let mut app = DaemonApplication::new();
    app.run()
}
