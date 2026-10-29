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
    ///
    /// `max_bytes` bounds what the file may end up holding. It is counted from the
    /// start of the transfer rather than from this response, so a resumed download
    /// cannot walk past it one attempt at a time, and a chunk that would cross it is
    /// refused before it is written.
    pub async fn download_file(
        &self,
        url: &str,
        target_path: &Path,
        downloaded_bytes: u64,
        max_bytes: u64,
        cancel_flag: Option<&Arc<AtomicBool>>,
        progress_callback: Option<Arc<dyn Fn(u64, u64) + Send + Sync>>,
    ) -> DwallSettingsResult<u64> {
        let mut request = self.current().get(url);
        if downloaded_bytes > 0 {
            request = request.header("Range", format!("bytes={downloaded_bytes}-"));
        }

        let mut response = request.send().await?;

        // A partial file can already hold the whole resource: the process may have
        // been killed after the transfer and before extraction. The server then
        // answers `416 Range Not Satisfiable`; retrying the same range forever is
        // worse than letting the extractor decide whether the bytes are a complete
        // archive. A genuinely corrupt partial is removed by the extraction path.
        if downloaded_bytes > 0 && response.status() == StatusCode::RANGE_NOT_SATISFIABLE {
            debug!(
                url = url,
                downloaded_bytes = downloaded_bytes,
                "The partial download already covers the resource"
            );
            return Ok(downloaded_bytes);
        }

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

        if already_written > max_bytes {
            return Err(DwallSettingsError::DownloadTooLarge { limit: max_bytes });
        }

        // A declared length over the bound is refused before the body is read, and
        // before any file is created: the server has already said how much it will
        // send, so there is nothing worth streaming for a download that cannot fit.
        // A resumed response declares only the bytes still to come, which is why the
        // partial file's length is added back in.
        if let Some(length) = response.content_length()
            && already_written.saturating_add(length) > max_bytes
        {
            return Err(DwallSettingsError::DownloadTooLarge { limit: max_bytes });
        }

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
                return Err(DwallSettingsError::Cancelled);
            }

            // Refuse before writing, so the file never holds more than the caller
            // allowed, not even for a moment.
            if written.saturating_add(chunk.len() as u64) > max_bytes {
                return Err(DwallSettingsError::DownloadTooLarge { limit: max_bytes });
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

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;
    use std::sync::Mutex;

    use super::*;

    use crate::network::testing::{TestServer, response, response_without_length};

    /// A limit none of these tests reaches; the bound has tests of its own below.
    const NO_LIMIT: u64 = u64::MAX;

    fn scratch(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("dwall_client_{name}"));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();

        dir
    }

    fn write(path: &std::path::Path, contents: &str) {
        fs::write(path, contents).unwrap();
    }

    fn read(path: &std::path::Path) -> String {
        fs::read_to_string(path).unwrap()
    }

    // ── Resuming ────────────────────────────────────────────────────────────

    #[test]
    fn a_partial_download_is_resumed_only_on_a_206_answer() {
        let server = TestServer::new(response("206 Partial Content", b"bbbb"));
        let target = scratch("resume").join("theme.zip");
        write(&target, "aaaa");

        let written = tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(HttpClient::new(None).unwrap().download_file(
                &server.url,
                &target,
                4,
                NO_LIMIT,
                None,
                None,
            ))
            .unwrap();

        assert_eq!(written, 8);
        assert_eq!(read(&target), "aaaabbbb");
        assert!(
            server
                .request_head()
                .to_lowercase()
                .contains("range: bytes=4-"),
            "a resuming client has to ask for the rest: {}",
            server.request_head()
        );
    }

    /// A mirror that ignores the range request and sends the whole file: appending it to
    /// the partial file would corrupt the archive, so the download has to start over.
    #[test]
    fn a_200_answer_truncates_the_partial_file() {
        let server = TestServer::new(response("200 OK", b"cc"));
        let target = scratch("truncate").join("theme.zip");
        write(&target, "aaaa");

        let written = tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(HttpClient::new(None).unwrap().download_file(
                &server.url,
                &target,
                4,
                NO_LIMIT,
                None,
                None,
            ))
            .unwrap();

        assert_eq!(written, 2);
        assert_eq!(read(&target), "cc");
    }

    /// A partial that already holds the whole resource makes the server answer
    /// 416. That is a completed transfer, not a reason to retry the same range.
    #[test]
    fn a_416_on_a_complete_partial_is_treated_as_complete() {
        let server = TestServer::new(response("416 Range Not Satisfiable", b""));
        let target = scratch("range_not_satisfiable").join("theme.zip");
        write(&target, "complete");

        let written = tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(HttpClient::new(None).unwrap().download_file(
                &server.url,
                &target,
                8,
                NO_LIMIT,
                None,
                None,
            ))
            .unwrap();

        assert_eq!(written, 8);
        assert_eq!(read(&target), "complete");
    }

    #[test]
    fn no_range_is_requested_without_a_partial_file() {
        let server = TestServer::new(response("200 OK", b"whole"));
        let target = scratch("fresh").join("theme.zip");

        let written = tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(HttpClient::new(None).unwrap().download_file(
                &server.url,
                &target,
                0,
                NO_LIMIT,
                None,
                None,
            ))
            .unwrap();

        assert_eq!(written, 5);
        assert!(
            !server.request_head().to_lowercase().contains("range:"),
            "nothing to resume, so nothing to ask for"
        );
    }

    // ── Cancelling and failing ──────────────────────────────────────────────

    /// Cancelling has to stop the transfer *between* chunks: the flag is checked before
    /// each write, so a cancelled download leaves no partial body behind.
    #[test]
    fn the_cancel_flag_stops_the_transfer() {
        let server = TestServer::new(response("200 OK", b"body that must not land"));
        let target = scratch("cancel").join("theme.zip");
        let cancelled = Arc::new(AtomicBool::new(true));

        let error = tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(HttpClient::new(None).unwrap().download_file(
                &server.url,
                &target,
                0,
                NO_LIMIT,
                Some(&cancelled),
                None,
            ))
            .unwrap_err();

        assert!(matches!(error, DwallSettingsError::Cancelled));
        assert_eq!(
            read(&target),
            "",
            "no chunk may be written after cancelling"
        );
    }

    #[test]
    fn an_error_status_is_reported_before_anything_is_created() {
        let server = TestServer::new(response("404 Not Found", b"no such asset"));
        let target = scratch("not_found").join("theme.zip");

        let result = tokio::runtime::Runtime::new().unwrap().block_on(
            HttpClient::new(None).unwrap().download_file(
                &server.url,
                &target,
                0,
                NO_LIMIT,
                None,
                None,
            ),
        );

        assert!(result.is_err());
        assert!(
            !target.exists(),
            "a failed request must not leave a file behind"
        );
    }

    // ── Progress ────────────────────────────────────────────────────────────

    #[test]
    fn progress_reports_the_total_when_the_length_is_known() {
        let server = TestServer::new(response("200 OK", b"0123456789"));
        let target = scratch("progress_known").join("theme.zip");
        let seen = Arc::new(Mutex::new(Vec::new()));
        let recorder = Arc::clone(&seen);

        tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(HttpClient::new(None).unwrap().download_file(
                &server.url,
                &target,
                0,
                NO_LIMIT,
                None,
                Some(Arc::new(move |written, total| {
                    recorder.lock().unwrap().push((written, total));
                })),
            ))
            .unwrap();

        let samples = seen.lock().unwrap().clone();
        assert!(!samples.is_empty());
        assert_eq!(*samples.last().unwrap(), (10, 10));
    }

    /// Without a `Content-Length` there is no total to show, and the old code reported
    /// the written bytes as if they were the total. The callback has to say "unknown".
    #[test]
    fn progress_reports_no_total_when_the_length_is_unknown() {
        let server = TestServer::new(response_without_length(b"0123456789"));
        let target = scratch("progress_unknown").join("theme.zip");
        let seen = Arc::new(Mutex::new(Vec::new()));
        let recorder = Arc::clone(&seen);

        tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(HttpClient::new(None).unwrap().download_file(
                &server.url,
                &target,
                0,
                NO_LIMIT,
                None,
                Some(Arc::new(move |written, total| {
                    recorder.lock().unwrap().push((written, total));
                })),
            ))
            .unwrap();

        assert_eq!(read(&target), "0123456789");
        assert_eq!(*seen.lock().unwrap().last().unwrap(), (10, 0));
    }

    // ── The size limit ──────────────────────────────────────────────────────

    #[test]
    fn a_body_exactly_at_the_limit_is_accepted() {
        let server = TestServer::new(response("200 OK", b"0123456789"));
        let target = scratch("at_limit").join("theme.zip");

        let written = tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(HttpClient::new(None).unwrap().download_file(
                &server.url,
                &target,
                0,
                10,
                None,
                None,
            ))
            .unwrap();

        assert_eq!(written, 10);
        assert_eq!(read(&target), "0123456789");
    }

    /// Without a declared length the bound still has to hold while the body is
    /// streamed: the check happens before the write, so an oversized body is refused
    /// whole rather than silently truncated at the boundary.
    #[test]
    fn a_streamed_body_over_the_limit_is_refused_and_not_written() {
        let server = TestServer::new(response_without_length(b"0123456789"));
        let target = scratch("too_large").join("theme.zip");

        let error = tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(HttpClient::new(None).unwrap().download_file(
                &server.url,
                &target,
                0,
                4,
                None,
                None,
            ))
            .unwrap_err();

        assert!(matches!(
            error,
            DwallSettingsError::DownloadTooLarge { limit: 4 }
        ));
        assert_eq!(read(&target), "", "nothing past the limit may be written");
    }

    /// A declared Content-Length over the bound is refused before the body is read,
    /// so a download that cannot fit never creates a file at all.
    #[test]
    fn a_declared_length_over_the_limit_is_refused_before_the_body() {
        let server = TestServer::new(response("200 OK", b"0123456789"));
        let target = scratch("declared_too_large").join("theme.zip");

        let error = tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(HttpClient::new(None).unwrap().download_file(
                &server.url,
                &target,
                0,
                4,
                None,
                None,
            ))
            .unwrap_err();

        assert!(matches!(
            error,
            DwallSettingsError::DownloadTooLarge { limit: 4 }
        ));
        assert!(
            !target.exists(),
            "a declared oversized download must create nothing"
        );
    }

    /// The bytes already on disk count, so a resumed transfer cannot walk past the
    /// limit one attempt at a time.
    #[test]
    fn a_resumed_download_counts_the_partial_file_against_the_limit() {
        let server = TestServer::new(response("206 Partial Content", b"bbbb"));
        let target = scratch("resume_limit").join("theme.zip");
        write(&target, "aaaa");

        let error = tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(HttpClient::new(None).unwrap().download_file(
                &server.url,
                &target,
                4,
                6,
                None,
                None,
            ))
            .unwrap_err();

        assert!(matches!(
            error,
            DwallSettingsError::DownloadTooLarge { limit: 6 }
        ));
        assert_eq!(read(&target), "aaaa", "the partial file is left as it was");
    }

    // ── Reading a small body ────────────────────────────────────────────────

    #[test]
    fn get_text_returns_the_body() {
        let server = TestServer::new(response("200 OK", b"{\"ok\":true}"));

        let body = tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(HttpClient::new(None).unwrap().get_text(&server.url, 1024))
            .unwrap();

        assert_eq!(body, "{\"ok\":true}");
    }

    #[test]
    fn get_text_rejects_a_declared_body_over_the_limit() {
        let server = TestServer::new(response("200 OK", b"0123456789"));

        let error = tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(HttpClient::new(None).unwrap().get_text(&server.url, 4))
            .unwrap_err();

        assert!(error.to_string().contains("byte limit"), "{error}");
    }

    /// The declared length can be absent — or a lie. The limit has to hold while the body
    /// is being read, not only when the server admits to its size.
    #[test]
    fn get_text_rejects_a_body_that_grows_past_the_limit() {
        let server = TestServer::new(response_without_length(b"0123456789"));

        let error = tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(HttpClient::new(None).unwrap().get_text(&server.url, 4))
            .unwrap_err();

        assert!(error.to_string().contains("byte limit"), "{error}");
    }

    #[test]
    fn get_text_rejects_a_body_that_is_not_utf8() {
        let server = TestServer::new(response("200 OK", &[0xff, 0xfe, 0xfd]));

        let error = tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(HttpClient::new(None).unwrap().get_text(&server.url, 1024))
            .unwrap_err();

        assert!(error.to_string().contains("UTF-8"), "{error}");
    }

    // ── Reconfiguring and the proxy branch ──────────────────────────────────

    fn socks5(host: &str) -> dwall::config::Network {
        dwall::config::Network::Socks5 {
            host: host.to_string(),
            port: 1080,
        }
    }

    #[test]
    fn a_socks5_configuration_builds_a_client() {
        let proxy = socks5("127.0.0.1");

        assert!(HttpClient::new(Some(&proxy)).is_ok());
    }

    #[test]
    fn reconfigure_installs_the_new_client() {
        let http = HttpClient::new(None).unwrap();
        let before = http.current();

        http.reconfigure(Some(&socks5("127.0.0.1"))).unwrap();

        assert!(
            !Arc::ptr_eq(&before, &http.current()),
            "a successful reconfigure has to install the client it built"
        );
    }

    /// The rebuild is the only thing that can fail; when it does, the request path
    /// must keep using the client that already works.
    #[test]
    fn reconfigure_keeps_the_previous_client_when_the_new_one_cannot_be_built() {
        let http = HttpClient::new(None).unwrap();
        let before = http.current();

        // A space cannot appear in a host, so the proxy URL cannot be built.
        assert!(http.reconfigure(Some(&socks5("not a host"))).is_err());
        assert!(
            Arc::ptr_eq(&before, &http.current()),
            "a failed reconfigure must not drop the working client"
        );
    }
}
