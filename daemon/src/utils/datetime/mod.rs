mod consts;
mod error;
mod math;
mod month;

use std::fmt;
use std::time::{Duration, SystemTime};

use crate::utils::datetime::consts::{SECONDS_PER_DAY, SECONDS_PER_HOUR, SECONDS_PER_MINUTE};
use crate::utils::datetime::error::{DateTimeError, DateTimeResult};
use crate::utils::datetime::math::{days_since_epoch, days_to_ymd, is_leap_year};

pub use crate::utils::datetime::month::Month;

/// UTC date-time structure
///
/// # Performance considerations
/// - Methods `year()`, `month()`, and `day()` involve computation; it is recommended to use `ymd()` or `ymd_hms()` to fetch all values at once.
/// - Timestamp operations (e.g., `timestamp()`, `add_seconds()`) are O(1).
/// - `ymd()` and `ymd_hms()` use optimized O(1) algorithms combined with lookup tables.
///
/// # Leap second note
/// This implementation does not support leap seconds (second must be < 60). Unix timestamps themselves do not include leap second information.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct UtcDateTime {
    inner: Duration,
}

impl UtcDateTime {
    /// Creates a new UTC date-time with the specified components.
    pub fn new(
        year: u16,
        month: Month,
        day: u8,
        hour: u8,
        minute: u8,
        second: u8,
    ) -> DateTimeResult<Self> {
        // Validate input parameters
        if year < 1970 {
            return Err(DateTimeError::InvalidYear(year));
        }
        if day == 0 || day > month.days_in_month(year) {
            return Err(DateTimeError::InvalidDay(day));
        }
        if hour >= 24 {
            return Err(DateTimeError::InvalidHour(hour));
        }
        if minute >= 60 {
            return Err(DateTimeError::InvalidMinute(minute));
        }
        // Leap seconds are not supported
        if second >= 60 {
            return Err(DateTimeError::InvalidSecond(second));
        }

        let days = days_since_epoch(year, month, day)?;
        let total_seconds = days as u64 * SECONDS_PER_DAY
            + hour as u64 * SECONDS_PER_HOUR
            + minute as u64 * SECONDS_PER_MINUTE
            + second as u64;

        Ok(Self {
            inner: Duration::from_secs(total_seconds),
        })
    }

    /// Gets the current UTC time.
    #[inline]
    pub fn now() -> Self {
        let duration = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .expect("Invalid system time");
        Self { inner: duration }
    }

    /// Creates a new UTC date-time from a Unix timestamp (seconds).
    #[inline]
    pub const fn from_timestamp(secs: u64) -> Self {
        Self {
            inner: Duration::from_secs(secs),
        }
    }

    /// Creates a new UTC date-time from a Unix timestamp (milliseconds).
    #[inline]
    pub const fn from_timestamp_millis(millis: u64) -> Self {
        Self {
            inner: Duration::from_millis(millis),
        }
    }

    /// Gets the Unix timestamp (seconds).
    #[inline]
    pub const fn timestamp(&self) -> u64 {
        self.inner.as_secs()
    }

    /// Gets the Unix timestamp (milliseconds).
    #[inline]
    pub const fn timestamp_millis(&self) -> u64 {
        self.inner.as_millis() as u64
    }

    /// Returns the year.
    ///
    /// Note: If you need year, month, and day together, use `ymd()` or `ymd_hms()` for better performance.
    #[inline]
    pub fn year(&self) -> u16 {
        let (year, _, _) = self.ymd();
        year
    }

    /// Returns the month.
    ///
    /// Note: If you need year, month, and day together, use `ymd()` or `ymd_hms()` for better performance.
    #[inline]
    pub fn month(&self) -> Month {
        let (_, month, _) = self.ymd();
        month
    }

    /// Returns the day of the month.
    ///
    /// Note: If you need year, month, and day together, use `ymd()` or `ymd_hms()` for better performance.
    #[inline]
    pub fn day(&self) -> u8 {
        let (_, _, day) = self.ymd();
        day
    }

    /// Returns the hour (0–23).
    #[inline]
    pub const fn hour(&self) -> u8 {
        ((self.inner.as_secs() % SECONDS_PER_DAY) / SECONDS_PER_HOUR) as u8
    }

    /// Returns the minute (0–59).
    #[inline]
    pub const fn minute(&self) -> u8 {
        ((self.inner.as_secs() % SECONDS_PER_HOUR) / SECONDS_PER_MINUTE) as u8
    }

    /// Returns the second (0–59).
    #[inline]
    pub const fn second(&self) -> u8 {
        (self.inner.as_secs() % SECONDS_PER_MINUTE) as u8
    }

    /// Returns the millisecond component (0–999).
    #[inline]
    pub const fn millisecond(&self) -> u16 {
        (self.inner.as_millis() % 1000) as u16
    }

    /// Returns the year, month, and day (recommended for fetching multiple date components efficiently).
    #[inline]
    pub fn ymd(&self) -> (u16, Month, u8) {
        let days = (self.inner.as_secs() / SECONDS_PER_DAY) as u32;
        days_to_ymd(days)
    }

