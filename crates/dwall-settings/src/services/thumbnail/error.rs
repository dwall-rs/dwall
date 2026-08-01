//! Error types for the cache module

use std::path::PathBuf;

use thiserror::Error;

/// Errors that can occur during cache operations
#[derive(Debug, Error)]
pub enum ThumbnailError {
    /// Error occurred during HTTP request
    #[error(transparent)]
    Request(#[from] reqwest::Error),

    /// Error occurred during file system operations
    #[error(transparent)]
    Io(#[from] std::io::Error),

    /// Error occurred when downloading image
    #[error("Failed to download image after {retries} retries")]
    DownloadFailed {
        /// URL that failed to download
        url: String,
        /// Number of retries attempted
        retries: u32,
    },

    /// Error occurred when cache file expected but not found
    #[error("Cached file not found: {0}")]
    CachedFileNotFound(PathBuf),

    /// Other unspecified errors
    #[error("{0}")]
    Other(String),
}
