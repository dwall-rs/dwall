//! HTTP client with optional SOCKS5 proxy support.

use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, RwLock};
use std::time::Duration;

use reqwest::{Client, StatusCode};
use tokio::io::AsyncWriteExt;

use crate::error::{DwallSettingsError, DwallSettingsResult};

/// How long to wait for the TCP/TLS connection to be established.
const CONNECT_TIMEOUT: Duration = Duration::from_secs(15);

/// Maximum idle time between two body chunks.
///
/// A per-read timeout (instead of a total one) keeps large theme archives working
/// while still failing a stalled transfer instead of hanging forever.
const READ_TIMEOUT: Duration = Duration::from_secs(60);

/// Sent with every request, in the form GitHub asks for.
const USER_AGENT: &str = concat!("dwall/", env!("CARGO_PKG_VERSION"));

/// Generic HTTP downloader for files.
///
/// Provides pure technical download functionality without business logic.
///
/// The underlying `reqwest::Client` sits behind an `RwLock` so it can be rebuilt
/// when the user edits the network settings: otherwise a new proxy address only
/// took effect after restarting the window.
pub struct HttpClient {
    client: Arc<RwLock<Arc<Client>>>,
}

fn build_client(network: Option<&dwall::config::Network>) -> DwallSettingsResult<Client> {
    let builder = reqwest::ClientBuilder::new()
        .connect_timeout(CONNECT_TIMEOUT)
        .read_timeout(READ_TIMEOUT)
        // GitHub's API rejects requests without a `User-Agent`; sending one everywhere
        // keeps that from being a per-call-site detail.
        .user_agent(USER_AGENT);

    let builder = match network {
        Some(dwall::config::Network::Socks5 { host, port }) => {
            let proxy = reqwest::Proxy::all(format!("socks5h://{host}:{port}")).map_err(|e| {
                error!(error = ?e, "Failed to create SOCKS5 proxy");
                e
            })?;
            info!("Using SOCKS5 proxy: {host}:{port}");
            builder.proxy(proxy)
        }
        _ => builder,
    };

    builder.build().map_err(|e| {
        error!(error = %e, "Failed to create HTTP client");
        DwallSettingsError::from(e)
    })
}

impl HttpClient {
    /// Create a new HTTP client with network configuration
    pub fn new(network: Option<&dwall::config::Network>) -> DwallSettingsResult<Self> {
        Ok(Self {
            client: Arc::new(RwLock::new(Arc::new(build_client(network)?))),
        })
    }

    /// Rebuilds the client from the current network configuration.
    ///
    /// Called after the config is written so a changed proxy applies immediately.
    /// On failure the previous client is kept.
    pub fn reconfigure(&self, network: Option<&dwall::config::Network>) -> DwallSettingsResult<()> {
        let client = Arc::new(build_client(network)?);
        *self.client.write().unwrap_or_else(|e| e.into_inner()) = client;
        debug!("Reconfigured the HTTP client");
        Ok(())
    }

    fn current(&self) -> Arc<Client> {
        self.client
            .read()
            .unwrap_or_else(|e| e.into_inner())
            .clone()
    }

    /// Streams a URL into `target_path`, returning the number of bytes on disk.
    ///
    /// The body is written chunk by chunk — a theme archive can be hundreds of MB,
    /// so it is never buffered whole — the cancellation flag is checked between
    /// chunks (so cancelling actually stops the transfer), and a range request is
    /// only continued when the server really answered `206 Partial Content`.
    pub async fn download_file(
        &self,
        url: &str,
        target_path: &Path,
        downloaded_bytes: u64,
        cancel_flag: Option<&Arc<AtomicBool>>,
        progress_callback: Option<Arc<dyn Fn(u64, u64) + Send + Sync>>,
    ) -> DwallSettingsResult<u64> {
        let mut request = self.current().get(url);
        if downloaded_bytes > 0 {
            request = request.header("Range", format!("bytes={downloaded_bytes}-"));
        }

        let mut response = request.send().await?;
        response.error_for_status_ref()?;

        let resume = downloaded_bytes > 0 && response.status() == StatusCode::PARTIAL_CONTENT;

        if downloaded_bytes > 0 && !resume {
            // The server (or a mirror) ignored the range request; appending the
            // whole body to the partial file would corrupt it, so start over.
            debug!(
                url = url,
                "Range request was not honoured, restarting the download"
            );
        }

        let already_written = if resume { downloaded_bytes } else { 0 };
        let total_size = already_written + response.content_length().unwrap_or(0);

        let mut file = if resume {
            tokio::fs::OpenOptions::new()
                .write(true)
                .append(true)
                .open(target_path)
                .await?
        } else {
            tokio::fs::File::create(target_path).await?
        };

        let mut written = already_written;

        while let Some(chunk) = response.chunk().await? {
            if let Some(cancel) = cancel_flag
                && cancel.load(Ordering::Relaxed)
            {
                return Err(DwallSettingsError::Other("Download cancelled".to_string()));
            }

            file.write_all(&chunk).await?;
            written += chunk.len() as u64;

            if let Some(callback) = &progress_callback {
                callback(written, total_size);
            }
        }

        file.flush().await?;

        Ok(written)
    }

    /// Fetches a small text response — an API answer, not a payload.
    ///
    /// Reads incrementally and gives up past `max_bytes`, so a mirror or a wrong URL
    /// cannot make the process buffer something unbounded. Uses the same client as
    /// every other request, so the configured proxy applies.
    pub async fn get_text(&self, url: &str, max_bytes: usize) -> DwallSettingsResult<String> {
        let mut response = self.current().get(url).send().await?;
        response.error_for_status_ref()?;

        if let Some(length) = response.content_length()
            && length > max_bytes as u64
        {
            return Err(DwallSettingsError::Other(format!(
                "{url} answered with {length} bytes, over the {max_bytes} byte limit"
            )));
        }

        let mut body = Vec::new();

        while let Some(chunk) = response.chunk().await? {
            if body.len() + chunk.len() > max_bytes {
                return Err(DwallSettingsError::Other(format!(
                    "{url} exceeded the {max_bytes} byte limit"
                )));
            }

            body.extend_from_slice(&chunk);
        }

        String::from_utf8(body).map_err(|e| {
            error!(error = %e, "Response was not valid UTF-8");
            DwallSettingsError::Other(format!("{url} did not answer with UTF-8: {e}"))
        })
    }
}

impl Clone for HttpClient {
    /// Clones the shared handle, not the client: clones stay in sync when the
    /// configuration is applied.
    fn clone(&self) -> Self {
        Self {
            client: self.client.clone(),
        }
    }
}
