//! Theme file system path resolution.

use std::path::PathBuf;

use dwall::DWALL_CACHE_DIR;

/// Resolves file system paths for theme storage
pub struct ThemePathResolver;

impl ThemePathResolver {
    /// Get the directory path for a theme
    pub fn theme_dir(config: &dwall::Config, theme_id: &str) -> PathBuf {
        config.themes_directory().join(theme_id)
    }

    /// Staging directory used while a theme is being downloaded and extracted.
    ///
    /// It is renamed onto [`theme_dir`](Self::theme_dir) only after a successful
    /// extraction, so an interrupted download never removes a working theme.
    pub fn staging_dir(config: &dwall::Config, theme_id: &str) -> PathBuf {
        config
            .themes_directory()
            .join(format!("{theme_id}.staging"))
    }

    /// Get the path for the theme zip file
    pub fn zip_path(config: &dwall::Config, theme_id: &str) -> PathBuf {
        config.themes_directory().join(format!("{theme_id}.zip"))
    }

    /// Get the path for the temporary download file
    pub fn temp_zip_path(config: &dwall::Config, theme_id: &str) -> PathBuf {
        config
            .themes_directory()
            .join(format!("{theme_id}.zip.tmp"))
    }

    /// Get the thumbnails directory for a theme
    pub fn thumbnails_dir(theme_id: &str) -> PathBuf {
        DWALL_CACHE_DIR.join("thumbnails").join(theme_id)
    }

    /// Build a cached image path for one thumbnail resource.
    ///
    /// The URL is part of the file name, not just of the download lock: two mirror
    /// URLs for the same thumbnail are different resources, and a name keyed only by
    /// `(theme, serial)` meant the second request served — and could concurrently
    /// overwrite — the first one's bytes.
    pub fn cached_image_path(
        theme_id: &str,
        serial_number: u8,
        url: &str,
        extension: &str,
    ) -> PathBuf {
        let fingerprint = url_fingerprint(url);

        Self::thumbnails_dir(theme_id).join(format!("{serial_number}-{fingerprint}.{extension}"))
    }
}

/// A short, stable name for a URL.
///
/// FNV-1a, chosen over `DefaultHasher` because the file name has to stay the same
/// across runs and toolchains: the hash only has to identify a resource, and a
/// changed hash would silently orphan the cache rather than break anything.
fn url_fingerprint(url: &str) -> String {
    const OFFSET_BASIS: u64 = 0xcbf2_9ce4_8422_2325;
    const PRIME: u64 = 0x0000_0100_0000_01b3;

    let mut hash = OFFSET_BASIS;
    for byte in url.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(PRIME);
    }

    format!("{hash:016x}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_same_url_always_maps_to_the_same_file() {
        let first = ThemePathResolver::cached_image_path("Big Sur", 1, "https://a/1.jpg", "jpg");
        let second = ThemePathResolver::cached_image_path("Big Sur", 1, "https://a/1.jpg", "jpg");

        assert_eq!(first, second);
    }

    /// The lock is keyed by `(theme, serial, url)`, so the file has to be too:
    /// otherwise the two URLs take different locks and write the same path.
    #[test]
    fn two_urls_for_one_thumbnail_are_two_files() {
        let mirror =
            ThemePathResolver::cached_image_path("Big Sur", 1, "https://mirror/1.jpg", "jpg");
        let direct =
            ThemePathResolver::cached_image_path("Big Sur", 1, "https://github/1.jpg", "jpg");

        assert_ne!(mirror, direct);
    }

    #[test]
    fn the_file_name_keeps_the_serial_and_the_extension() {
        let path = ThemePathResolver::cached_image_path("Big Sur", 3, "https://a/9.avif", "avif");
        let name = path.file_name().unwrap().to_string_lossy().into_owned();

        assert!(name.starts_with("3-"), "unexpected name: {name}");
        assert!(name.ends_with(".avif"), "unexpected name: {name}");
        assert_eq!(
            path.parent(),
            Some(ThemePathResolver::thumbnails_dir("Big Sur").as_path())
        );
    }

    #[test]
    fn the_fingerprint_is_stable_and_url_specific() {
        assert_eq!(
            url_fingerprint("https://a/1.jpg"),
            url_fingerprint("https://a/1.jpg")
        );
        assert_ne!(
            url_fingerprint("https://a/1.jpg"),
            url_fingerprint("https://a/1.png")
        );
        assert_eq!(url_fingerprint("").len(), 16);
    }
}
