// === Time unit conversions ===
pub(super) const SECONDS_PER_MINUTE: u64 = 60;
pub(super) const SECONDS_PER_HOUR: u64 = 3600; // 60 * 60
pub(super) const SECONDS_PER_DAY: u64 = 86400; // 24 * 3600

// === Gregorian calendar cycle constants (based on the proleptic Gregorian calendar) ===

/// Total number of days in a 400-year cycle.
///
/// The Gregorian calendar has 97 leap years every 400 years (years divisible by 4 but not by 100,
/// unless also divisible by 400).
/// Calculation: 400 * 365 + 97 = 146,097 days.
pub(super) const DAYS_PER_400_YEARS: u32 = 146097;

/// Total number of days in a 100-year cycle (not a complete Gregorian cycle, used for auxiliary calculations).
///
/// In a 100-year period, there are typically 24 leap years (since the 100th year is not a leap year
/// unless it's divisible by 400).
/// Calculation: 100 * 365 + 24 = 36,524 days.
pub(super) const DAYS_PER_100_YEARS: u32 = 36524;

/// Total number of days in a 4-year cycle (including one leap year).
///
/// Calculation: 4 * 365 + 1 = 1,461 days.
pub(super) const DAYS_PER_4_YEARS: u32 = 1461;

/// Number of days in a common (non-leap) year.
pub(super) const DAYS_PER_YEAR: u32 = 365;

/// Number of days from the Unix epoch (1970-01-01) to the algorithm's reference date (March 1, year 0).
///
/// The algorithm uses March 1 of year 0 (0000-03-01) as its reference date, which offers advantages:
/// 1. Each year starts in March, placing the leap day (February 29) at the end of the year, simplifying calculations.
/// 2. Avoids leap-year edge cases when handling January and February.
///
/// 719,468 = number of days between 1970-01-01 and 0000-03-01.
pub(super) const UNIX_EPOCH_TO_MARCH_1_YEAR_0: u32 = 719468;

/// Month conversion magic constant.
///
/// Used to quickly map "day-of-year counted from March 1" to a month index via division.
/// The formula `(5 * day_of_year + 2) / 153` maps day-of-year to a month index (March = 0 through February = 11).
/// The value 153 is derived mathematically from the pattern of month lengths starting in March.
pub(super) const MONTH_DIVISOR: u32 = 153;

// === Cumulative days tables ===

/// Precomputed cumulative days table [common year, leap year].
///
/// Index `i` represents the cumulative number of days at the start of the `i`-th month (0-based,
/// so it actually corresponds to month `i+1`).
pub(super) const CUMULATIVE_DAYS: [[u32; 13]; 2] = [
    // Common year: cumulative days at the start of each month (index 0 = Jan 1 = day 0)
    [0, 31, 59, 90, 120, 151, 181, 212, 243, 273, 304, 334, 365],
    // Leap year
    [0, 31, 60, 91, 121, 152, 182, 213, 244, 274, 305, 335, 366],
];
