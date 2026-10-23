//! Theme download orchestration.

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::time::Duration;

use tauri::Runtime;
use tokio::fs;
use tokio::time::sleep;

use crate::archive;
use crate::error::{DwallSettingsError, DwallSettingsResult};
use crate::filesystem::create_dir_if_missing;
use crate::network::HttpClient;
use crate::network::MirrorUrlResolver;
use crate::network::progress::{OwnedProgressEmitter, ProgressEmitter};
use crate::network::tasks::DownloadTaskManager;
use crate::theme::download_url::ThemeDownloadUrlBuilder;
use crate::theme::paths::ThemePathResolver;

/// Removes the download task from the tracker when dropped, so no early return can
/// leave the theme permanently marked as "already being processed".
struct DownloadTaskGuard<'a> {
    tracker: &'a DownloadTaskManager,
    theme_id: &'a str,
}

impl Drop for DownloadTaskGuard<'_> {
    fn drop(&mut self) {
        self.tracker.remove_task(self.theme_id);
    }
}

/// Orchestrates the theme download use case
pub struct ThemeDownloader {
    task_tracker: DownloadTaskManager,
    http_client: HttpClient,
}

impl ThemeDownloader {
    pub fn new(http_client: HttpClient) -> Self {
        Self {
            task_tracker: DownloadTaskManager::new(),
            http_client,
        }
    }

    /// Download and extract a theme
    ///
    /// The archive is extracted into a staging directory and only swapped in once
    /// it succeeded, so a failed download cannot destroy a working installation.
    pub async fn download_and_extract<R: Runtime>(
        &self,
        config: &dwall::Config,
        theme_id: &str,
        progress_emitter: Option<&ProgressEmitter<'_, R>>,
    ) -> DwallSettingsResult<PathBuf> {
        let cancel_flag = self.task_tracker.add_task(theme_id)?;
        let _task_guard = DownloadTaskGuard {
            tracker: &self.task_tracker,
            theme_id,
        };

        let github_url = ThemeDownloadUrlBuilder::build(theme_id);
        let asset_url = MirrorUrlResolver::resolve(config.network(), &github_url).await;

        let theme_dir = ThemePathResolver::theme_dir(config, theme_id);
        let staging_dir = ThemePathResolver::staging_dir(config, theme_id);
        let zip_path = ThemePathResolver::zip_path(config, theme_id);
        let temp_path = ThemePathResolver::temp_zip_path(config, theme_id);

        // Always start from a clean staging directory; the installed theme (if any)
        // is left untouched until the swap below.
        if staging_dir.exists() {
            fs::remove_dir_all(&staging_dir).await?;
        }
        create_dir_if_missing(&staging_dir).await?;

        let download_result = self
            .download_with_retry(
                &asset_url,
                &temp_path,
                &cancel_flag,
                progress_emitter,
                theme_id,
            )
            .await;

        if let Err(e) = download_result {
            cleanup_dir(&staging_dir).await;
            self.handle_download_error(&temp_path).await;
            return Err(e);
        }

        let extract_result = stage_archive(&temp_path, &zip_path, &staging_dir).await;

        if let Err(e) = extract_result {
            cleanup_dir(&staging_dir).await;
            return Err(e);
        }

        // Extraction succeeding only means the archive unpacked. `archive::extract_zip`
        // strips a single shared top-level directory and nothing else, so an archive
        // carrying one stray top-level file leaves the theme one level down; and an
        // archive can be complete enough to unpack while missing a frame image. Both
        // are refused here, before the swap below deletes the installed copy.
        if let Err(e) = verify_staged_theme(&staging_dir, config) {
            error!(
                theme_id = theme_id,
                staging_dir = %staging_dir.display(),
                error = %e,
                "Downloaded archive is not a theme the daemon can apply, keeping the installed theme"
            );
            cleanup_dir(&staging_dir).await;
            return Err(e);
        }

        // Swap: drop the previous installation only now that a complete replacement
        // exists on disk.
        if theme_dir.exists()
            && let Err(e) = fs::remove_dir_all(&theme_dir).await
        {
            cleanup_dir(&staging_dir).await;
            return Err(e.into());
        }

        fs::rename(&staging_dir, &theme_dir).await.map_err(|e| {
            error!(
                theme_dir = %theme_dir.display(),
                error = %e,
                "Failed to move the extracted theme into place"
            );
            DwallSettingsError::from(e)
        })?;

        Ok(theme_dir)
    }

    /// Cancel a theme download
    pub async fn cancel(&self, theme_id: &str) {
        self.task_tracker.cancel_task(theme_id);
    }

