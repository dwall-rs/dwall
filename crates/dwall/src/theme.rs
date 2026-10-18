//! Theme validation, selection and solar-angle loading.

use std::cell::RefCell;
use std::collections::{HashMap, VecDeque};
use std::fs;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::time::SystemTime;

use time::Date;

use crate::config::{Config, ImageFormat};
use crate::theme_manifest::{THEME_MANIFEST_FILENAME, ThemeManifest};
use crate::{DwallResult, SolarAngle};

// ── Errors ──────────────────────────────────────────────────────────────────

/// Error type for solar theme-related operations
#[derive(Debug, thiserror::Error)]
pub enum ThemeError {
    #[error("Theme directory '{0}' does not exist")]
    ThemeDirectoryNotFound(String),
    #[error("Default theme is missing or not configured")]
    DefaultThemeMissing,
    #[error("Solar configuration file 'solar.json' is missing in theme directory")]
    SolarConfigurationMissing,
    #[error("Image files do not match solar configuration: expected {expected}, found {found}")]
    ImageSolarConfigurationMismatch { expected: usize, found: usize },
    #[error("Wallpaper image file '{path}' does not exist")]
    WallpaperImageMissing { path: String },
    #[error("No monitor-specific wallpaper configurations found")]
    MonitorWallpaperConfigurationMissing,
    #[error("Every configured monitor failed to receive a wallpaper ({failed} attempted)")]
    AllConfiguredMonitorsFailed { failed: usize },
    #[error("Theme manifest 'theme.toml' is missing in theme directory")]
    ThemeManifestMissing,
    #[error("Failed to parse theme manifest: {0}")]
    ThemeManifestParse(String),
    #[error("Unsupported theme schema version {0}; supported: 1")]
    UnsupportedThemeSchema(u32),
    #[error("Theme has no solar frames")]
    NoSolarFrames,
    #[error("Theme frame index {0} is out of range (max 255)")]
    FrameIndexOutOfRange(usize),
    #[error("Solar altitude {0} is out of range [-90, 90]")]
    InvalidAltitude(f64),
    #[error("Solar azimuth {0} is out of range [0, 360)")]
    InvalidAzimuth(f64),
}

// ── Theme directory & solar angle loading ───────────────────────────────────

/// Resolves the theme directory, returning `(path, is_customized)`.
pub fn get_theme_directory_path(configuration: &Config, theme_identifier: &str) -> (PathBuf, bool) {
    let path = configuration.themes_directory().join(theme_identifier);
    if path.exists() {
        return (path, false);
    }

    let path = configuration
        .customized_themes_directory()
        .join(theme_identifier);

    (path, true)
}

/// Whether a directory looks like a valid theme (has `solar.json` or `theme.toml`).
pub fn is_theme_directory(theme_dir: &Path) -> bool {
    theme_dir.join(SOLAR_CONFIG_FILENAME).is_file() || ThemeManifest::exists(theme_dir)
}

/// Maximum number of theme solar-angle sets kept resident.
///
/// Random mode can cycle through an unbounded theme pool; a small LRU bounds
/// memory without meaningfully hurting the common case of 1–3 active themes.
const SOLAR_CACHE_CAPACITY: usize = 32;

/// Parsed theme data: the solar frames plus, for `theme.toml` themes, the image
/// format declared by the manifest. Legacy `solar.json` themes fall back to the
/// global config's image format (`None`).
pub(crate) struct ThemeData {
    pub(crate) angles: Rc<Vec<SolarAngle>>,
    pub(crate) image_format: Option<ImageFormat>,
}

/// Bounded LRU cache of parsed theme data.
struct SolarAngleCache {
    entries: HashMap<PathBuf, CacheEntry>,
    order: VecDeque<PathBuf>,
}

/// One theme's parsed data together with what it was read from.
struct CacheEntry {
    data: Rc<ThemeData>,
    /// The file the data came from: `theme.toml` when the theme has one, otherwise
    /// `solar.json`.
    source: PathBuf,
    /// That file's modification time when it was read.
    ///
    /// The cache used to be keyed by directory alone, so a theme whose files were
    /// replaced under an unchanged path — a re-downloaded theme, an edited custom
    /// theme — kept serving the previous contents for the life of the daemon.
    modified: Option<SystemTime>,
}

impl SolarAngleCache {
    fn new() -> Self {
        Self {
            entries: HashMap::new(),
            order: VecDeque::new(),
        }
    }

    /// The cached data for `key`, when it is still the data `source` describes.
    fn get(
        &mut self,
        key: &Path,
        source: &Path,
        modified: Option<SystemTime>,
    ) -> Option<Rc<ThemeData>> {
        let entry = self.entries.get(key)?;

        if entry.source != source || entry.modified != modified {
            debug!(
                theme_directory = %key.display(),
                "Theme data changed on disk, dropping the cached copy"
            );
            self.remove(key);

            return None;
        }

        let data = Rc::clone(&entry.data);
        self.touch(key);

        Some(data)
    }

