//! Dynamic threshold configuration based on geographic location

use crate::{
    Position,
    domain::visual::color_scheme::constants::{
        ARCTIC_CIRCLE_LAT, CIVIL_TWILIGHT_DEG, HIGH_LAT_BOUNDARY, HIGH_LAT_MAX_ADJUSTMENT,
        HYSTERESIS_BAND, POLAR_BASE_THRESHOLD, POLAR_MAX_ADJUSTMENT, THRESHOLD_MAX, THRESHOLD_MIN,
        TROPIC_LAT, TROPICAL_MAX_ADJUSTMENT,
    },
};

/// Dynamic switching threshold based on geographic location
///
/// Twilight duration varies significantly across latitudes:
/// - **Tropics** (< 23.5°): the sun crosses the horizon nearly vertically,
///   twilight is very short (within 15 minutes). The threshold can be
///   raised (toward -4.5°) to switch earlier.
/// - **Mid-latitudes** (23.5°–45°): use the standard civil twilight
///   threshold of -6°.
/// - **High latitudes** (45°–66.5°): the sun crosses the horizon at a
///   shallow angle; twilight can last 30–60 minutes. The threshold should
///   be deepened (toward -9°) to avoid switching to dark while it is
///   still quite bright.
/// - **Inside the polar circles** (> 66.5°): the shallowest path;
///   deepest threshold (-8° to -12°).
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct ThresholdConfig {
    /// Base switching threshold (degrees), dynamically computed from latitude
    pub(crate) base_threshold: f64,
    /// Hysteresis half-bandwidth (degrees), typically [`HYSTERESIS_BAND`]
    pub(crate) hysteresis_band: f64,
}

impl ThresholdConfig {
    /// Automatically compute the threshold based on geographic location
    pub fn from_position(position: &Position) -> Self {
        Self {
            base_threshold: Self::calculate_threshold(position),
            hysteresis_band: HYSTERESIS_BAND,
        }
    }

    /// Use the default civil twilight threshold (suitable when location is unknown)
    pub fn default_civil() -> Self {
        Self {
            base_threshold: CIVIL_TWILIGHT_DEG,
            hysteresis_band: HYSTERESIS_BAND,
        }
    }

    /// Compute the base threshold by latitude zone
    ///
    /// Uses `cos(lat)` as the latitude factor: 1.0 at the equator, 0.0 at the poles,
    /// which naturally reflects how the sun's path angle relative to the horizon
    /// varies with latitude.
    fn calculate_threshold(position: &Position) -> f64 {
        let abs_lat = position.latitude().abs();
        let lat_rad = abs_lat.to_radians();
        let latitude_factor = lat_rad.cos(); // 1.0 (equator) → 0.0 (pole)

        let threshold = if abs_lat > ARCTIC_CIRCLE_LAT {
            // Inside polar circles: -8° to -12°, deepening linearly with latitude
            // extreme_factor: 0.0 (66.5°) → 1.0 (90°)
            let extreme_factor = (abs_lat - ARCTIC_CIRCLE_LAT) / TROPIC_LAT;
            POLAR_BASE_THRESHOLD - extreme_factor * POLAR_MAX_ADJUSTMENT
        } else if abs_lat > HIGH_LAT_BOUNDARY {
            // High latitudes: -6° to about -9°, smooth transition using the cosine factor
            let adjustment = (1.0 - latitude_factor) * HIGH_LAT_MAX_ADJUSTMENT;
            CIVIL_TWILIGHT_DEG - adjustment
        } else if abs_lat < TROPIC_LAT {
            // Tropics: -6° to -4.5°, greatest shallowing at the equator
            // tropical_factor: 1.0 (equator) → 0.0 (tropics)
            let tropical_factor = (TROPIC_LAT - abs_lat) / TROPIC_LAT;
            CIVIL_TWILIGHT_DEG + tropical_factor * TROPICAL_MAX_ADJUSTMENT
        } else {
            // Mid-latitudes (23.5°–45°): standard threshold directly
            CIVIL_TWILIGHT_DEG
        };

        // Clamp to the reasonable astronomical range
        threshold.clamp(THRESHOLD_MIN, THRESHOLD_MAX)
    }

    /// Lower altitude bound for switching to Light
    ///
    /// When currently Dark, the altitude must **rise above** this value to switch to Light.
    #[inline]
    pub(crate) fn light_switch_point(&self) -> f64 {
        self.base_threshold + self.hysteresis_band
    }

    /// Upper altitude bound for switching to Dark
    ///
    /// When currently Light, the altitude must **drop below** this value to switch to Dark.
    #[inline]
    pub(crate) fn dark_switch_point(&self) -> f64 {
        self.base_threshold - self.hysteresis_band
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mock_position(lat: f64) -> Position {
        Position::from_raw_position(lat, 0.0, 0.)
    }

    fn make_config(base: f64) -> ThresholdConfig {
        ThresholdConfig {
            base_threshold: base,
            hysteresis_band: 0.5,
        }
    }

    #[test]
    fn threshold_tropical_is_shallower_than_civil() {
        // Tropical (latitude 10°) threshold should be shallower (closer to 0°) than -6°
        let pos_tropical = mock_position(10.0);
        let pos_midlat = mock_position(35.0);
        let t_tropical = ThresholdConfig::from_position(&pos_tropical).base_threshold;
        let t_midlat = ThresholdConfig::from_position(&pos_midlat).base_threshold;
        assert!(
            t_tropical > t_midlat,
            "Tropical threshold {t_tropical} should be higher than mid-latitude {t_midlat}"
        );
    }

    #[test]
    fn threshold_high_latitude_is_deeper_than_civil() {
        // High latitude (latitude 60°) threshold should be deeper (closer to -9°) than -6°
        let pos_high = mock_position(60.0);
        let pos_mid = mock_position(35.0);
        let t_high = ThresholdConfig::from_position(&pos_high).base_threshold;
        let t_mid = ThresholdConfig::from_position(&pos_mid).base_threshold;
        assert!(
            t_high < t_mid,
            "High-latitude threshold {t_high} should be lower than mid-latitude {t_mid}"
        );
    }

    #[test]
    fn threshold_clamped_within_bounds() {
        // Pole (90°) should not exceed [THRESHOLD_MIN, THRESHOLD_MAX]
        let pos_pole = mock_position(90.0);
        let t = ThresholdConfig::from_position(&pos_pole).base_threshold;
        assert!((THRESHOLD_MIN..=THRESHOLD_MAX).contains(&t));
    }

    #[test]
    fn default_civil_threshold() {
        let config = ThresholdConfig::default_civil();
        assert_eq!(config.base_threshold, CIVIL_TWILIGHT_DEG);
        assert_eq!(config.hysteresis_band, HYSTERESIS_BAND);
    }

    #[test]
    fn switch_points_symmetric_around_base() {
        let config = make_config(-6.0);
        assert_eq!(config.light_switch_point(), -5.5);
        assert_eq!(config.dark_switch_point(), -6.5);
    }
}
