//! Daily random theme selector with shuffle algorithm
//!
//! Provides a shuffle-based selection that ensures:
//! - No theme repeats until all themes have been used
//! - After all themes are used, reshuffle and start a new round
//! - State is kept in memory only (not persisted to config)

use getrandom::getrandom;
use time::Date;

/// Filter available themes by user-specified pool
///
/// - If pool is `None`, returns all available themes
/// - If pool is `Some(p)`, returns only themes that are in both `p` and `available`
/// - Invalid themes in pool (not in available) are silently ignored
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

/// Simple xorshift64 random number generator
///
/// Lightweight PRNG that only requires a seed, suitable for shuffle operations.
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

/// Get a random seed from OS entropy source
///
/// Falls back to system time if entropy source is unavailable.
fn get_random_seed() -> u64 {
    let mut buf = [0u8; 8];
    if getrandom(&mut buf).is_ok() {
        return u64::from_ne_bytes(buf);
    }

    // Fallback: use system time as seed
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos() as u64
}

/// Daily random theme selector
///
/// Selects a random theme each day using a shuffle algorithm.
/// The selection state is maintained in memory and lost on restart.
pub struct DailyRandomThemeSelector {
    /// Last date when a theme was applied
    last_applied_date: Option<Date>,
    /// Shuffled order of themes for current round
    shuffle_order: Vec<String>,
    /// Current position in the shuffle order
    current_index: usize,
}

impl DailyRandomThemeSelector {
    /// Create a new selector
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

    /// Select the next theme from the available themes
    ///
    /// Returns None if no themes are available.
    /// When all themes have been used, reshuffles and starts a new round.
    pub fn select_next<T: AsRef<str>>(&mut self, available_themes: &[T]) -> Option<String> {
        if available_themes.is_empty() {
            return None;
        }

        // Reshuffle if we've used all themes or this is the first selection
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

        // Same seed should produce same sequence
        for _ in 0..10 {
            assert_eq!(rng1.next(), rng2.next());
        }
    }

    #[test]
    fn test_simple_rng_shuffle() {
        let mut rng = SimpleRng::new(42);
        let mut items = vec![1, 2, 3, 4, 5];
        rng.shuffle(&mut items);

        // Should still contain all items
        let mut sorted = items.clone();
        sorted.sort();
        assert_eq!(sorted, vec![1, 2, 3, 4, 5]);
    }

    #[test]
    fn test_daily_selector_needs_switch() {
        let selector = DailyRandomThemeSelector::new();
        let today = Date::new(2026, time::Month::August, 1);

        // First run should need switch
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
        // When pool is Some([]), no themes should match
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
        // When pool has specific themes, only those should be selected
        let available = vec![
            "theme1".to_string(),
            "theme2".to_string(),
            "theme3".to_string(),
        ];
        let pool = Some(vec!["theme1".to_string(), "theme3".to_string()]);

        let themes_to_select = filter_themes_by_pool(&available, pool.as_ref());

        assert_eq!(themes_to_select.len(), 2);
        assert!(themes_to_select.contains(&&"theme1".to_string()));
        assert!(themes_to_select.contains(&&"theme3".to_string()));
    }

    #[test]
    fn test_pool_filtering_pool_has_invalid_themes() {
        // When pool contains themes not in available, they should be ignored
        let available = vec!["theme1".to_string(), "theme2".to_string()];
        let pool = Some(vec!["theme1".to_string(), "nonexistent".to_string()]);

        let themes_to_select = filter_themes_by_pool(&available, pool.as_ref());

        assert_eq!(themes_to_select.len(), 1);
        assert_eq!(themes_to_select[0], "theme1");
    }

    #[test]
    fn test_pool_filtering_none_pool_uses_all() {
        // When pool is None, all available themes should be used
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
        // When available is empty, result should be empty regardless of pool
        let available: Vec<String> = vec![];
        let pool = Some(vec!["theme1".to_string()]);

        let themes_to_select = filter_themes_by_pool(&available, pool.as_ref());

        assert!(themes_to_select.is_empty());
    }

    #[test]
    fn test_pool_filtering_case_sensitive() {
        // Pool filtering should be case-sensitive
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

        // Select all themes
        let mut selected = Vec::new();
        for _ in 0..3 {
            if let Some(theme) = selector.select_next(&themes) {
                selected.push(theme);
            }
        }

        // All themes should be selected (no duplicates in one round)
        let mut sorted = selected.clone();
        sorted.sort();
        assert_eq!(sorted, vec!["a", "b", "c"]);

        // Next selection should start a new round (reshuffle)
        let next = selector.select_next(&themes);
        assert!(next.is_some());
    }
}