    fn insert(
        &mut self,
        key: PathBuf,
        source: PathBuf,
        modified: Option<SystemTime>,
        data: Rc<ThemeData>,
    ) {
        let entry = CacheEntry {
            data,
            source,
            modified,
        };

        if self.entries.contains_key(&key) {
            self.entries.insert(key.clone(), entry);
            self.touch(&key);
            return;
        }

        if self.entries.len() >= SOLAR_CACHE_CAPACITY
            && let Some(oldest) = self.order.pop_front()
        {
            self.entries.remove(&oldest);
        }

        self.entries.insert(key.clone(), entry);
        self.order.push_back(key);
    }

    fn remove(&mut self, key: &Path) {
        self.entries.remove(key);
        self.order.retain(|tracked| tracked != key);
    }

    fn touch(&mut self, key: &Path) {
        if let Some(position) = self.order.iter().position(|k| k == key)
            && let Some(k) = self.order.remove(position)
        {
            self.order.push_back(k);
        }
    }
}

/// The file a theme's data is read from, and when it last changed.
fn theme_data_source(theme_dir: &Path) -> (PathBuf, Option<SystemTime>) {
    let source = if ThemeManifest::exists(theme_dir) {
        theme_dir.join(THEME_MANIFEST_FILENAME)
    } else {
        theme_dir.join(SOLAR_CONFIG_FILENAME)
    };

    let modified = fs::metadata(&source)
        .ok()
        .and_then(|metadata| metadata.modified().ok());

    (source, modified)
}

thread_local! {
    /// Cache solar configuration to avoid repeated reads
    static SOLAR_CACHE: RefCell<SolarAngleCache> = RefCell::new(SolarAngleCache::new());
}

/// Builds the on-disk path of a theme's wallpaper image for the given index.
///
/// Image files are 1-based (`index + 1`), so the index is widened before the
/// increment: `u8::MAX` would otherwise wrap to `0.jpg` in release builds.
pub fn wallpaper_image_path(
    theme_dir: &Path,
    index: u8,
    image_format: &ImageFormat,
    is_customized: bool,
) -> PathBuf {
    let file_name = format!("{}.{}", u16::from(index) + 1, image_format.as_str());
    if is_customized {
        theme_dir.join("images").join(file_name)
    } else {
        theme_dir.join(image_format.as_str()).join(file_name)
    }
}

/// Reads a theme's frames — from `theme.toml` when present (custom themes),
/// otherwise the legacy `solar.json` — without caching.
pub fn read_solar_angles(theme_dir: &Path) -> DwallResult<Vec<SolarAngle>> {
    Ok(read_theme_data(theme_dir)?.angles.as_ref().clone())
}

/// Reads a theme's frames and image format without caching.
fn read_theme_data(theme_dir: &Path) -> DwallResult<ThemeData> {
    if ThemeManifest::exists(theme_dir) {
        let manifest = ThemeManifest::read(theme_dir)?;
        Ok(ThemeData {
            angles: Rc::new(manifest.solar_angles()?),
            image_format: Some(manifest.image_format().clone()),
        })
    } else {
        Ok(ThemeData {
            angles: Rc::new(read_solar_json(theme_dir)?),
            image_format: None,
        })
    }
}

/// Reads a legacy theme's `solar.json`.
fn read_solar_json(theme_dir: &Path) -> DwallResult<Vec<SolarAngle>> {
    let solar_config_path = theme_dir.join(SOLAR_CONFIG_FILENAME);
    if !solar_config_path.exists() {
        error!(
            solar_config_path = %solar_config_path.display(),
            "Solar configuration file is missing"
        );
        return Err(ThemeError::SolarConfigurationMissing.into());
    }

    let solar_config_content = fs::read_to_string(&solar_config_path).map_err(|read_error| {
        error!(
            solar_config_path = %solar_config_path.display(),
            error = ?read_error,
            "Failed to read solar configuration file"
        );
        read_error
    })?;

    let solar_angles: Vec<SolarAngle> =
        serde_json::from_str(&solar_config_content).map_err(|parse_error| {
            error!(
                solar_config_path = %solar_config_path.display(),
                error = ?parse_error,
                "Failed to parse solar configuration JSON"
            );
            parse_error
        })?;

    debug!(
        solar_angles_count = solar_angles.len(),
        "Successfully loaded solar configuration"
    );

    Ok(solar_angles)
}

/// Load theme data (frames + image format) for a specific theme directory.
///
/// Returns a shared handle so callers never clone the angle list on a hit.
pub(crate) fn load_cached_theme_data(theme_directory: &Path) -> DwallResult<Rc<ThemeData>> {
    let theme_directory = theme_directory.canonicalize()?;
    debug!(path = %theme_directory.display(), "Loading theme data from canonical and absolute path");

    let (source, modified) = theme_data_source(&theme_directory);

    if let Some(cached) =
        SOLAR_CACHE.with(|cache| cache.borrow_mut().get(&theme_directory, &source, modified))
    {
        debug!("Using cached theme data");
        return Ok(cached);
    }

    let data = Rc::new(read_theme_data(&theme_directory)?);
    SOLAR_CACHE.with(|cache| {
        cache.borrow_mut().insert(
            theme_directory.to_path_buf(),
            source,
            modified,
            data.clone(),
        );
    });

    Ok(data)
}