    /// Returns the hour, minute, and second.
    #[inline]
    pub const fn hms(&self) -> (u8, u8, u8) {
        (self.hour(), self.minute(), self.second())
    }

    /// Returns year, month, day, hour, minute, and second (more efficient than calling individual methods).
    #[inline]
    pub fn ymd_hms(&self) -> (u16, Month, u8, u8, u8, u8) {
        let (y, m, d) = self.ymd();
        let (h, min, s) = self.hms();
        (y, m, d, h, min, s)
    }

    /// Adds the specified number of seconds.
    #[inline]
    pub fn add_seconds(&self, secs: u64) -> DateTimeResult<Self> {
        self.inner
            .checked_add(Duration::from_secs(secs))
            .map(|inner| Self { inner })
            .ok_or(DateTimeError::Overflow)
    }

    /// Adds the specified number of minutes.
    #[inline]
    pub fn add_minutes(&self, mins: u64) -> DateTimeResult<Self> {
        self.add_seconds(
            mins.checked_mul(SECONDS_PER_MINUTE)
                .ok_or(DateTimeError::Overflow)?,
        )
    }

    /// Adds the specified number of hours.
    #[inline]
    pub fn add_hours(&self, hours: u64) -> DateTimeResult<Self> {
        self.add_seconds(
            hours
                .checked_mul(SECONDS_PER_HOUR)
                .ok_or(DateTimeError::Overflow)?,
        )
    }

    /// Adds the specified number of days.
    #[inline]
    pub fn add_days(&self, days: u64) -> DateTimeResult<Self> {
        self.add_seconds(
            days.checked_mul(SECONDS_PER_DAY)
                .ok_or(DateTimeError::Overflow)?,
        )
    }

    /// Subtracts the specified number of seconds.
    #[inline]
    pub fn sub_seconds(&self, secs: u64) -> DateTimeResult<Self> {
        self.inner
            .checked_sub(Duration::from_secs(secs))
            .map(|inner| Self { inner })
            .ok_or(DateTimeError::Overflow)
    }

    /// Subtracts the specified number of minutes.
    #[inline]
    pub fn sub_minutes(&self, mins: u64) -> DateTimeResult<Self> {
        self.sub_seconds(
            mins.checked_mul(SECONDS_PER_MINUTE)
                .ok_or(DateTimeError::Overflow)?,
        )
    }

    /// Subtracts the specified number of hours.
    #[inline]
    pub fn sub_hours(&self, hours: u64) -> DateTimeResult<Self> {
        self.sub_seconds(
            hours
                .checked_mul(SECONDS_PER_HOUR)
                .ok_or(DateTimeError::Overflow)?,
        )
    }

    /// Subtracts the specified number of days.
    #[inline]
    pub fn sub_days(&self, days: u64) -> DateTimeResult<Self> {
        self.sub_seconds(
            days.checked_mul(SECONDS_PER_DAY)
                .ok_or(DateTimeError::Overflow)?,
        )
    }

    /// Returns the start of the day (00:00:00).
    #[inline]
    pub fn start_of_day(&self) -> Self {
        let days = self.inner.as_secs() / SECONDS_PER_DAY;
        Self {
            inner: Duration::from_secs(days * SECONDS_PER_DAY),
        }
    }

    /// Returns the end of the day (23:59:59).
    #[inline]
    pub fn end_of_day(&self) -> Self {
        let days = self.inner.as_secs() / SECONDS_PER_DAY;
        Self {
            inner: Duration::from_secs((days + 1) * SECONDS_PER_DAY - 1),
        }
    }

    /// Computes the difference between this and another time in seconds.
    ///
    /// Note: The result may be negative if `self < other`.
    #[inline]
    pub fn diff_seconds(&self, other: &Self) -> i64 {
        self.inner.as_secs() as i64 - other.inner.as_secs() as i64
    }

    /// Computes the difference between this and another time in days.
    ///
    /// Note: The result may be negative if `self < other`.
    #[inline]
    pub fn diff_days(&self, other: &Self) -> i64 {
        self.diff_seconds(other) / SECONDS_PER_DAY as i64
    }

    /// Computes the duration between this and another time.
    ///
    /// Returns an error if `self < other`.
    #[inline]
    pub fn duration_since(&self, other: &Self) -> DateTimeResult<Duration> {
        if self.inner >= other.inner {
            Ok(self.inner - other.inner)
        } else {
            Err(DateTimeError::NegativeDuration)
        }
    }

    /// Checks if this time is before the given time.
    #[inline]
    pub const fn is_before(&self, other: &Self) -> bool {
        self.inner.as_secs() < other.inner.as_secs()
    }

    /// Checks if this time is after the given time.
    #[inline]
    pub const fn is_after(&self, other: &Self) -> bool {
        self.inner.as_secs() > other.inner.as_secs()
    }