    /// Applies a changed network configuration to the HTTP client.
    pub fn reconfigure(&self, network: Option<&dwall::config::Network>) -> DwallSettingsResult<()> {
        self.http_client.reconfigure(network)
    }

    async fn download_with_retry(
        &self,
        url: &str,
        temp_path: &std::path::Path,
        cancel_flag: &Arc<AtomicBool>,
        progress_emitter: Option<&ProgressEmitter<'_, impl Runtime>>,
        task_id: &str,
    ) -> DwallSettingsResult<()> {
        const MAX_RETRIES: u32 = 3;

        let progress_callback = progress_emitter
            .map(|emitter| OwnedProgressEmitter::new(emitter.window()).throttled_callback(task_id));

        for attempt in 0..MAX_RETRIES {
            if attempt > 0 {
                warn!(attempt = attempt, "Retrying download");
                sleep(Duration::from_secs(1)).await;
            }

            let downloaded_bytes = if temp_path.exists() {
                fs::metadata(temp_path).await?.len()
            } else {
                0
            };

            match self
                .http_client
                .download_file(
                    url,
                    temp_path,
                    downloaded_bytes,
                    MAX_THEME_ARCHIVE_BYTES,
                    Some(cancel_flag),
                    progress_callback.clone(),
                )
                .await
            {
                Ok(_) => return Ok(()),
                // A cancelled download is the user's decision, not a failure to
                // retry: the cancellation flag is still set, so the next attempt
                // would abort on its first chunk and the wait would be pointless.
                Err(e) if !is_retryable(&e) => return Err(e),
                Err(e) if attempt < MAX_RETRIES - 1 => {
                    warn!(error = %e, "Download failed, will retry");
                }
                Err(e) => return Err(e),
            }
        }

        Err(DwallSettingsError::Other(
            "Download failed after retries".to_string(),
        ))
    }

    async fn handle_download_error(&self, temp_path: &std::path::Path) {
        // Keep a non-empty partial file: the next attempt resumes it with a range
        // request. Only a zero-byte leftover is useless.
        if let Ok(metadata) = fs::metadata(temp_path).await
            && metadata.len() == 0
        {
            let _ = fs::remove_file(temp_path).await;
        }
    }
}

/// The most the archive on disk may hold.
///
/// The same bound the extractor enforces: a compressed archive is never larger than
/// what it expands to, so this is the largest download that could still be extracted,
/// and a server that streams forever stops here instead of filling the disk first.
const MAX_THEME_ARCHIVE_BYTES: u64 = archive::MAX_EXTRACTED_BYTES;

/// Whether a failed attempt is worth trying again.
///
/// The cancellation message is part of the renderer contract
/// (`isCancellation` in `src/models/download.ts` matches this text), so it is
/// pinned by a test rather than left to whoever edits the variant next. An oversized
/// download is refused the same way: the server will answer with the same bytes
/// again, so a retry only writes them again.
fn is_retryable(error: &DwallSettingsError) -> bool {
    !matches!(
        error,
        DwallSettingsError::Cancelled | DwallSettingsError::DownloadTooLarge { .. }
    )
}

/// Moves the finished download into place, extracts it, and clears the archive.
///
/// The archive is removed on both paths: it used to be the last statement of the
/// extraction chain, so a failed extraction left `themes/<id>.zip` behind — the
/// largest file the app writes, in a directory nothing else cleans.
async fn stage_archive(
    temp_path: &std::path::Path,
    zip_path: &std::path::Path,
    staging_dir: &std::path::Path,
) -> DwallSettingsResult<()> {
    let extraction = async {
        fs::rename(temp_path, zip_path).await?;
        archive::extract_zip(zip_path, staging_dir).await
    }
    .await;

    if let Err(e) = archive::remove_file(zip_path).await {
        warn!(
            zip_path = %zip_path.display(),
            error = %e,
            "Failed to remove the downloaded archive"
        );
    }

    extraction
}

/// Best-effort removal of a staging directory.
async fn cleanup_dir(dir: &std::path::Path) {
    if dir.exists()
        && let Err(e) = fs::remove_dir_all(dir).await
    {
        warn!(dir = %dir.display(), error = %e, "Failed to clean up the staging directory");
    }
}

