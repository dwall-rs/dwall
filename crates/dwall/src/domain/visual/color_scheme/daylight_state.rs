//! Extreme daylight state detection

use time::Date;

use crate::{
    Position,
    domain::{
        time::solar_calculator::SolarPosition,
        visual::color_scheme::{ThresholdConfig, constants::WHITE_NIGHT_AMPLITUDE_MARGIN},
    },
};

/// The extreme daylight state for a given day
///
/// Determined by sampling [`SolarPosition::altitude()`] (which already includes
/// refraction correction) at 24 whole hours of the day. This should be updated
/// once a day around local sunrise and cached; do not recompute on the main
/// call path.
///
/// # Why sampling instead of an analytic formula
///
/// The analytic formula for the extreme (`90° - |φ - δ|`) requires accessing the
/// declination from outside `SolarPosition`, breaking encapsulation. Moreover,
/// the formula itself does not include refraction correction and would require
/// manually adding a constant, but `altitude()` already returns the apparent
/// altitude after refraction; the two cannot be mixed directly. The overhead
/// of 24 samples per day is far outweighed by the correctness and simplicity gains.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DaylightState {
    /// Normal day/night cycle, using standard hysteresis logic
    Normal,

    /// Midnight sun: the lowest altitude of the day is still above the base threshold
    ///
    /// The sun never descends into twilight; force light mode.
    MidnightSun,

    /// Polar night: the highest altitude of the day is still below the base threshold
    ///
    /// The sun never rises above twilight; astronomical determination is useless.
    /// Fall back to local clock waking-hour range judgement.
    PolarNight,

    /// White night: the sun oscillates near the base threshold without fully
    /// crossing the hysteresis dead zone
    ///
    /// Use the daily altitude amplitude midpoint as a dynamic threshold, with a
    /// forced wide dead zone to prevent small altitude perturbations from
    /// triggering frequent switches.
    WhiteNight {
        /// Minimum apparent altitude of the day (degrees, including refraction)
        min_altitude: f64,
        /// Maximum apparent altitude of the day (degrees, including refraction)
        max_altitude: f64,
    },
}

impl DaylightState {
    /// Detect the extreme state by sampling solar altitudes at 24 whole hours of the day
    ///
    /// # Parameters
    /// - `position`: geographic coordinates of the observer
    /// - `date`: the local date to evaluate (UTC)
    /// - `config`: threshold configuration for comparison with the extremes
    ///
    /// # When to call
    /// Recommended to call once around local midnight each day and cache the result.
    /// Should not be triggered on every call to `determine_color_scheme_with_hysteresis`.
    pub fn detect(position: &Position, date: Date, config: &ThresholdConfig) -> Self {
        // Sample apparent altitudes for hours 0–23 (refraction already included;
        // no extra constant needed)
        let altitudes: Vec<f64> = (0u8..24)
            .filter_map(|h| {
                date.with_hms(h, 0, 0)
                    .ok()
                    .map(|dt| SolarPosition::new(position, &dt.assume_utc()).altitude())
            })
            .collect();

        // If sampling fails, degrade to normal state (conservative)
        if altitudes.is_empty() {
            return DaylightState::Normal;
        }

        let min_alt = altitudes.iter().cloned().fold(f64::INFINITY, f64::min);
        let max_alt = altitudes.iter().cloned().fold(f64::NEG_INFINITY, f64::max);

        // Midnight sun: the minimum is still above the base threshold → sun never enters twilight
        if min_alt >= config.base_threshold {
            return DaylightState::MidnightSun;
        }

        // Polar night: the maximum is still below the base threshold → sun never leaves night
        if max_alt < config.base_threshold {
            return DaylightState::PolarNight;
        }

        // White night: altitude oscillation is very small and the maximum does not
        // significantly exceed the threshold
        // Conditions: amplitude < WHITE_NIGHT_AMPLITUDE_MARGIN,
        //        and max altitude < base_threshold + WHITE_NIGHT_AMPLITUDE_MARGIN
        // i.e. the sun merely grazes past the threshold without completing a full
        // day-night switch
        let amplitude = max_alt - min_alt;
        if amplitude < WHITE_NIGHT_AMPLITUDE_MARGIN
            && max_alt < config.base_threshold + WHITE_NIGHT_AMPLITUDE_MARGIN
        {
            return DaylightState::WhiteNight {
                min_altitude: min_alt,
                max_altitude: max_alt,
            };
        }

        DaylightState::Normal
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_config(base: f64) -> ThresholdConfig {
        ThresholdConfig {
            base_threshold: base,
            hysteresis_band: 0.5,
        }
    }

    #[test]
    fn midnight_sun_variant_exists() {
        let _config = make_config(-6.0);
        let _state = DaylightState::MidnightSun;
        // This test is a placeholder - actual testing happens in engine.rs
        // where the full decision logic is tested
    }

    #[test]
    fn white_night_variant_holds_altitudes() {
        let state = DaylightState::WhiteNight {
            min_altitude: -7.0,
            max_altitude: -5.0,
        };
        match state {
            DaylightState::WhiteNight {
                min_altitude,
                max_altitude,
            } => {
                assert_eq!(min_altitude, -7.0);
                assert_eq!(max_altitude, -5.0);
            }
            _ => panic!("Expected WhiteNight variant"),
        }
    }

    #[test]
    fn daylight_state_copy_semantics() {
        let state = DaylightState::Normal;
        let copied = state;
        assert_eq!(state, copied);
    }
}
