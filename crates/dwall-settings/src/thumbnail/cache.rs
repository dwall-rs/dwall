//! Thumbnail cache orchestrator.

use std::path::{Path, PathBuf};

use tokio::io::AsyncReadExt;

use crate::error::{DwallSettingsError, DwallSettingsResult};
use crate::network::HttpClient;
use crate::theme::paths::ThemePathResolver;
use crate::thumbnail::cleanup::Cleanup;
use crate::thumbnail::storage::Storage;
use crate::thumbnail::store::{CacheKey, clear_cache, initialize_cache, lock_for_download};

/// Bytes inspected to recognise an image format.
const MAGIC_BYTES_LEN: usize = 16;

/// The most a thumbnail may be.
///
/// A thumbnail is a few hundred KB at most, so this only has to be generous enough
/// that no real image reaches it; its job is to stop a server that streams forever.
const MAX_THUMBNAIL_BYTES: u64 = 32 * 1024 * 1024;

/// Manages the thumbnail cache system
pub struct ThumbnailCache {
    http_client: HttpClient,
}

impl ThumbnailCache {
    pub fn new(client: HttpClient) -> Self {
        Self {
            http_client: client,
        }
    }

    /// Applies a changed network configuration to the HTTP client.
    pub fn reconfigure(&self, network: Option<&dwall::config::Network>) -> DwallSettingsResult<()> {
        self.http_client.reconfigure(network)
    }

    /// Get or save a cached thumbnail
    pub(crate) async fn get_or_save_thumbnail(
        &self,
        theme_id: &str,
        serial_number: u8,
        url: &str,
    ) -> DwallSettingsResult<PathBuf> {
        trace!(
            theme_id = theme_id,
            serial_number = serial_number,
            url = url,
            "Received request to cache image"
        );

        initialize_cache().await?;

        let cache_key = CacheKey {
            theme_id: theme_id.to_string(),
            serial_number,
            url: url.to_string(),
        };

        // Single-flight per key: concurrent requests for the same thumbnail wait
        // for the in-flight download instead of fetching it twice.
        let (_lock, _guard) = lock_for_download(cache_key.clone()).await;

        if let Some(path) = self.try_get_cached_file(&cache_key).await {
            return Ok(path);
        }

        self.trigger_probabilistic_cleanup().await;

        self.download_and_cache_image(&cache_key).await
    }

    /// Returns the cached file for `cache_key` when it exists and is a real image.
    async fn try_get_cached_file(&self, cache_key: &CacheKey) -> Option<PathBuf> {
        let path = self.build_image_path(cache_key);

        let size = tokio::fs::metadata(&path)
            .await
            .map(|metadata| metadata.len())
            .unwrap_or(0);

        if size == 0 {
            return None;
        }

        // A mirror (or a captive portal) can answer `200 OK` with an HTML body;
        // without this check the broken response would stay cached for 30 days.
        if !looks_like_image_file(&path).await {
            warn!(
                path = %path.display(),
                "Cached thumbnail is not a valid image, discarding it"
            );
            let _ = tokio::fs::remove_file(&path).await;
            return None;
        }

        if let Err(e) = Storage::update_file_access_time(&path) {
            warn!(path = %path.display(), error = %e, "Failed to update file access time");
        }

        Some(path)
    }

    /// Trigger cache cleanup with a small probability
    async fn trigger_probabilistic_cleanup(&self) {
        if rand::random::<f32>() < 0.05 {
            tokio::spawn(async {
                if let Err(e) = Cleanup::cleanup_expired_cache().await {
                    error!(error = %e, "Failed to clean up expired cache");
                }

                if let Err(e) = Cleanup::enforce_cache_size_limit().await {
                    error!(error = %e, "Failed to enforce cache size limit");
                }
            });
        }
    }

    /// Download and cache an image
    async fn download_and_cache_image(&self, cache_key: &CacheKey) -> DwallSettingsResult<PathBuf> {
        let image_path = self.build_image_path(cache_key);

        Storage::ensure_directories(
            image_path
                .parent()
                .ok_or_else(|| DwallSettingsError::Other("Invalid thumbnail path".to_string()))?,
        )
        .await?;

        // Remove any leftover empty/corrupt file so it is not mistaken for a cached image
        if image_path.exists() {
            let _ = tokio::fs::remove_file(&image_path).await;
        }

        debug!(url = cache_key.url, image_path = %image_path.display(), "Downloading image from URL");

        download_thumbnail(&self.http_client, &cache_key.url, &image_path).await?;

        debug!(path = %image_path.display(), "Image successfully cached");
        Ok(image_path)
    }

