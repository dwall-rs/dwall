use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
    sync::{Arc, LazyLock},
    time::Duration,
};

use dwall::DWALL_CACHE_DIR;
use tokio::sync::{Mutex, OnceCell};

use crate::{error::DwallSettingsResult, fs::create_dir_if_missing};

// Cache key type for better type safety
#[derive(Hash, Eq, PartialEq, Clone)]
struct CacheKey {
    theme_id: String,
    serial_number: u8,
    url: String,
}

type ImageCache = Arc<Mutex<HashMap<CacheKey, Arc<OnceCell<PathBuf>>>>>;

static THUMBNAIL_CACHE: LazyLock<ImageCache> =
    LazyLock::new(|| Arc::new(Mutex::new(HashMap::new())));
static CLEANUP_FLAG: OnceCell<()> = OnceCell::const_new();

fn get_url_extension(url: &str) -> Option<&str> {
    let after_last_slash = url.rfind('/').map_or(url, |pos| &url[pos + 1..]);

    let clean_path = if let Some(query_pos) = after_last_slash.find('?') {
        &after_last_slash[..query_pos]
    } else if let Some(fragment_pos) = after_last_slash.find('#') {
        &after_last_slash[..fragment_pos]
    } else {
        after_last_slash
    };

    Path::new(clean_path)
        .extension()
        .and_then(|ext| ext.to_str())
}

async fn create_http_client() -> reqwest::Result<reqwest::Client> {
    debug!("Creating HTTP client");
    reqwest::ClientBuilder::new()
        .connect_timeout(Duration::from_secs(120))
        .read_timeout(Duration::from_secs(120))
        .build()
}

async fn download_image(url: &str, image_path: &Path) -> DwallSettingsResult<()> {
    debug!(url = url, "Downloading image");

    let client = create_http_client().await.map_err(|e| {
        error!(error = ?e, "Failed to create HTTP client");
        e
    })?;

    trace!(url = url, "Sending HTTP GET request");
    let response = client.get(url).send().await.map_err(|e| {
        error!(url = url, error = ?e, "Failed to get online image");
        e
    })?;

    trace!(url = url, "Received response. Reading bytes");
    let buffer = response.bytes().await.map_err(|e| {
        error!(error = ?e, "Failed to read image bytes from response");
        e
    })?;

    trace!(
        image_path = image_path.display().to_string(),
        "Writing image to path"
    );
    fs::write(image_path, buffer).map_err(|e| {
        error!(
            image_path = image_path.display().to_string(),
            error = ?e,
            "Failed to write image"
        );
        e.into()
    })
}

async fn ensure_directories(thumbnails_dir: &Path, theme_dir: &Path) -> DwallSettingsResult<()> {
    CLEANUP_FLAG
        .get_or_init(|| async {
            if !thumbnails_dir.exists() {
                warn!(
                    path = thumbnails_dir.display().to_string(),
                    "Thumbnails directory does not exist",
                );

                info!(
                    path = DWALL_CACHE_DIR.display().to_string(),
                    "Clearing old cache"
                );

                if let Err(e) = fs::remove_dir_all(&*DWALL_CACHE_DIR) {
                    error!(error = %e, "Failed to clear old cache directory");
                } else {
                    info!(
                        path = DWALL_CACHE_DIR.display().to_string(),
                        "Old cache cleared successfully"
                    );
                }
            }
        })
        .await;

    create_dir_if_missing(theme_dir).await.map_err(Into::into)
}

#[tauri::command]
pub async fn get_or_save_cached_thumbnails(
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

    let cache_key = CacheKey {
        theme_id: theme_id.to_string(),
        serial_number,
        url: url.to_string(),
    };

    let mut cache = THUMBNAIL_CACHE.lock().await;
    trace!("Acquired lock on thumbnail cache");

    let cell = cache
        .entry(cache_key.clone())
        .or_insert_with(|| {
            debug!(
                theme_id = theme_id,
                serial_number = serial_number,
                url = url,
                "Creating new OnceCell for cache key"
            );
            Arc::new(OnceCell::new())
        })
        .clone();
    drop(cache); // Release the lock early

    if let Some(cached_path) = cell.get() {
        info!(
            path = cached_path.display().to_string(),
            "Found the cached image"
        );
        return Ok(cached_path.clone());
    }

    let result = async {
        let thumbnails_dir = DWALL_CACHE_DIR.join("thumbnails");
        let theme_dir = thumbnails_dir.join(&cache_key.theme_id);

        debug!(
            thumbnails_dir = thumbnails_dir.display().to_string(),
            theme_dir = theme_dir.display().to_string(),
            "Ensuring directories exist"
        );
        ensure_directories(&thumbnails_dir, &theme_dir).await?;

        let extension = get_url_extension(&cache_key.url).unwrap_or("jpg");
        let image_path = theme_dir.join(format!("{}.{}", cache_key.serial_number, extension));

        if image_path.exists() {
            info!(
                image_path = image_path.display().to_string(),
                "Image already cached"
            );
            return Ok(image_path);
        }

        debug!(
            url = url,
            image_path = image_path.display().to_string(),
            "Downloading image from URL"
        );
        download_image(&cache_key.url, &image_path).await?;

        info!(
            image_path = image_path.display().to_string(),
            "Image successfully cached"
        );

        Ok(image_path)
    }
    .await;

    match &result {
        Ok(path) => {
            debug!(
                path = path.display().to_string(),
                "Caching result in OnceCell"
            );
            // Ignore error if initialization fails (another task might have initialized it)
            let _ = cell.set(path.clone());
        }
        Err(e) => {
            error!(error = ?e, "Failed to cache image");
            // Don't cache errors
        }
    }

    result
}
