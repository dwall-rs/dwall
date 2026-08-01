//! Pure HTTP client (technical implementation, no business knowledge)

use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

use reqwest::Client;
use tokio::io::AsyncWriteExt;

use crate::error::DwallSettingsResult;

/// Generic HTTP downloader for files
///
/// Provides pure technical download functionality without business logic.
pub struct HttpClient {
    client: Arc<Client>,
}

impl HttpClient {
    /// Create a new HTTP client with network configuration
    pub fn new(network: Option<&dwall::config::Network>) -> DwallSettingsResult<Self> {
        let builder = reqwest::ClientBuilder::new().connect_timeout(Duration::from_secs(120));

        let builder = match network {
            Some(dwall::config::Network::Socks5 { host, port }) => {
                let proxy =
                    reqwest::Proxy::all(format!("socks5h://{host}:{port}")).map_err(|e| {
                        error!(error = ?e, "Failed to create SOCKS5 proxy");
                        e
                    })?;
                info!("Using SOCKS5 proxy: {host}:{port}");
                builder.proxy(proxy)
            }
            _ => builder,
        };

        let client = builder.build().map_err(|e| {
            error!(error = %e, "Failed to create HTTP client");
            e
        })?;

        Ok(Self {
            client: Arc::new(client),
        })
    }

    /// Download a file from URL to local path
    pub async fn download_file(
        &self,
        url: &str,
        target_path: &Path,
        downloaded_bytes: u64,
        cancel_flag: Option<&Arc<std::sync::atomic::AtomicBool>>,
        progress_callback: Option<Arc<dyn Fn(u64, u64) + Send + Sync>>,
    ) -> DwallSettingsResult<u64> {
        let mut file = if downloaded_bytes > 0 {
            tokio::fs::OpenOptions::new()
                .write(true)
                .append(true)
                .open(target_path)
                .await?
        } else {
            tokio::fs::File::create(target_path).await?
        };

        let mut request = self.client.get(url);
        if downloaded_bytes > 0 {
            request = request.header("Range", format!("bytes={downloaded_bytes}-"));
        }

        let response = request.send().await?;
        response.error_for_status_ref()?;

        let bytes = response.bytes().await?;
        let total_size = bytes.len() as u64;

        if let Some(cancel) = cancel_flag
            && cancel.load(std::sync::atomic::Ordering::Relaxed)
        {
            return Err(crate::error::DwallSettingsError::Other(
                "Download cancelled".to_string(),
            ));
        }

        file.write_all(&bytes).await?;

        if let Some(callback) = progress_callback {
            callback(total_size, total_size);
        }

        Ok(total_size)
    }

    pub fn arc(&self) -> Arc<Client> {
        self.client.clone()
    }
}

impl Clone for HttpClient {
    fn clone(&self) -> Self {
        Self { client: self.arc() }
    }
}