    /// Build the image path for a cache key
    ///
    /// One path per key, so the lock table and the file name agree: the key is
    /// `(theme, serial, url)` and the name carries all three.
    fn build_image_path(&self, cache_key: &CacheKey) -> PathBuf {
        let extension = get_url_extension(&cache_key.url).unwrap_or("jpg");

        ThemePathResolver::cached_image_path(
            &cache_key.theme_id,
            cache_key.serial_number,
            &cache_key.url,
            extension,
        )
    }
}

/// The sibling file a download is streamed into before it becomes the cache entry.
fn partial_file_path(path: &Path) -> PathBuf {
    let mut name = path.as_os_str().to_os_string();
    name.push(".part");

    PathBuf::from(name)
}

/// Downloads one thumbnail and only publishes it once it is complete.
///
/// The HTTP client streams straight to the path it is given, so handing it the
/// final name meant a transfer that died mid-body left a truncated file behind.
/// The magic-byte check accepted it (a truncated JPEG still starts with a JPEG
/// header), and it was then served and its access time refreshed on every hit,
/// so the cache never healed and only "clear cache" could remove it.
///
/// The body therefore goes to a sibling `.part` file. The final name appears only
/// after the download, the non-empty check and the image check have all passed;
/// every failure removes the partial file.
async fn download_thumbnail(
    http_client: &HttpClient,
    url: &str,
    image_path: &Path,
) -> DwallSettingsResult<()> {
    let partial = partial_file_path(image_path);

    // A crash between two attempts can leave one behind; it is ours by
    // construction, and a resumed transfer is not wanted for a thumbnail.
    let _ = tokio::fs::remove_file(&partial).await;

    let written = match http_client
        .download_file(url, &partial, 0, MAX_THUMBNAIL_BYTES, None, None)
        .await
    {
        Ok(written) => written,
        Err(error) => {
            let _ = tokio::fs::remove_file(&partial).await;
            return Err(error);
        }
    };

    if written == 0 {
        let _ = tokio::fs::remove_file(&partial).await;
        return Err(DwallSettingsError::Other(format!(
            "Empty response downloading {url}"
        )));
    }

    if !looks_like_image_file(&partial).await {
        let _ = tokio::fs::remove_file(&partial).await;
        return Err(DwallSettingsError::Other(format!(
            "Response from {url} is not an image"
        )));
    }

    if let Err(error) = tokio::fs::rename(&partial, image_path).await {
        let _ = tokio::fs::remove_file(&partial).await;
        return Err(error.into());
    }

    Ok(())
}

/// Extract file extension from URL
fn get_url_extension(url: &str) -> Option<&str> {
    let after_last_slash = url.rfind('/').map_or(url, |pos| &url[pos + 1..]);

    let clean_path = if let Some(query_pos) = after_last_slash.find('?') {
        &after_last_slash[..query_pos]
    } else if let Some(fragment_pos) = after_last_slash.find('#') {
        &after_last_slash[..fragment_pos]
    } else {
        after_last_slash
    };

    std::path::Path::new(clean_path)
        .extension()
        .and_then(|ext| ext.to_str())
}

/// Whether the first bytes of `path` look like a supported image format.
async fn looks_like_image_file(path: &Path) -> bool {
    let Ok(mut file) = tokio::fs::File::open(path).await else {
        return false;
    };

    let mut buffer = [0u8; MAGIC_BYTES_LEN];
    match file.read(&mut buffer).await {
        Ok(read) => looks_like_image(&buffer[..read]),
        Err(_) => false,
    }
}

/// Magic-byte check for the formats the themes actually ship (JPEG, PNG, WebP,
/// GIF, BMP and the ISO-BMFF family used by AVIF/HEIC).
fn looks_like_image(bytes: &[u8]) -> bool {
    const JPEG: &[u8] = &[0xFF, 0xD8, 0xFF];
    const PNG: &[u8] = &[0x89, b'P', b'N', b'G'];
    const GIF: &[u8] = b"GIF8";
    const BMP: &[u8] = b"BM";
    const RIFF: &[u8] = b"RIFF";
    const WEBP: &[u8] = b"WEBP";
    const FTYP: &[u8] = b"ftyp";

    let starts_with =
        |prefix: &[u8]| bytes.len() >= prefix.len() && &bytes[..prefix.len()] == prefix;
    let contains_at = |offset: usize, marker: &[u8]| {
        bytes.len() >= offset + marker.len() && &bytes[offset..offset + marker.len()] == marker
    };

    starts_with(JPEG)
        || starts_with(PNG)
        || starts_with(GIF)
        || starts_with(BMP)
        || (starts_with(RIFF) && contains_at(8, WEBP))
        || contains_at(4, FTYP)
}

/// Get or save a cached thumbnail
pub async fn get_or_save_cached_thumbnails(
    cache: &ThumbnailCache,
    theme_id: &str,
    serial_number: u8,
    url: &str,
) -> DwallSettingsResult<PathBuf> {
    cache
        .get_or_save_thumbnail(theme_id, serial_number, url)
        .await
}