// ── Validation ──────────────────────────────────────────────────────────────

/// Solar configuration filename
const SOLAR_CONFIG_FILENAME: &str = "solar.json";

/// Validates that a solar theme exists and has the proper configuration and
/// image files.
pub struct ThemeValidator;

impl ThemeValidator {
    /// Validates a solar theme directory.
    pub fn validate(
        themes_directory: &Path,
        theme_identifier: &str,
        is_customized: bool,
        image_format: &ImageFormat,
    ) -> DwallResult<()> {
        Self::validate_with(
            themes_directory,
            theme_identifier,
            is_customized,
            image_format,
            load_cached_theme_data,
        )
    }

    /// Validates a solar theme directory, reading its files instead of the cache.
    ///
    /// [`validate`](Self::validate) reuses the daemon's parse cache, which is keyed
    /// by path. A caller that has just replaced the *contents* of a directory under
    /// an unchanged path — the settings app checking a freshly extracted staging
    /// directory, which reuses `<id>.staging` for every download of that theme —
    /// would otherwise be shown the previous download's frames and image format.
    pub fn validate_uncached(
        themes_directory: &Path,
        theme_identifier: &str,
        is_customized: bool,
        image_format: &ImageFormat,
    ) -> DwallResult<()> {
        Self::validate_with(
            themes_directory,
            theme_identifier,
            is_customized,
            image_format,
            |theme_directory| Ok(Rc::new(read_theme_data(theme_directory)?)),
        )
    }

    fn validate_with(
        themes_directory: &Path,
        theme_identifier: &str,
        is_customized: bool,
        image_format: &ImageFormat,
        load_theme_data: impl Fn(&Path) -> DwallResult<Rc<ThemeData>>,
    ) -> DwallResult<()> {
        trace!(
            theme_id = theme_identifier,
            themes_directory = %themes_directory.display(),
            "Starting solar theme validation"
        );

        let theme_directory_path = themes_directory.join(theme_identifier);

        if !theme_directory_path.exists() {
            warn!(
                theme_id = theme_identifier,
                theme_path = %theme_directory_path.display(),
                "Solar theme directory not found"
            );
            return Err(ThemeError::ThemeDirectoryNotFound(theme_identifier.to_string()).into());
        }

        let theme_data = load_theme_data(&theme_directory_path)?;

        // Manifest themes: enforce the documented angle ranges (solar.json is trusted).
        if theme_data.image_format.is_some() {
            for angle in theme_data.angles.iter() {
                if !(-90.0..=90.0).contains(&angle.altitude()) {
                    return Err(ThemeError::InvalidAltitude(angle.altitude()).into());
                }
                if !(0.0..360.0).contains(&angle.azimuth()) {
                    return Err(ThemeError::InvalidAzimuth(angle.azimuth()).into());
                }
            }
        }

        // Custom themes carry their own image format; legacy themes use the config's.
        let effective_format = theme_data
            .image_format
            .clone()
            .unwrap_or_else(|| image_format.clone());

        let expected_image_indices: Vec<u8> = theme_data
            .angles
            .iter()
            .map(|angle| angle.index())
            .collect();

        if !validate_theme_image_files(
            &theme_directory_path,
            &expected_image_indices,
            is_customized,
            effective_format.as_str(),
        ) {
            warn!(
                theme_id = theme_identifier,
                expected_images = expected_image_indices.len(),
                "Solar theme image validation failed"
            );
            return Err(ThemeError::ImageSolarConfigurationMismatch {
                expected: expected_image_indices.len(),
                found: 0,
            }
            .into());
        }

        info!(
            theme_id = theme_identifier,
            solar_angles_count = theme_data.angles.len(),
            is_customized = is_customized,
            "Solar theme validation completed successfully"
        );
        Ok(())
    }
}

/// Validates that all required image files exist for the theme's solar configuration
fn validate_theme_image_files(
    theme_directory: &Path,
    expected_image_indices: &[u8],
    is_customized: bool,
    image_file_format: &str,
) -> bool {
    let images_directory_path = if is_customized {
        theme_directory.join("images")
    } else {
        theme_directory.join(image_file_format)
    };

    if !images_directory_path.is_dir() {
        warn!(
            images_directory = %images_directory_path.display(),
            "Theme images directory not found or is not a directory"
        );
        return false;
    }

    let mut missing_images_count = 0;
    let validation_successful = expected_image_indices.iter().all(|&image_index| {
        let image_filename = format!("{}.{}", u16::from(image_index) + 1, image_file_format);
        let image_file_path = images_directory_path.join(image_filename);

        let image_exists = image_file_path.exists() && image_file_path.is_file();
        if !image_exists {
            missing_images_count += 1;
            warn!(
                image_path = %image_file_path.display(),
                image_index = image_index,
                "Required theme image file is missing"
            );
        }
        image_exists
    });

    if !validation_successful {
        error!(
            missing_images = missing_images_count,
            total_expected = expected_image_indices.len(),
            "Theme image validation failed due to missing files"
        );
    }

    validation_successful
}

// ── Random theme selection ──────────────────────────────────────────────────

