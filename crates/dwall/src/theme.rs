//! Theme validation, selection and solar-angle loading.

use std::cell::RefCell;
use std::collections::{HashMap, VecDeque};
use std::fs;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use time::Date;

use crate::config::{Config, ImageFormat};
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
}

// ── Theme directory & solar angle loading ───────────────────────────────────

/// Resolves the theme directory, returning `(path, is_customized)`.
pub(crate) fn get_theme_directory_path(
    configuration: &Config,
    theme_identifier: &str,
) -> (PathBuf, bool) {
    let path = configuration.themes_directory().join(theme_identifier);
    if path.exists() {
        return (path, false);
    }

    let path = configuration
        .customized_themes_directory()
        .join(theme_identifier);

    (path, true)
}

/// Maximum number of theme solar-angle sets kept resident.
///
/// Random mode can cycle through an unbounded theme pool; a small LRU bounds
/// memory without meaningfully hurting the common case of 1–3 active themes.
const SOLAR_CACHE_CAPACITY: usize = 32;

/// Bounded LRU cache of parsed `solar.json` files.
struct SolarAngleCache {
    entries: HashMap<PathBuf, Rc<Vec<SolarAngle>>>,
    order: VecDeque<PathBuf>,
}

impl SolarAngleCache {
    fn new() -> Self {
        Self {
            entries: HashMap::new(),
            order: VecDeque::new(),
        }
    }

    fn get(&mut self, key: &Path) -> Option<Rc<Vec<SolarAngle>>> {
        let value = self.entries.get(key).cloned();
        if value.is_some() {
            self.touch(key);
        }
        value
    }

    fn insert(&mut self, key: PathBuf, value: Rc<Vec<SolarAngle>>) {
        if self.entries.contains_key(&key) {
            self.entries.insert(key.clone(), value);
            self.touch(&key);
            return;
        }

        if self.entries.len() >= SOLAR_CACHE_CAPACITY
            && let Some(oldest) = self.order.pop_front()
        {
            self.entries.remove(&oldest);
        }

        self.entries.insert(key.clone(), value);
        self.order.push_back(key);
    }

    fn touch(&mut self, key: &Path) {
        if let Some(position) = self.order.iter().position(|k| k == key)
            && let Some(k) = self.order.remove(position)
        {
            self.order.push_back(k);
        }
    }
}

thread_local! {
    /// Cache solar configuration to avoid repeated reads
    static SOLAR_CACHE: RefCell<SolarAngleCache> = RefCell::new(SolarAngleCache::new());
}

/// Load solar configuration for a specific theme directory.
///
/// Returns a shared handle so callers never clone the angle list on a hit.
pub(crate) fn load_cached_solar_angles(theme_directory: &Path) -> DwallResult<Rc<Vec<SolarAngle>>> {
    let theme_directory = theme_directory.canonicalize()?;
    debug!(path = %theme_directory.display(), "Loading solar configuration from canonical and absolute path");

    if let Some(cached_angles) = SOLAR_CACHE.with(|cache| cache.borrow_mut().get(&theme_directory))
    {
        debug!("Using cached solar configuration");
        return Ok(cached_angles);
    }

    let solar_config_path = theme_directory.join(SOLAR_CONFIG_FILENAME);
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

    let solar_angles = Rc::new(solar_angles);
    SOLAR_CACHE.with(|cache| {
        cache
            .borrow_mut()
            .insert(theme_directory.to_path_buf(), solar_angles.clone());
    });

    Ok(solar_angles)
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

        let solar_angle_configuration = load_cached_solar_angles(&theme_directory_path)?;
        let expected_image_indices: Vec<u8> = solar_angle_configuration
            .iter()
            .map(|angle| angle.index())
            .collect();

        if !validate_theme_image_files(
            &theme_directory_path,
            &expected_image_indices,
            is_customized,
            image_format.as_str(),
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
            solar_angles_count = solar_angle_configuration.len(),
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
        let image_filename = format!("{}.{}", image_index + 1, image_file_format);
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

        if self.current_index >= self.shuffle_order.len() {
            self.shuffle_order = available_themes
                .iter()
                .map(|s| s.as_ref().to_string())
                .collect();
            let mut rng = SimpleRng::new(get_random_seed());
            rng.shuffle(&mut self.shuffle_order);
            self.current_index = 0;
        }

        let theme = self.shuffle_order[self.current_index].clone();
        self.current_index += 1;
        Some(theme)
    }

    /// Mark that a theme was applied for the given date
    pub fn mark_applied(&mut self, today: &Date) {
        self.last_applied_date = Some(*today);
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
