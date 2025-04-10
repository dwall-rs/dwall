//! Theme download functionality
//!
//! This module provides functionality for downloading and extracting themes.

mod downloader;
mod error;
mod extractor;
mod file_manager;
mod http_service;
mod task_manager;

pub use downloader::ThemeDownloader;
pub use error::DownloadError;

// Re-export the public API
pub use self::downloader::{cancel_theme_download, download_theme_and_extract};
