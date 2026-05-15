mod consts;
mod error;
mod math;
mod month;
mod offset;

use std::fmt;
use std::ops::{Add, Sub};
use std::time::{Duration, SystemTime};

use self::consts::{SECONDS_PER_DAY, SECONDS_PER_HOUR, SECONDS_PER_MINUTE};
use self::error::Result;
use self::math::{days_since_epoch, days_to_ymd, is_leap_year};

pub use self::error::Error;
pub use self::offset::Offset;

pub use self::month::Month;

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
    pub fn new(year: u16, month: Month, day: u8, hour: u8, minute: u8, second: u8) -> Result<Self> {
        // Validate input parameters
        if year < 1970 {
            return Err(Error::InvalidYear(year));
        }
        if day == 0 || day > month.days_in_month(year) {
            return Err(Error::InvalidDay(day));
        }
        if hour >= 24 {
            return Err(Error::InvalidHour(hour));
        }
        if minute >= 60 {
            return Err(Error::InvalidMinute(minute));
        }
        // Leap seconds are not supported
        if second >= 60 {
            return Err(Error::InvalidSecond(second));
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
    pub const fn year(&self) -> u16 {
        let (year, _, _) = self.ymd();
        year
    }

    /// Returns the month.
    ///
    /// Note: If you need year, month, and day together, use `ymd()` or `ymd_hms()` for better performance.
    #[inline]
    pub const fn month(&self) -> Month {
        let (_, month, _) = self.ymd();
        month
    }

    /// Returns the day of the month.
    ///
    /// Note: If you need year, month, and day together, use `ymd()` or `ymd_hms()` for better performance.
    #[inline]
    pub const fn day(&self) -> u8 {
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
    pub const fn ymd(&self) -> (u16, Month, u8) {
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
    pub const fn ymd_hms(&self) -> (u16, Month, u8, u8, u8, u8) {
        let (y, m, d) = self.ymd();
        let (h, min, s) = self.hms();
        (y, m, d, h, min, s)
    }

    /// Adds the specified number of seconds.
    #[inline]
    pub fn add_seconds(&self, secs: u64) -> Result<Self> {
        self.inner
            .checked_add(Duration::from_secs(secs))
            .map(|inner| Self { inner })
            .ok_or(Error::Overflow)
    }

    /// Adds the specified number of seconds without checking for overflow.
    #[inline]
    pub fn add_seconds_unchecked(&self, secs: u64) -> Self {
        Self {
            inner: self.inner.add(Duration::from_secs(secs)),
        }
    }

    /// Adds the specified number of minutes.
    #[inline]
    pub fn add_minutes(&self, mins: u64) -> Result<Self> {
        self.add_seconds(
            mins.checked_mul(SECONDS_PER_MINUTE)
                .ok_or(Error::Overflow)?,
        )
    }

    /// Adds the specified number of hours.
    #[inline]
    pub fn add_hours(&self, hours: u64) -> Result<Self> {
        self.add_seconds(hours.checked_mul(SECONDS_PER_HOUR).ok_or(Error::Overflow)?)
    }

    /// Adds the specified number of days.
    #[inline]
    pub fn add_days(&self, days: u64) -> Result<Self> {
        self.add_seconds(days.checked_mul(SECONDS_PER_DAY).ok_or(Error::Overflow)?)
    }

    /// Subtracts the specified number of seconds.
    #[inline]
    pub fn sub_seconds(&self, secs: u64) -> Result<Self> {
        self.inner
            .checked_sub(Duration::from_secs(secs))
            .map(|inner| Self { inner })
            .ok_or(Error::Overflow)
    }

    /// Subtracts the specified number of seconds without checking for overflow.
    #[inline]
    pub fn sub_seconds_unchecked(&self, secs: u64) -> Self {
        Self {
            inner: self.inner.sub(Duration::from_secs(secs)),
        }
    }

    /// Subtracts the specified number of minutes.
    #[inline]
    pub fn sub_minutes(&self, mins: u64) -> Result<Self> {
        self.sub_seconds(
            mins.checked_mul(SECONDS_PER_MINUTE)
                .ok_or(Error::Overflow)?,
        )
    }

    /// Subtracts the specified number of hours.
    #[inline]
    pub fn sub_hours(&self, hours: u64) -> Result<Self> {
        self.sub_seconds(hours.checked_mul(SECONDS_PER_HOUR).ok_or(Error::Overflow)?)
    }

    /// Subtracts the specified number of days.
    #[inline]
    pub fn sub_days(&self, days: u64) -> Result<Self> {
        self.sub_seconds(days.checked_mul(SECONDS_PER_DAY).ok_or(Error::Overflow)?)
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
    pub fn duration_since(&self, other: &Self) -> Result<Duration> {
        if self.inner >= other.inner {
            Ok(self.inner - other.inner)
        } else {
            Err(Error::NegativeDuration)
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
    pub const fn is_leap_year(&self) -> bool {
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

/// A point in time with a fixed UTC offset.
///
/// Internally stores the "offset-adjusted timestamp" to efficiently obtain components such as year, month, and day.
/// This value is **not public**; external code can only obtain UTC time through semantically clear methods (e.g., `utc()`).
#[derive(Debug, Clone, Copy)]
pub struct OffsetDateTime {
    inner: UtcDateTime, // offset-adjusted displayed time
    offset: Offset,
}

impl OffsetDateTime {
    /// Constructs a local time from the given UTC time and offset.
    ///
    /// The internal `inner` is stored as `utc + offset`.
    /// Returns an error if the result is earlier than the Unix epoch or overflows.
    pub fn from_utc(utc: UtcDateTime, offset: Offset) -> Result<Self> {
        let utc_secs = utc.timestamp() as i64;
        let local_secs = utc_secs
            .checked_add(offset.seconds() as i64)
            .ok_or(Error::Overflow)?;
        if local_secs < 0 {
            return Err(Error::Overflow);
        }
        Ok(OffsetDateTime {
            inner: UtcDateTime::from_timestamp(local_secs as u64),
            offset,
        })
    }

    /// Converts to the corresponding UTC time.
    ///
    /// That is, `inner - offset`.
    /// Returns an error if the result is earlier than the Unix epoch.
    pub fn utc(&self) -> Result<UtcDateTime> {
        let local_secs = self.inner.timestamp() as i64;
        let utc_secs = local_secs
            .checked_sub(self.offset.seconds() as i64)
            .ok_or(Error::Overflow)?;
        if utc_secs < 0 {
            return Err(Error::Overflow);
        }
        Ok(UtcDateTime::from_timestamp(utc_secs as u64))
    }

    #[cfg(debug_assertions)]
    /// Constructs a local time from local time components and a time zone offset.
    ///
    /// First validates the local time, then computes UTC as `local - offset`.
    /// Returns an error if UTC is earlier than the epoch.
    pub fn new(
        year: u16,
        month: Month,
        day: u8,
        hour: u8,
        minute: u8,
        second: u8,
        offset: Offset,
    ) -> Result<Self> {
        let dt = UtcDateTime::new(year, month, day, hour, minute, second)?;
        let utc_secs = dt.timestamp() as i64 - offset.seconds() as i64;
        if utc_secs < 0 {
            return Err(Error::Overflow);
        }
        Ok(OffsetDateTime { inner: dt, offset })
    }

    /// Automatically reads the system time zone and converts to local time.
    pub fn now_local() -> Result<Self> {
        let utc_now = UtcDateTime::now();
        let offset = Offset::local_offset();
        Self::from_utc(utc_now, offset)
    }

    /// Returns the UTC timestamp (seconds).
    pub const fn timestamp(&self) -> u64 {
        (self.inner.timestamp() as i64 - self.offset.seconds() as i64) as u64
    }

    /// Returns the time zone offset.
    #[inline]
    pub fn offset(&self) -> Offset {
        self.offset
    }

    /// Returns the year.
    #[inline]
    pub fn year(&self) -> u16 {
        self.inner.year()
    }

    /// Returns the month.
    #[inline]
    pub fn month(&self) -> Month {
        self.inner.month()
    }

    /// Returns the day.
    #[inline]
    pub fn day(&self) -> u8 {
        self.inner.day()
    }

    /// Returns the hour.
    #[inline]
    pub fn hour(&self) -> u8 {
        self.inner.hour()
    }

    /// Returns the minute.
    #[inline]
    pub fn minute(&self) -> u8 {
        self.inner.minute()
    }

    /// Returns the second.
    #[inline]
    pub fn second(&self) -> u8 {
        self.inner.second()
    }

    /// Returns the year, month, day, hour, minute, and second.
    #[inline]
    pub fn ymd_hms(&self) -> (u16, Month, u8, u8, u8, u8) {
        self.inner.ymd_hms()
    }

    /// Adds the specified number of seconds, keeping the offset unchanged.
    pub fn add_seconds(&self, secs: u64) -> Result<Self> {
        let new_inner = self.inner.add_seconds(secs)?;
        Ok(OffsetDateTime {
            inner: new_inner,
            offset: self.offset,
        })
    }

    /// Adds the specified number of minutes, keeping the offset unchanged.
    pub fn add_minutes(&self, mins: u64) -> Result<Self> {
        let new_inner = self.inner.add_minutes(mins)?;
        Ok(OffsetDateTime {
            inner: new_inner,
            offset: self.offset,
        })
    }

    /// Adds the specified number of hours, keeping the offset unchanged.
    pub fn add_hours(&self, hours: u64) -> Result<Self> {
        let new_inner = self.inner.add_hours(hours)?;
        Ok(OffsetDateTime {
            inner: new_inner,
            offset: self.offset,
        })
    }

    /// Adds the specified number of days, keeping the offset unchanged.
    pub fn add_days(&self, days: u64) -> Result<Self> {
        let new_inner = self.inner.add_days(days)?;
        Ok(OffsetDateTime {
            inner: new_inner,
            offset: self.offset,
        })
    }

    /// Subtracts the specified number of seconds, keeping the offset unchanged.
    pub fn sub_seconds(&self, secs: u64) -> Result<Self> {
        let new_inner = self.inner.sub_seconds(secs)?;
        Ok(OffsetDateTime {
            inner: new_inner,
            offset: self.offset,
        })
    }

    /// Subtracts the specified number of minutes, keeping the offset unchanged.
    pub fn sub_minutes(&self, mins: u64) -> Result<Self> {
        let new_inner = self.inner.sub_minutes(mins)?;
        Ok(OffsetDateTime {
            inner: new_inner,
            offset: self.offset,
        })
    }

    /// Subtracts the specified number of hours, keeping the offset unchanged.
    pub fn sub_hours(&self, hours: u64) -> Result<Self> {
        let new_inner = self.inner.sub_hours(hours)?;
        Ok(OffsetDateTime {
            inner: new_inner,
            offset: self.offset,
        })
    }

    /// Subtracts the specified number of days, keeping the offset unchanged.
    pub fn sub_days(&self, days: u64) -> Result<Self> {
        let new_inner = self.inner.sub_days(days)?;
        Ok(OffsetDateTime {
            inner: new_inner,
            offset: self.offset,
        })
    }

    /// Formats as an RFC 3339 string with offset, e.g., `2025-06-15T14:30:45+08:00`.
    pub fn to_rfc3339(&self) -> String {
        let (year, month, day) = self.inner.ymd();
        let (hour, minute, second) = self.inner.hms();
        let offset_sign = if self.offset.seconds() >= 0 { '+' } else { '-' };
        let offset_abs = self.offset.seconds().unsigned_abs();
        let offset_h = offset_abs / 3600;
        let offset_m = (offset_abs % 3600) / 60;
        format!(
            "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}{}{:02}:{:02}",
            year, month as u8, day, hour, minute, second, offset_sign, offset_h, offset_m
        )
    }
}

impl From<OffsetDateTime> for UtcDateTime {
    /// Conversion from `LocalDateTime` may fail, so this implementation will panic.
    /// In real projects, it is recommended to use the `to_utc()` method to obtain a `Result`.
    fn from(local: OffsetDateTime) -> Self {
        local.utc().expect("LocalDateTime to UtcDateTime overflow")
    }
}

impl fmt::Display for OffsetDateTime {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_rfc3339())
    }
}

impl PartialEq for OffsetDateTime {
    fn eq(&self, other: &Self) -> bool {
        self.utc().ok() == other.utc().ok()
    }
}

impl Eq for OffsetDateTime {}

impl PartialOrd for OffsetDateTime {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        let a = self.utc().ok()?;
        let b = other.utc().ok()?;
        Some(a.cmp(&b))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ----- UtcDateTime tests (existing + additional) -----
    #[test]
    fn test_utc_create_and_components() {
        let dt = UtcDateTime::new(2024, Month::March, 15, 10, 30, 45).unwrap();
        assert_eq!(dt.year(), 2024);
        assert_eq!(dt.month(), Month::March);
        assert_eq!(dt.day(), 15);
        assert_eq!(dt.hour(), 10);
        assert_eq!(dt.minute(), 30);
        assert_eq!(dt.second(), 45);
    }

    #[test]
    fn test_utc_arithmetic() {
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
    fn test_utc_ymd_hms() {
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
    fn test_utc_leap_year_boundary() {
        let dt1 = UtcDateTime::new(2024, Month::February, 29, 0, 0, 0).unwrap();
        assert_eq!(dt1.day(), 29);

        let result = UtcDateTime::new(2023, Month::February, 29, 0, 0, 0);
        assert!(result.is_err());
    }

    #[test]
    fn test_utc_roundtrip_conversion() {
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
    fn test_utc_leap_second_rejection() {
        let result = UtcDateTime::new(2025, Month::June, 30, 23, 59, 60);
        assert!(result.is_err());
    }

    #[test]
    fn test_utc_start_end_of_day() {
        let dt = UtcDateTime::new(2025, Month::June, 15, 14, 30, 45).unwrap();
        let start = dt.start_of_day();
        assert_eq!(start.hms(), (0, 0, 0));
        assert_eq!(start.day(), 15);

        let end = dt.end_of_day();
        assert_eq!(end.hms(), (23, 59, 59));
        assert_eq!(end.day(), 15);
    }

    #[test]
    fn test_utc_comparison_methods() {
        let dt1 = UtcDateTime::from_timestamp(1000);
        let dt2 = UtcDateTime::from_timestamp(2000);
        let dt3 = UtcDateTime::from_timestamp(3000);

        assert!(dt1.is_before(&dt2));
        assert!(dt3.is_after(&dt2));
        assert!(dt2.is_between_inclusive(&dt1, &dt3));
        assert!(!dt1.is_between_exclusive(&dt1, &dt3));
    }

    #[test]
    fn test_utc_duration_since() {
        let early = UtcDateTime::from_timestamp(1000);
        let later = UtcDateTime::from_timestamp(5000);
        let diff = later.duration_since(&early).unwrap();
        assert_eq!(diff, Duration::from_secs(4000));
        assert!(early.duration_since(&later).is_err());
    }

    // ----- LocalDateTime tests -----
    // Helper to create a fixed offset (e.g., UTC+8)
    fn offset_plus_8() -> Offset {
        Offset { seconds: 8 * 3600 }
    }

    fn offset_minus_5() -> Offset {
        Offset { seconds: -5 * 3600 }
    }

    #[test]
    fn test_offset_new_and_components() {
        let offset = offset_plus_8();
        let local = OffsetDateTime::new(2025, Month::June, 15, 14, 30, 45, offset).unwrap();
        assert_eq!(local.year(), 2025);
        assert_eq!(local.month(), Month::June);
        assert_eq!(local.day(), 15);
        assert_eq!(local.hour(), 14);
        assert_eq!(local.minute(), 30);
        assert_eq!(local.second(), 45);
        assert_eq!(local.offset(), offset);
    }

    #[test]
    fn test_offset_to_utc_and_back() {
        let offset = offset_plus_8();
        let local = OffsetDateTime::new(2025, Month::June, 15, 14, 30, 0, offset).unwrap();
        let utc = local.utc().unwrap();
        // UTC should be 8 hours behind
        assert_eq!(utc.hour(), 6);
        assert_eq!(utc.day(), 15);

        // round trip
        let local2 = OffsetDateTime::from_utc(utc, offset).unwrap();
        assert_eq!(local, local2);
    }

    #[test]
    fn test_offset_negative_offset() {
        let offset = offset_minus_5(); // UTC-5
        let local = OffsetDateTime::new(2025, Month::June, 15, 1, 0, 0, offset).unwrap();
        let utc = local.utc().unwrap();
        assert_eq!(utc.hour(), 6);
    }

    #[test]
    fn test_offset_before_epoch_rejected() {
        // UTC 1970-01-01T00:00:00 + positive offset => local time is ok,
        // but if local time is 1970-01-01T00:00:00 + a negative offset that pushes UTC before epoch
        let offset = offset_plus_8(); // +8
        // Local 1970-01-01 00:00:00 => UTC would be 1969-12-31T16:00:00 (< epoch)
        let result = OffsetDateTime::new(1970, Month::January, 1, 0, 0, 0, offset);
        assert!(result.is_err());
    }

    #[test]
    fn test_offset_arithmetic_keep_offset() {
        let offset = offset_plus_8();
        let local = OffsetDateTime::new(2025, Month::June, 15, 12, 0, 0, offset).unwrap();
        let later = local.add_hours(5).unwrap();
        assert_eq!(later.hour(), 17);
        assert_eq!(later.offset(), offset);

        let earlier = local.sub_minutes(30).unwrap();
        assert_eq!(earlier.hour(), 11);
        assert_eq!(earlier.minute(), 30);
        assert_eq!(earlier.offset(), offset);
    }

    #[test]
    fn test_offset_comparison() {
        let offset = offset_plus_8();
        let a = OffsetDateTime::new(2025, Month::June, 15, 10, 0, 0, offset).unwrap();
        let b = OffsetDateTime::new(2025, Month::June, 15, 18, 0, 0, offset).unwrap();
        assert!(a < b);
        assert!(b > a);
    }

    #[test]
    fn test_offset_display_format() {
        let offset = offset_plus_8();
        let local = OffsetDateTime::new(2025, Month::June, 15, 14, 30, 45, offset).unwrap();
        let s = local.to_rfc3339();
        // Milliseconds are allowed to be omitted; the format is like 2025-06-15T14:30:45+08:00
        assert!(s.contains("2025-06-15T14:30:45"));
        assert!(s.contains("+08:00"));
        assert_eq!(format!("{}", local), s);
    }

    #[test]
    fn test_offset_from_to_utc_edge() {
        let utc = UtcDateTime::from_timestamp(0); // epoch
        let offset = offset_plus_8();
        let local = OffsetDateTime::from_utc(utc, offset).unwrap();
        assert_eq!(local.hour(), 8);
        assert_eq!(local.day(), 1);

        let utc_back = local.utc().unwrap();
        assert_eq!(utc_back.timestamp(), 0);
    }

    #[test]
    fn test_offset_overflow_handling() {
        let offset = offset_plus_8();
        let local = OffsetDateTime::new(2025, Month::June, 15, 10, 0, 0, offset).unwrap();
        // add a huge number of seconds should overflow
        let result = local.add_seconds(u64::MAX);
        assert!(result.is_err());
    }

    #[test]
    fn test_offset_different_offset_comparison() {
        // Times that represent the same UTC instant but different local times and offsets
        let utc = UtcDateTime::new(2025, Month::June, 15, 12, 0, 0).unwrap();
        let local_plus8 = OffsetDateTime::from_utc(utc, offset_plus_8()).unwrap();
        let local_minus5 = OffsetDateTime::from_utc(utc, offset_minus_5()).unwrap();
        assert_eq!(local_plus8, local_minus5); // Equal based on UTC
        assert!(
            local_plus8.year() != local_minus5.year() || local_plus8.hour() != local_minus5.hour()
        );
    }
}
