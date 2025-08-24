//! Windows API helper module

use crate::error::{DwallError, DwallResult};

/// Helper function to handle Windows API errors with consistent logging
///
/// This function maps a Windows error to a DwallError and logs the error message
pub(super) async fn handle_windows_error<T, F>(operation: &str, f: F) -> DwallResult<T>
where
    F: FnOnce() -> windows::core::Result<T>,
{
    trace!("{}", operation);
    match f() {
        Ok(result) => {
            debug!("{} completed successfully", operation);
            Ok(result)
        }
        Err(e) => {
            error!(error = %e, "{} failed", operation);
            Err(DwallError::Windows(e))
        }
    }
}
