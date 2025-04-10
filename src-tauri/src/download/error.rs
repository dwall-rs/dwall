//! Error types for the download module

use std::error::Error;

#[derive(Debug, thiserror::Error)]
pub enum DownloadError {
    #[error("{0}")]
    Connect(String),
    #[error("Download cancelled")]
    Cancelled,
    #[error("The theme does not exist on the server: {0}")]
    NotFound(String),
    #[error("Unhandled Error: {0}")]
    Unknown(String),
}

impl From<reqwest::Error> for DownloadError {
    fn from(value: reqwest::Error) -> Self {
        let source = value
            .source()
            .map(|e| format!("{:?}", e))
            .unwrap_or("".to_string());

        if value.is_connect() {
            return Self::Connect(source[43..source.len() - 1].to_string());
        }

        Self::Unknown(source)
    }
}
