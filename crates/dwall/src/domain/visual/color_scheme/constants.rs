//! Astronomical constants for color scheme switching thresholds

// ─────────────────────────────────────────────────────────────
// Twilight Thresholds
// ─────────────────────────────────────────────────────────────

/// Civil twilight threshold (degrees)
///
/// When the center of the sun is 6° below the horizon, the scattered light
/// at ground level has largely disappeared.
/// macOS / GNOME and other systems use this as the baseline for automatic switching.
pub(crate) const CIVIL_TWILIGHT_DEG: f64 = -6.0;

/// Hysteresis half-bandwidth (degrees)
///
/// The actual switching line is `base_threshold ± HYSTERESIS_BAND`,
/// giving a total dead zone width of `2 × HYSTERESIS_BAND = 1°`.
/// The sun moves roughly 0.1° per minute around sunrise/sunset,
/// so a 1° dead zone provides approximately 10 minutes of buffer.
pub(crate) const HYSTERESIS_BAND: f64 = 0.5;

/// White night amplitude trigger threshold (degrees)
///
/// When the daily maximum solar altitude is below
/// `base_threshold + WHITE_NIGHT_AMPLITUDE_MARGIN`
/// and the full-day amplitude is less than this value, the day is classified
/// as a white night, enabling the dynamic midpoint switching logic.
pub(crate) const WHITE_NIGHT_AMPLITUDE_MARGIN: f64 = 4.0;

// ─────────────────────────────────────────────────────────────
// Polar Night Clock Fallback
// ─────────────────────────────────────────────────────────────

/// Polar night clock fallback: waking hours start hour (local time, inclusive)
pub(crate) const WAKING_HOUR_START: u8 = 7;

/// Polar night clock fallback: waking hours end hour (local time, exclusive)
pub(crate) const WAKING_HOUR_END: u8 = 18;

// ─────────────────────────────────────────────────────────────
// Latitude Zone Boundaries
// ─────────────────────────────────────────────────────────────

/// Arctic / Antarctic Circle latitude (degrees)
pub(crate) const ARCTIC_CIRCLE_LAT: f64 = 66.5;

/// High-latitude boundary (degrees)
pub(crate) const HIGH_LAT_BOUNDARY: f64 = 45.0;

/// Tropic latitude (degrees)
pub(crate) const TROPIC_LAT: f64 = 23.5;

// ─────────────────────────────────────────────────────────────
// Threshold Adjustment Limits
// ─────────────────────────────────────────────────────────────

/// Polar base threshold (degrees): adjusted deeper from this value
pub(crate) const POLAR_BASE_THRESHOLD: f64 = -8.0;

/// Maximum deepening inside polar circles (degrees): deepest to -12° (end of nautical twilight)
pub(crate) const POLAR_MAX_ADJUSTMENT: f64 = 4.0;

/// Maximum deepening at high latitudes (degrees): deepest to about -9°
pub(crate) const HIGH_LAT_MAX_ADJUSTMENT: f64 = 3.0;

/// Maximum shallowing in the tropics (degrees): shallowest to about -4.5°
pub(crate) const TROPICAL_MAX_ADJUSTMENT: f64 = 1.5;

/// Lower clamp for the threshold (degrees): end of nautical twilight,
/// beyond which further deepening is not meaningful
pub(crate) const THRESHOLD_MIN: f64 = -12.0;

/// Upper clamp for the threshold (degrees): middle of civil twilight;
/// shallower values are not appropriate
pub(crate) const THRESHOLD_MAX: f64 = -4.5;