/// Filter available themes by a user-specified pool.
///
/// `None` returns all available themes; `Some(p)` returns only themes present
/// in both the pool and the available set. Invalid pool entries are ignored.
pub fn filter_themes_by_pool(available: &[String], pool: Option<&Vec<String>>) -> Vec<String> {
    match pool {
        Some(p) => available
            .iter()
            .filter(|t| p.contains(t))
            .cloned()
            .collect(),
        None => available.to_vec(),
    }
}

/// Simple xorshift64 random number generator.
struct SimpleRng {
    state: u64,
}

impl SimpleRng {
    fn new(seed: u64) -> Self {
        // Ensure state is never zero (xorshift requires non-zero state)
        Self { state: seed.max(1) }
    }

    fn next(&mut self) -> u64 {
        let mut x = self.state;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.state = x;
        x
    }

    /// Fisher-Yates shuffle
    fn shuffle<T>(&mut self, slice: &mut [T]) {
        for i in (1..slice.len()).rev() {
            let j = (self.next() as usize) % (i + 1);
            slice.swap(i, j);
        }
    }
}

/// Seed for the non-cryptographic daily shuffle. Uniqueness (not
/// unpredictability) is all that matters here, so system time suffices and
/// avoids pulling in an OS entropy dependency.
fn get_random_seed() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos() as u64
}

/// Daily random theme selector using a shuffle algorithm.
///
/// Ensures no theme repeats until all themes have been used; state is kept in
/// memory only.
pub struct DailyRandomThemeSelector {
    last_applied_date: Option<Date>,
    shuffle_order: Vec<String>,
    current_index: usize,
}

impl DailyRandomThemeSelector {
    pub fn new() -> Self {
        Self {
            last_applied_date: None,
            shuffle_order: Vec::new(),
            current_index: 0,
        }
    }

    /// Check if theme needs to be switched (new day or first run)
    pub fn needs_switch(&self, today: &Date) -> bool {
        match self.last_applied_date {
            None => true,
            Some(ref date) => date != today,
        }
    }

    /// Select the next theme from the available themes.
    ///
    /// Returns `None` if no themes are available. When all themes have been
    /// used, reshuffles and starts a new round.
    pub fn select_next<T: AsRef<str>>(&mut self, available_themes: &[T]) -> Option<String> {
        if available_themes.is_empty() {
            return None;
        }

        // The round was drawn from an earlier list, and a theme can be deleted
        // between two draws — the settings app replaces theme directories while the
        // daemon keeps running. Naming one that is gone fails the cycle, so walk on
        // to the next theme that still exists.
        while self.current_index < self.shuffle_order.len() {
            let candidate = &self.shuffle_order[self.current_index];

            if available_themes
                .iter()
                .any(|theme| theme.as_ref() == candidate)
            {
                let theme = candidate.clone();
                self.current_index += 1;

                return Some(theme);
            }

            debug!(
                theme_id = candidate,
                "Skipping a theme that was removed after this round was drawn"
            );
            self.shuffle_order.remove(self.current_index);
        }

        self.shuffle_order = available_themes
            .iter()
            .map(|s| s.as_ref().to_string())
            .collect();
        let mut rng = SimpleRng::new(get_random_seed());
        rng.shuffle(&mut self.shuffle_order);
        self.current_index = 0;

        let theme = self.shuffle_order[self.current_index].clone();
        self.current_index += 1;
        Some(theme)
    }

    /// Mark that a theme was applied for the given date
    pub fn mark_applied(&mut self, today: &Date) {
        self.last_applied_date = Some(*today);
    }

    /// Re-seeds the selector with a theme that was already applied today.
    ///
    /// Used when the daemon restarts: today's draw is restored instead of being
    /// rolled again, and it is placed first in a fresh shuffle so it does not
    /// repeat tomorrow while other candidates remain. Returns `false` (leaving
    /// the selector untouched) when the theme is no longer available.
    pub fn restore<T: AsRef<str>>(
        &mut self,
        theme_id: &str,
        available: &[T],
        today: &Date,
    ) -> bool {
        if !available.iter().any(|theme| theme.as_ref() == theme_id) {
            return false;
        }

        let mut remaining: Vec<String> = available
            .iter()
            .map(|theme| theme.as_ref().to_string())
            .filter(|theme| theme != theme_id)
            .collect();
        SimpleRng::new(get_random_seed()).shuffle(&mut remaining);

        self.shuffle_order = std::iter::once(theme_id.to_string())
            .chain(remaining)
            .collect();
        self.current_index = 1;
        self.mark_applied(today);

        true
    }
}