    /// Checks if this time falls within the inclusive range [start, end].
    #[inline]
    pub const fn is_between_inclusive(&self, start: &Self, end: &Self) -> bool {
        self.inner.as_secs() >= start.inner.as_secs() && self.inner.as_secs() <= end.inner.as_secs()
    }

    /// Checks if this time falls within the exclusive range (start, end).
    #[inline]
    pub const fn is_between_exclusive(&self, start: &Self, end: &Self) -> bool {
        self.inner.as_secs() > start.inner.as_secs() && self.inner.as_secs() < end.inner.as_secs()
    }

    /// Formats the date and time as an RFC 3339 string (YYYY-MM-DDTHH:MM:SS.mmmZ).
    #[inline]
    pub fn to_rfc3339(&self) -> String {
        let (year, month, day) = self.ymd();
        let (hour, minute, second) = self.hms();
        format!(
            "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}.{:03}Z",
            year,
            month as u8,
            day,
            hour,
            minute,
            second,
            self.millisecond()
        )
    }

    /// Checks if the year of this date is a leap year.
    #[inline]
    pub fn is_leap_year(&self) -> bool {
        is_leap_year(self.year())
    }
}

impl Default for UtcDateTime {
    #[inline]
    fn default() -> Self {
        Self {
            inner: Duration::from_secs(0), // Unix epoch
        }
    }
}

impl fmt::Display for UtcDateTime {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_rfc3339())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_and_components() {
        let dt = UtcDateTime::new(2024, Month::March, 15, 10, 30, 45).unwrap();
        assert_eq!(dt.year(), 2024);
        assert_eq!(dt.month(), Month::March);
        assert_eq!(dt.day(), 15);
        assert_eq!(dt.hour(), 10);
        assert_eq!(dt.minute(), 30);
        assert_eq!(dt.second(), 45);
    }

    #[test]
    fn test_arithmetic() {
        let dt = UtcDateTime::from_timestamp(1000000);
        let dt2 = dt.add_days(1).unwrap();
        assert_eq!(dt2.timestamp(), 1000000 + SECONDS_PER_DAY);

        let dt3 = dt2.sub_hours(2).unwrap();
        assert_eq!(
            dt3.timestamp(),
            1000000 + SECONDS_PER_DAY - 2 * SECONDS_PER_HOUR
        );
    }

    #[test]
    fn test_ymd_hms() {
        let test_cases = vec![
            (1970, Month::January, 1, 0, 0, 0),
            (2025, Month::January, 1, 12, 30, 45),
            (2100, Month::January, 1, 12, 30, 45),
            (2100, Month::February, 28, 14, 30, 45),
        ];

        for case in test_cases {
            let dt = UtcDateTime::new(case.0, case.1, case.2, case.3, case.4, case.5).unwrap();
            let ymd_hms = dt.ymd_hms();

            assert_eq!(ymd_hms, case);
        }
    }

    #[test]
    fn test_leap_year_boundary() {
        // test leap year February boundary
        let dt1 = UtcDateTime::new(2024, Month::February, 29, 0, 0, 0).unwrap();
        assert_eq!(dt1.day(), 29);

        // test non-leap year February boundary
        let result = UtcDateTime::new(2023, Month::February, 29, 0, 0, 0);
        assert!(result.is_err());
    }

    #[test]
    fn test_roundtrip_conversion() {
        // test ymd -> timestamp -> ymd
        let test_dates = vec![
            (2025, Month::January, 1),
            (2025, Month::December, 31),
            (2050, Month::June, 15),
            (2100, Month::February, 28),
        ];

        for (y, m, d) in test_dates {
            let dt = UtcDateTime::new(y, m, d, 12, 30, 45).unwrap();
            let ts = dt.timestamp();
            let dt2 = UtcDateTime::from_timestamp(ts);
            assert_eq!(dt2.ymd(), (y, m, d));
        }
    }

    #[test]
    fn test_leap_second_rejection() {
        // test rejection of leap seconds
        let result = UtcDateTime::new(2025, Month::June, 30, 23, 59, 60);
        assert!(result.is_err());
    }

    #[test]
    fn test_start_end_of_day() {
        let dt = UtcDateTime::new(2025, Month::June, 15, 14, 30, 45).unwrap();

        let start = dt.start_of_day();
        assert_eq!(start.hms(), (0, 0, 0));
        assert_eq!(start.day(), 15);

        let end = dt.end_of_day();
        assert_eq!(end.hms(), (23, 59, 59));
        assert_eq!(end.day(), 15);
    }

    #[test]
    fn test_comparison_methods() {
        let dt1 = UtcDateTime::from_timestamp(1000);
        let dt2 = UtcDateTime::from_timestamp(2000);
        let dt3 = UtcDateTime::from_timestamp(3000);

        assert!(dt1.is_before(&dt2));
        assert!(dt3.is_after(&dt2));
        assert!(dt2.is_between_inclusive(&dt1, &dt3));
        assert!(!dt1.is_between_exclusive(&dt1, &dt3));
    }
}