/// Clear the thumbnail cache
///
/// Removes the cached files from disk (not just the in-memory bookkeeping), so
/// "clear cache" means what the user expects.
pub async fn clear_thumbnail_cache() -> crate::error::DwallSettingsResult<u64> {
    info!("Manual cache cleanup requested");

    let thumbnails_dir = dwall::DWALL_CACHE_DIR.join("thumbnails");
    let mut removed_bytes = 0;

    if thumbnails_dir.exists() {
        removed_bytes = Storage::get_directory_size(&thumbnails_dir)
            .await
            .unwrap_or(0);

        if let Err(e) = tokio::fs::remove_dir_all(&thumbnails_dir).await {
            error!(dir = %thumbnails_dir.display(), error = %e, "Failed to remove the thumbnail directory");
        }
    }

    let tracked_keys = clear_cache().await;

    info!(
        removed_bytes = removed_bytes,
        tracked_keys = tracked_keys,
        "Thumbnail cache cleared"
    );

    Ok(removed_bytes)
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;
    use crate::network::testing::{TestServer, response};

    #[test]
    fn recognises_the_shipped_image_formats() {
        assert!(looks_like_image(&[0xFF, 0xD8, 0xFF, 0xE0, 0x00, 0x10]));
        assert!(looks_like_image(&[
            0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A
        ]));
        assert!(looks_like_image(b"GIF89a\0\0\0\0\0\0"));
        assert!(looks_like_image(b"BM\0\0\0\0\0\0\0\0\0\0\0\0"));

        // WebP: "RIFF" + size + "WEBP"
        assert!(looks_like_image(b"RIFF\x24\x00\x00\x00WEBPVP8 "));

        // AVIF/HEIC: ISO-BMFF, `ftyp` at offset 4
        assert!(looks_like_image(
            b"\x00\x00\x00\x20ftypavif\x00\x00\x00\x00"
        ));
    }

    #[test]
    fn rejects_html_challenge_pages_and_short_bodies() {
        // What a mirror or captive portal returns with `200 OK`.
        assert!(!looks_like_image(b"<!DOCTYPE html><html><head>"));
        assert!(!looks_like_image(b"{\"error\":\"rate limited\"}"));
        assert!(!looks_like_image(b""));
        assert!(!looks_like_image(b"RIFF"));
    }

    /// A truncated body is not a cache entry, no matter what the first bytes look
    /// like: the next attempt has to download again instead of serving the partial
    /// file for the rest of the cache's life.
    #[test]
    fn an_interrupted_thumbnail_is_not_kept_and_the_next_attempt_downloads_again() {
        let image = {
            let mut image = vec![0xFF, 0xD8, 0xFF, 0xE0, 0x00, 0x10, 0x4A, 0x46];
            image.extend_from_slice(&[0u8; 56]);
            image
        };
        let mut truncated =
            b"HTTP/1.1 200 OK\r\nContent-Length: 1024\r\nConnection: close\r\n\r\n".to_vec();
        truncated.extend_from_slice(&image);

        let server = TestServer::scripted(vec![truncated, response("200 OK", &image)]);
        let directory =
            std::env::temp_dir().join(format!("dwall_thumbnail_atomic_{}", std::process::id()));
        let _ = fs::remove_dir_all(&directory);
        fs::create_dir_all(&directory).unwrap();
        let image_path = directory.join("1-fingerprint.jpg");

        let runtime = tokio::runtime::Runtime::new().unwrap();
        let client = HttpClient::new(None).unwrap();

        runtime
            .block_on(download_thumbnail(&client, &server.url, &image_path))
            .expect_err("the body ends before its declared length");

        assert!(
            !image_path.exists(),
            "a failed download must not publish the final cache path"
        );
        assert!(
            !partial_file_path(&image_path).exists(),
            "the partial file must be removed when the transfer fails"
        );

        // The retry sees a clean slate and publishes the complete image.
        runtime
            .block_on(download_thumbnail(&client, &server.url, &image_path))
            .expect("the second attempt completes");

        assert_eq!(fs::read(&image_path).unwrap(), image);
        assert!(
            !partial_file_path(&image_path).exists(),
            "a successful download renames the partial file away"
        );

        let _ = fs::remove_dir_all(&directory);
    }

    #[test]
    fn extracts_the_extension_from_a_url() {
        assert_eq!(
            get_url_extension("https://example.com/a/1.avif"),
            Some("avif")
        );
        assert_eq!(
            get_url_extension("https://example.com/a/1.avif?token=abc#frag"),
            Some("avif")
        );
        assert_eq!(get_url_extension("https://example.com/a/1"), None);
    }
}