impl Default for DailyRandomThemeSelector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── Theme fixtures ──────────────────────────────────────────────────────

    /// A scratch themes directory.
    fn scratch(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("dwall_theme_{name}"));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();

        dir
    }

    fn solar_json(frames: &[(u8, f64, f64)]) -> String {
        let entries: Vec<String> = frames
            .iter()
            .map(|(index, altitude, azimuth)| {
                format!(r#"{{"index": {index}, "altitude": {altitude}, "azimuth": {azimuth}}}"#)
            })
            .collect();

        format!("[{}]", entries.join(","))
    }

    /// Writes a legacy theme: `solar.json` plus one image per frame.
    fn write_legacy(
        themes_directory: &Path,
        theme_id: &str,
        frames: &[(u8, f64, f64)],
        format: &str,
        images: &[u32],
    ) {
        let theme_dir = themes_directory.join(theme_id);
        fs::create_dir_all(theme_dir.join(format)).unwrap();
        fs::write(theme_dir.join(SOLAR_CONFIG_FILENAME), solar_json(frames)).unwrap();

        for index in images {
            fs::write(
                theme_dir.join(format).join(format!("{index}.{format}")),
                "image",
            )
            .unwrap();
        }
    }

    /// Writes a manifest theme: `theme.toml` plus images under `images/`.
    fn write_manifest_themed(
        themes_directory: &Path,
        theme_id: &str,
        frames: &[(u8, f64, f64)],
        image_format: &str,
    ) {
        let theme_dir = themes_directory.join(theme_id);
        fs::create_dir_all(theme_dir.join("images")).unwrap();

        let frame_toml: String = frames
            .iter()
            .map(|(index, altitude, azimuth)| {
                format!(
                    "\n[[solar.frames]]\nindex = {index}\naltitude = {altitude}\nazimuth = {azimuth}\n"
                )
            })
            .collect();

        fs::write(
            theme_dir.join(THEME_MANIFEST_FILENAME),
            format!(
                r#"schema = 1

[theme]
id = "{theme_id}"
name = "{theme_id}"
author = "tester"
version = 1
created_at = 1700000000
image_format = "{image_format}"

[solar]
mode = "native"
{frame_toml}"#
            ),
        )
        .unwrap();

        for (index, _, _) in frames {
            fs::write(
                theme_dir
                    .join("images")
                    .join(format!("{}.{image_format}", u16::from(*index) + 1)),
                "image",
            )
            .unwrap();
        }
    }

    // ── ThemeValidator ──────────────────────────────────────────────────────

    #[test]
    fn validator_accepts_a_legacy_solar_json_theme() {
        let dir = scratch("validator_legacy");
        write_legacy(
            &dir,
            "Big Sur",
            &[(0, -30.0, 0.0), (1, 30.0, 180.0)],
            "jpg",
            &[1, 2],
        );

        assert!(ThemeValidator::validate(&dir, "Big Sur", false, &ImageFormat::Jpeg).is_ok());

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn validator_rejects_a_missing_theme_directory() {
        let dir = scratch("validator_missing");

        assert!(ThemeValidator::validate(&dir, "Absent", false, &ImageFormat::Jpeg).is_err());

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn validator_reports_a_missing_frame_image() {
        let dir = scratch("validator_missing_image");
        // Two frames, one image: the theme cannot be applied, so it must not validate.
        write_legacy(
            &dir,
            "Partial",
            &[(0, -30.0, 0.0), (1, 30.0, 180.0)],
            "jpg",
            &[1],
        );

        assert!(ThemeValidator::validate(&dir, "Partial", false, &ImageFormat::Jpeg).is_err());

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn validator_rejects_a_theme_without_frames() {
        let dir = scratch("validator_no_frames");
        let theme_dir = dir.join("Empty");
        fs::create_dir_all(theme_dir.join("jpg")).unwrap();
        fs::write(theme_dir.join("jpg").join("1.jpg"), "image").unwrap();

        assert!(ThemeValidator::validate(&dir, "Empty", false, &ImageFormat::Jpeg).is_err());

        let _ = fs::remove_dir_all(&dir);
    }

    /// Real themes reuse one image for several sun positions, so a repeated frame
    /// index is not a defect.
    #[test]
    fn validator_accepts_a_theme_with_repeated_frame_indices() {
        let dir = scratch("validator_repeated");
        write_legacy(
            &dir,
            "Repeated",
            &[(0, -30.0, 0.0), (0, -10.0, 0.0)],
            "jpg",
            &[1],
        );

        assert!(ThemeValidator::validate(&dir, "Repeated", false, &ImageFormat::Jpeg).is_ok());

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn validator_accepts_a_manifest_theme_and_its_own_format() {
        let dir = scratch("validator_manifest");
        write_manifest_themed(
            &dir,
            "My Theme",
            &[(0, -30.0, 0.0), (1, 30.0, 180.0)],
            "png",
        );

        // The configuration says jpeg; the manifest says png, and the manifest wins.
        assert!(ThemeValidator::validate(&dir, "My Theme", true, &ImageFormat::Jpeg).is_ok());

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn validator_rejects_an_altitude_out_of_range() {
        let dir = scratch("validator_altitude");
        write_manifest_themed(&dir, "Bad Altitude", &[(0, 91.0, 0.0)], "png");

        let error = ThemeValidator::validate(&dir, "Bad Altitude", true, &ImageFormat::Png)
            .expect_err("an altitude above the zenith is not a sun position");

        assert!(
            matches!(
                error,
                crate::error::DwallError::ThemeProcessing(ThemeError::InvalidAltitude(_))
            ),
            "{error}"
        );

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn validator_rejects_an_azimuth_of_360() {
        // 360 is north again, but the documented range is [0, 360).
        let dir = scratch("validator_azimuth");
        write_manifest_themed(&dir, "Bad Azimuth", &[(0, 10.0, 360.0)], "png");

        let error = ThemeValidator::validate(&dir, "Bad Azimuth", true, &ImageFormat::Png)
            .expect_err("360 is outside the documented range");

        assert!(
            matches!(
                error,
                crate::error::DwallError::ThemeProcessing(ThemeError::InvalidAzimuth(_))
            ),
            "{error}"
        );

        let _ = fs::remove_dir_all(&dir);
    }

    /// A legacy theme's angles are trusted, so an impossible one is accepted there —
    /// the range check is for manifests, which a maker generates.
    #[test]
    fn validator_trusts_a_legacy_themes_angles() {
        let dir = scratch("validator_legacy_angles");
        write_legacy(&dir, "Trusted", &[(0, 100.0, 400.0)], "jpg", &[1]);

        assert!(ThemeValidator::validate(&dir, "Trusted", false, &ImageFormat::Jpeg).is_ok());

        let _ = fs::remove_dir_all(&dir);
    }

    // ── Solar data cache ────────────────────────────────────────────────────

    fn cache_data(frames: usize) -> Rc<ThemeData> {
        let angles = (0..frames)
            .map(|index| SolarAngle::new(index as u8, 0.0, 0.0))
            .collect();

        Rc::new(ThemeData {
            angles: Rc::new(angles),
            image_format: None,
        })
    }

    /// The regression test for a theme replaced under an unchanged path: the cache
    /// is keyed by directory, so without checking the file the daemon kept the
    /// previous frames for its whole lifetime.
    #[test]
    fn cached_theme_data_is_invalidated_when_solar_json_changes() {
        let dir = scratch("cache_invalidated");
        write_legacy(
            &dir,
            "Theme",
            &[(0, -30.0, 0.0), (1, 30.0, 180.0)],
            "jpg",
            &[1, 2],
        );
        let theme_dir = dir.join("Theme");

        assert_eq!(load_cached_theme_data(&theme_dir).unwrap().angles.len(), 2);

        // A re-download replaces the frames under the same directory.
        std::thread::sleep(std::time::Duration::from_millis(20));
        fs::write(
            theme_dir.join(SOLAR_CONFIG_FILENAME),
            solar_json(&[(0, -30.0, 0.0), (1, 0.0, 90.0), (2, 30.0, 180.0)]),
        )
        .unwrap();

        assert_eq!(
            load_cached_theme_data(&theme_dir).unwrap().angles.len(),
            3,
            "the new frames must be read, not the cached ones"
        );

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn the_cached_data_is_reused_while_the_files_are_unchanged() {
        let dir = scratch("cache_reused");
        write_legacy(&dir, "Theme", &[(0, -30.0, 0.0)], "jpg", &[1]);
        let theme_dir = dir.join("Theme");

        let first = load_cached_theme_data(&theme_dir).unwrap();
        let second = load_cached_theme_data(&theme_dir).unwrap();

        assert!(
            Rc::ptr_eq(&first, &second),
            "an unchanged theme is served from the cache"
        );

        let _ = fs::remove_dir_all(&dir);
    }

    /// A manifest appearing later changes both the frames and the format, so the
    /// cached copy has to go even though it is the same directory.
    #[test]
    fn a_theme_that_gains_a_manifest_is_read_again() {
        let dir = scratch("cache_manifest");
        write_legacy(&dir, "Theme", &[(0, -30.0, 0.0)], "jpg", &[1]);
        let theme_dir = dir.join("Theme");

        let legacy = load_cached_theme_data(&theme_dir).unwrap();
        assert!(legacy.image_format.is_none());

        std::thread::sleep(std::time::Duration::from_millis(20));
        write_manifest_themed(&dir, "Theme", &[(0, -30.0, 0.0), (1, 30.0, 180.0)], "png");

        let manifest = load_cached_theme_data(&theme_dir).unwrap();
        assert_eq!(manifest.image_format, Some(ImageFormat::Png));
        assert_eq!(manifest.angles.len(), 2);

        let _ = fs::remove_dir_all(&dir);
    }

    fn cache_key(index: usize) -> PathBuf {
        PathBuf::from(format!("C:/themes/theme{index}"))
    }

    #[test]
    fn the_solar_cache_never_exceeds_its_capacity() {
        let mut cache = SolarAngleCache::new();

        for index in 0..SOLAR_CACHE_CAPACITY * 2 {
            let key = cache_key(index);
            cache.insert(key.clone(), key, None, cache_data(1));
        }

        assert_eq!(cache.entries.len(), SOLAR_CACHE_CAPACITY);
        assert_eq!(cache.order.len(), SOLAR_CACHE_CAPACITY);
    }

    #[test]
    fn the_solar_cache_evicts_the_oldest_entry_at_capacity() {
        let mut cache = SolarAngleCache::new();

        for index in 0..(SOLAR_CACHE_CAPACITY + 1) {
            let key = cache_key(index);
            cache.insert(key.clone(), key, None, cache_data(1));
        }

        let oldest = cache_key(0);

        assert!(
            cache.get(&oldest, &oldest, None).is_none(),
            "the first entry is the one to go"
        );
        let newest = cache_key(SOLAR_CACHE_CAPACITY);
        assert!(cache.get(&newest, &newest, None).is_some());
    }

    #[test]
    fn a_cache_hit_moves_the_entry_to_the_back() {
        let mut cache = SolarAngleCache::new();

        for index in 0..SOLAR_CACHE_CAPACITY {
            let key = cache_key(index);
            cache.insert(key.clone(), key, None, cache_data(1));
        }

        // Reading the oldest entry makes it the most recently used, so the eviction
        // takes the one after it instead.
        let touched = cache_key(0);
        assert!(cache.get(&touched, &touched, None).is_some());

        let added = cache_key(SOLAR_CACHE_CAPACITY);
        cache.insert(added.clone(), added.clone(), None, cache_data(1));

        assert!(cache.get(&touched, &touched, None).is_some());
        let next_oldest = cache_key(1);
        assert!(cache.get(&next_oldest, &next_oldest, None).is_none());
    }

    // ── Manifest precedence ─────────────────────────────────────────────────

    #[test]
    fn a_manifest_theme_wins_over_solar_json() {
        let dir = scratch("manifest_wins");
        // Both files present, with different frames and formats.
        write_legacy(
            &dir,
            "Theme",
            &[(0, -30.0, 0.0), (1, 0.0, 90.0)],
            "jpg",
            &[1, 2],
        );
        write_manifest_themed(&dir, "Theme", &[(0, -30.0, 0.0), (1, 30.0, 180.0)], "png");

        let data = load_cached_theme_data(&dir.join("Theme")).unwrap();

        assert_eq!(data.image_format, Some(ImageFormat::Png));
        assert_eq!(data.angles.len(), 2);
        assert_eq!(data.angles[1].altitude(), 30.0, "the manifest's frames");

        let _ = fs::remove_dir_all(&dir);
    }

    // ── Round selection over a changing theme list ──────────────────────────

    /// The regression test for a theme deleted mid-round: the draw is kept in
    /// memory, and naming a theme that is gone makes the next cycle fail.
    #[test]
    fn select_next_skips_a_theme_removed_after_the_round_started() {
        let mut selector = DailyRandomThemeSelector::new();
        // A round drawn from ["a", "b", "c"] whose next entry is "b"; the order is
        // set directly so the test does not depend on the shuffle's seed.
        selector.shuffle_order = vec!["b".to_string(), "c".to_string(), "a".to_string()];

        // "b" is deleted from disk before the next draw.
        let available = vec!["a".to_string(), "c".to_string()];

        assert_eq!(
            selector.select_next(&available),
            Some("c".to_string()),
            "the removed theme is skipped and the round keeps its order"
        );
        assert_eq!(selector.select_next(&available), Some("a".to_string()));

        // The round is exhausted, so the next draw starts a new one.
        assert!(available.contains(&selector.select_next(&available).unwrap()));
    }

    #[test]
    fn a_round_still_uses_every_theme_once_while_nothing_changes() {
        let mut selector = DailyRandomThemeSelector::new();
        let themes = vec!["a".to_string(), "b".to_string(), "c".to_string()];

        let mut drawn: Vec<String> = (0..3)
            .map(|_| selector.select_next(&themes).unwrap())
            .collect();
        drawn.sort();

        assert_eq!(drawn, themes);
    }

    #[test]
    fn test_simple_rng_deterministic() {
        let mut rng1 = SimpleRng::new(12345);
        let mut rng2 = SimpleRng::new(12345);

        for _ in 0..10 {
            assert_eq!(rng1.next(), rng2.next());
        }
    }

    #[test]
    fn test_simple_rng_shuffle() {
        let mut rng = SimpleRng::new(42);
        let mut items = vec![1, 2, 3, 4, 5];
        rng.shuffle(&mut items);

        let mut sorted = items.clone();
        sorted.sort();
        assert_eq!(sorted, vec![1, 2, 3, 4, 5]);
    }

    #[test]
    fn test_daily_selector_needs_switch() {
        let selector = DailyRandomThemeSelector::new();
        let today = Date::new(2026, time::Month::August, 1);

        assert!(selector.needs_switch(&today));
    }

    #[test]
    fn test_daily_selector_no_switch_same_day() {
        let mut selector = DailyRandomThemeSelector::new();
        let today = Date::new(2026, time::Month::August, 1);

        selector.mark_applied(&today);
        assert!(!selector.needs_switch(&today));
    }

    #[test]
    fn test_daily_selector_switch_new_day() {
        let mut selector = DailyRandomThemeSelector::new();
        let today = Date::new(2026, time::Month::August, 1);
        let tomorrow = Date::new(2026, time::Month::August, 2);

        selector.mark_applied(&today);
        assert!(selector.needs_switch(&tomorrow));
    }

    #[test]
    fn test_wallpaper_image_path_is_one_based() {
        let dir = Path::new("C:/themes/Demo");

        assert!(wallpaper_image_path(dir, 0, &ImageFormat::Jpeg, false).ends_with("jpg/1.jpg"));
        // `u8::MAX` must widen instead of wrapping to `0.jpg`.
        assert!(
            wallpaper_image_path(dir, u8::MAX, &ImageFormat::Jpeg, false).ends_with("jpg/256.jpg")
        );
        assert!(wallpaper_image_path(dir, 2, &ImageFormat::Png, true).ends_with("images/3.png"));
    }

    #[test]
    fn test_daily_selector_restore_keeps_today_and_avoids_repeat() {
        let mut selector = DailyRandomThemeSelector::new();
        let today = Date::new(2026, time::Month::August, 1);
        let themes = vec!["a".to_string(), "b".to_string(), "c".to_string()];

        assert!(selector.restore("b", &themes, &today));
        assert!(!selector.needs_switch(&today), "today's draw must be kept");

        // The restored theme sits first in the shuffle, so it is not drawn again
        // before the other candidates.
        let next = selector.select_next(&themes).unwrap();
        assert_ne!(next, "b");
    }

    #[test]
    fn test_daily_selector_restore_rejects_missing_theme() {
        let mut selector = DailyRandomThemeSelector::new();
        let today = Date::new(2026, time::Month::August, 1);
        let themes = vec!["a".to_string()];

        assert!(!selector.restore("gone", &themes, &today));
        assert!(selector.needs_switch(&today));
    }

    #[test]
    fn test_daily_selector_select_next() {
        let mut selector = DailyRandomThemeSelector::new();
        let themes = vec![
            "theme1".to_string(),
            "theme2".to_string(),
            "theme3".to_string(),
        ];

        let selected = selector.select_next(&themes);
        assert!(selected.is_some());
        assert!(themes.contains(&selected.unwrap()));
    }

    #[test]
    fn test_daily_selector_empty_themes() {
        let mut selector = DailyRandomThemeSelector::new();
        let themes: Vec<String> = vec![];

        assert!(selector.select_next(&themes).is_none());
    }

    #[test]
    fn test_pool_filtering_empty_pool() {
        let available = vec![
            "theme1".to_string(),
            "theme2".to_string(),
            "theme3".to_string(),
        ];
        let pool: Option<Vec<String>> = Some(vec![]);

        let themes_to_select = filter_themes_by_pool(&available, pool.as_ref());

        assert!(themes_to_select.is_empty());
    }

    #[test]
    fn test_pool_filtering_with_valid_pool() {
        let available = vec![
            "theme1".to_string(),
            "theme2".to_string(),
            "theme3".to_string(),
        ];
        let pool = Some(vec!["theme1".to_string(), "theme3".to_string()]);

        let themes_to_select = filter_themes_by_pool(&available, pool.as_ref());

        assert_eq!(themes_to_select.len(), 2);
        assert!(themes_to_select.contains(&"theme1".to_string()));
        assert!(themes_to_select.contains(&"theme3".to_string()));
    }

    #[test]
    fn test_pool_filtering_pool_has_invalid_themes() {
        let available = vec!["theme1".to_string(), "theme2".to_string()];
        let pool = Some(vec!["theme1".to_string(), "nonexistent".to_string()]);

        let themes_to_select = filter_themes_by_pool(&available, pool.as_ref());

        assert_eq!(themes_to_select.len(), 1);
        assert_eq!(themes_to_select[0], "theme1");
    }

    #[test]
    fn test_pool_filtering_none_pool_uses_all() {
        let available = vec![
            "theme1".to_string(),
            "theme2".to_string(),
            "theme3".to_string(),
        ];
        let pool: Option<Vec<String>> = None;

        let themes_to_select = filter_themes_by_pool(&available, pool.as_ref());

        assert_eq!(themes_to_select.len(), 3);
        assert_eq!(themes_to_select.len(), available.len());
        for (selected, avail) in themes_to_select.iter().zip(available.iter()) {
            assert_eq!(selected, avail);
        }
    }

    #[test]
    fn test_pool_filtering_empty_available() {
        let available: Vec<String> = vec![];
        let pool = Some(vec!["theme1".to_string()]);

        let themes_to_select = filter_themes_by_pool(&available, pool.as_ref());

        assert!(themes_to_select.is_empty());
    }

    #[test]
    fn test_pool_filtering_case_sensitive() {
        let available = vec!["Theme1".to_string(), "theme1".to_string()];
        let pool = Some(vec!["theme1".to_string()]);

        let themes_to_select = filter_themes_by_pool(&available, pool.as_ref());

        assert_eq!(themes_to_select.len(), 1);
        assert_eq!(themes_to_select[0], "theme1");
    }

    #[test]
    fn test_daily_selector_no_repeat_until_all_used() {
        let mut selector = DailyRandomThemeSelector::new();
        let themes = vec!["a".to_string(), "b".to_string(), "c".to_string()];

        let mut selected = Vec::new();
        for _ in 0..3 {
            if let Some(theme) = selector.select_next(&themes) {
                selected.push(theme);
            }
        }

        let mut sorted = selected.clone();
        sorted.sort();
        assert_eq!(sorted, vec!["a", "b", "c"]);

        let next = selector.select_next(&themes);
        assert!(next.is_some());
    }
}
