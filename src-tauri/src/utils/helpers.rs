//! Helper functions
//!
//! This module contains various helper functions used throughout the application.

/// Creates directories if they don't exist
pub async fn create_dir_if_missing<P: AsRef<std::path::Path>>(path: P) -> std::io::Result<()> {
    let path = path.as_ref();
    if !path.exists() {
        tokio::fs::create_dir_all(path).await
    } else {
        Ok(())
    }
}