/// Whether a staging directory holds a theme the daemon can actually apply.
///
/// Runs the daemon's own [`dwall::ThemeValidator`] against the staging directory,
/// so "this download is accepted" and "the daemon can apply it" are the same
/// question rather than two answers that can drift apart. The validator is
/// addressed the way it expects — a themes directory plus a theme id — which the
/// staging path already is (`<themes_directory>/<id>.staging`).
fn verify_staged_theme(staging_dir: &Path, config: &dwall::Config) -> DwallSettingsResult<()> {
    let Some(themes_directory) = staging_dir.parent() else {
        return Err(DwallSettingsError::Other(format!(
            "Staging directory has no parent: {}",
            staging_dir.display()
        )));
    };

    let Some(theme_identifier) = staging_dir.file_name().and_then(|name| name.to_str()) else {
        return Err(DwallSettingsError::Other(format!(
            "Staging directory has no usable name: {}",
            staging_dir.display()
        )));
    };

    // `customized = false`: a download always lands in `themes_directory`, whose
    // layout the daemon reads as `<theme>/<image_format>/<n>.<ext>`.
    //
    // The uncached entry point matters here: `<id>.staging` is reused by every
    // download of one theme, so the daemon's path-keyed parse cache would hand back
    // the previous download's frames and image format.
    dwall::ThemeValidator::validate_uncached(
        themes_directory,
        theme_identifier,
        false,
        config.image_format(),
    )?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;

    use super::*;

    #[test]
    fn an_oversized_download_is_not_retried() {
        assert!(!is_retryable(&DwallSettingsError::DownloadTooLarge {
            limit: 1
        }));
        assert!(!is_retryable(&DwallSettingsError::Cancelled));
        assert!(is_retryable(&DwallSettingsError::Other(
            "connection reset".to_string()
        )));
    }

    /// An empty scratch themes directory of its own, so the tests can run in
    /// parallel without sharing a staging path.
    fn scratch_themes_directory(name: &str) -> PathBuf {
        let root = std::env::temp_dir().join(format!("dwall_downloader_{name}"));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();

        root
    }

    fn config_for(themes_directory: &Path) -> dwall::Config {
        dwall::Config::default().with_themes_directory(themes_directory)
    }

    fn write_solar_json(theme: &Path, frames: &str) {
        fs::write(theme.join("solar.json"), frames).unwrap();
    }

    /// A legacy (`solar.json`) theme: images live under the config's format.
    fn write_layered_theme(theme: &Path) {
        fs::create_dir_all(theme.join("jpg")).unwrap();
        write_solar_json(
            theme,
            r#"[{"index": 0, "altitude": 10.0, "azimuth": 20.0}]"#,
        );
        fs::write(theme.join("jpg").join("1.jpg"), b"image").unwrap();
    }

    #[test]
    fn a_staged_legacy_theme_is_accepted() {
        let root = scratch_themes_directory("accept");
        let staging = root.join("Demo.staging");
        write_layered_theme(&staging);

        assert!(verify_staged_theme(&staging, &config_for(&root)).is_ok());
    }

    /// The extractor only strips a top-level directory when *every* entry shares
    /// it, so a release asset that gains one top-level file (`README.md`,
    /// `LICENSE`) unpacks to `<staging>/<Theme>/solar.json`. Accepting that would
    /// delete the working theme and leave a directory the daemon cannot read.
    #[test]
    fn a_stray_top_level_file_does_not_make_the_nested_theme_valid() {
        let root = scratch_themes_directory("nested");
        let staging = root.join("Demo.staging");
        fs::create_dir_all(&staging).unwrap();
        fs::write(staging.join("README.md"), b"# release notes").unwrap();
        write_layered_theme(&staging.join("Demo"));

        assert!(verify_staged_theme(&staging, &config_for(&root)).is_err());
    }

    #[test]
    fn an_empty_staging_directory_is_rejected() {
        let root = scratch_themes_directory("empty");
        let staging = root.join("Demo.staging");
        fs::create_dir_all(&staging).unwrap();

        assert!(verify_staged_theme(&staging, &config_for(&root)).is_err());
    }

    #[test]
    fn a_theme_missing_a_frame_image_is_rejected() {
        let root = scratch_themes_directory("missing_image");
        let staging = root.join("Demo.staging");
        fs::create_dir_all(staging.join("jpg")).unwrap();
        write_solar_json(
            &staging,
            r#"[{"index": 0, "altitude": 10.0, "azimuth": 20.0},
                {"index": 1, "altitude": 40.0, "azimuth": 90.0}]"#,
        );
        fs::write(staging.join("jpg").join("1.jpg"), b"image").unwrap();

        assert!(verify_staged_theme(&staging, &config_for(&root)).is_err());
    }

    #[test]
    fn a_staged_manifest_theme_uses_its_own_image_format() {
        let root = scratch_themes_directory("manifest");
        let staging = root.join("Demo.staging");
        fs::create_dir_all(staging.join("png")).unwrap();
        fs::write(
            staging.join("theme.toml"),
            r#"
schema = 1

[theme]
id = "demo"
name = "Demo"
author = "alice"
version = 1
created_at = 1700000000
image_format = "png"

[solar]
mode = "native"

[[solar.frames]]
index = 0
altitude = 10.0
azimuth = 20.0
"#,
        )
        .unwrap();
        fs::write(staging.join("png").join("1.png"), b"image").unwrap();

        assert!(verify_staged_theme(&staging, &config_for(&root)).is_ok());
    }

    #[test]
    fn a_staging_directory_without_a_usable_name_is_rejected() {
        let root = scratch_themes_directory("no_name");

        assert!(verify_staged_theme(&root, &config_for(&root)).is_err());
    }

    /// `<id>.staging` is reused by every download of one theme, so the second
    /// download has to be judged on its own files. The daemon's validator reads
    /// through a path-keyed cache, and its entry from the first check would
    /// otherwise decide the second.
    #[test]
    fn reusing_a_staging_directory_revalidates_its_new_contents() {
        let root = scratch_themes_directory("reused");
        let staging = root.join("Demo.staging");

        // First download: one frame with its image.
        write_layered_theme(&staging);
        assert!(verify_staged_theme(&staging, &config_for(&root)).is_ok());

        // Second download into the same path: two frames, one image missing.
        fs::remove_dir_all(&staging).unwrap();
        fs::create_dir_all(staging.join("jpg")).unwrap();
        write_solar_json(
            &staging,
            r#"[{"index": 0, "altitude": 10.0, "azimuth": 20.0},
                {"index": 1, "altitude": 40.0, "azimuth": 90.0}]"#,
        );
        fs::write(staging.join("jpg").join("1.jpg"), b"image").unwrap();

        assert!(verify_staged_theme(&staging, &config_for(&root)).is_err());
    }

    // ── stage_archive ───────────────────────────────────────────────────────

    /// Builds a zip archive at `path` from `(name, contents)` pairs.
    fn write_zip(path: &Path, entries: &[(&str, &[u8])]) {
        use std::io::Write;
        use zip::CompressionMethod;
        use zip::write::SimpleFileOptions;

        let file = fs::File::create(path).unwrap();
        let mut writer = zip::ZipWriter::new(file);
        let options = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);

        for (name, contents) in entries {
            writer.start_file(*name, options).unwrap();
            writer.write_all(contents).unwrap();
        }

        writer.finish().unwrap();
    }

    fn runtime() -> tokio::runtime::Runtime {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap()
    }

    #[test]
    fn a_staged_archive_is_extracted_and_the_archive_is_removed() {
        let root = scratch_themes_directory("stage_ok");
        let temp_path = root.join("Demo.zip.tmp");
        let zip_path = root.join("Demo.zip");
        let staging = root.join("Demo.staging");
        fs::create_dir_all(&staging).unwrap();
        write_zip(
            &temp_path,
            &[("Demo/solar.json", b"[]"), ("Demo/jpg/1.jpg", b"x")],
        );

        runtime()
            .block_on(stage_archive(&temp_path, &zip_path, &staging))
            .unwrap();

        assert!(staging.join("solar.json").is_file());
        assert!(
            !zip_path.exists(),
            "the intermediate archive must not survive"
        );
        assert!(!temp_path.exists());
    }

    /// The leak this closes: `remove_file` used to be the last statement of the
    /// extraction chain, so a failed extraction left the archive behind forever.
    #[test]
    fn a_failed_extraction_still_removes_the_archive() {
        let root = scratch_themes_directory("stage_corrupt");
        let temp_path = root.join("Demo.zip.tmp");
        let zip_path = root.join("Demo.zip");
        let staging = root.join("Demo.staging");
        fs::create_dir_all(&staging).unwrap();
        // Not a zip at all: `extract_zip` fails after the rename.
        fs::write(&temp_path, b"<html>not a zip</html>").unwrap();

        let result = runtime().block_on(stage_archive(&temp_path, &zip_path, &staging));

        assert!(result.is_err());
        assert!(
            !zip_path.exists(),
            "a failed extraction must not leave the archive on disk"
        );
    }

    // ── is_retryable ────────────────────────────────────────────────────────

    #[test]
    fn a_cancelled_download_is_not_retried() {
        assert!(!is_retryable(&DwallSettingsError::Cancelled));
    }

    #[test]
    fn an_ordinary_failure_is_retried() {
        assert!(is_retryable(&DwallSettingsError::Other(
            "connection reset".to_string()
        )));
        assert!(is_retryable(&DwallSettingsError::Io(
            std::io::Error::other("disk full")
        )));
    }

    /// The renderer decides "the user cancelled" by matching this text
    /// (`isCancellation`), so the variant's message is part of the contract.
    #[test]
    fn the_cancellation_message_stays_what_the_renderer_matches() {
        assert_eq!(
            DwallSettingsError::Cancelled.to_string(),
            "Download cancelled"
        );
    }

    // ── Partial downloads ───────────────────────────────────────────────────

    fn downloader() -> ThemeDownloader {
        ThemeDownloader::new(HttpClient::new(None).unwrap())
    }

    #[test]
    fn a_non_empty_partial_download_is_kept_for_the_next_attempt() {
        let root = scratch_themes_directory("keep_partial");
        let temp_path = root.join("Demo.zip.tmp");
        fs::write(&temp_path, b"partial").unwrap();

        runtime().block_on(downloader().handle_download_error(&temp_path));

        assert_eq!(
            fs::read(&temp_path).unwrap(),
            b"partial",
            "a non-empty partial is what the next attempt resumes from"
        );

        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn an_empty_partial_download_is_removed() {
        let root = scratch_themes_directory("drop_empty");
        let temp_path = root.join("Demo.zip.tmp");
        fs::write(&temp_path, b"").unwrap();

        runtime().block_on(downloader().handle_download_error(&temp_path));

        assert!(
            !temp_path.exists(),
            "a zero-byte leftover has nothing to resume from"
        );

        let _ = fs::remove_dir_all(&root);
    }

    /// A one-shot HTTP server on localhost, so the downloader's resume arithmetic can
    /// be observed through the Range header it sends.
    struct OneShotServer {
        url: String,
        request: std::sync::Arc<std::sync::Mutex<String>>,
    }

    impl OneShotServer {
        fn new(response: Vec<u8>) -> Self {
            use std::io::{Read, Write};
            use std::net::TcpListener;

            let listener = TcpListener::bind("127.0.0.1:0").expect("a free local port");
            let url = format!("http://{}", listener.local_addr().unwrap());
            let request = std::sync::Arc::new(std::sync::Mutex::new(String::new()));
            let captured = std::sync::Arc::clone(&request);

            std::thread::spawn(move || {
                let Ok((mut stream, _)) = listener.accept() else {
                    return;
                };

                let mut head = Vec::new();
                let mut buffer = [0u8; 1024];
                while let Ok(read) = stream.read(&mut buffer) {
                    if read == 0 {
                        break;
                    }
                    head.extend_from_slice(&buffer[..read]);
                    if head.windows(4).any(|window| window == b"\r\n\r\n") {
                        break;
                    }
                }

                *captured.lock().unwrap() = String::from_utf8_lossy(&head).into_owned();
                let _ = stream.write_all(&response);
                let _ = stream.flush();
            });

            Self { url, request }
        }

        fn request_head(&self) -> String {
            self.request.lock().unwrap().clone()
        }
    }

    fn http_response(status: &str, body: &[u8]) -> Vec<u8> {
        let mut response = format!(
            "HTTP/1.1 {status}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
            body.len()
        )
        .into_bytes();
        response.extend_from_slice(body);

        response
    }

    /// The downloader reads the bytes already on disk and asks for the rest: this is
    /// the arithmetic download_file cannot do on its own.
    #[test]
    fn download_with_retry_resumes_from_the_bytes_already_on_disk() {
        let root = scratch_themes_directory("resume_retry");
        let temp_path = root.join("Demo.zip.tmp");
        fs::write(&temp_path, b"aaaa").unwrap();

        let server = OneShotServer::new(http_response("206 Partial Content", b"bbbb"));
        let cancel = Arc::new(AtomicBool::new(false));
        let no_emitter: Option<&ProgressEmitter<'_, tauri::Wry>> = None;

        runtime()
            .block_on(downloader().download_with_retry(
                &server.url,
                &temp_path,
                &cancel,
                no_emitter,
                "Demo",
            ))
            .unwrap();

        assert_eq!(fs::read(&temp_path).unwrap(), b"aaaabbbb");
        assert!(
            server
                .request_head()
                .to_lowercase()
                .contains("range: bytes=4-"),
            "the partial file's length has to become the range start: {}",
            server.request_head()
        );

        let _ = fs::remove_dir_all(&root);
    }
}
