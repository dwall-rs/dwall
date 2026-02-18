use crate::utils::datetime::{
    consts::{
        CUMULATIVE_DAYS, DAYS_PER_4_YEARS, DAYS_PER_100_YEARS, DAYS_PER_400_YEARS, DAYS_PER_YEAR,
        MONTH_DIVISOR, UNIX_EPOCH_TO_MARCH_1_YEAR_0,
    },
    error::DateTimeResult,
    month::Month,
};

#[inline]
pub(super) const fn is_leap_year(year: u16) -> bool {
    // Rust 1.87.0+ supports is_multiple_of
    (year.is_multiple_of(4) && !year.is_multiple_of(100)) || year.is_multiple_of(400)
}

/// Converts a Gregorian calendar date into the number of days since the Unix epoch (1970-01-01).
pub(super) fn days_since_epoch(year: u16, month: Month, day: u8) -> DateTimeResult<u32> {
    let years_since_epoch = year as u32 - 1970;

    // Compute total days from complete years
    let leap_years = count_leap_years(1970, year);
    let mut days = years_since_epoch * DAYS_PER_YEAR + leap_years;

    // Use lookup table for fast cumulative days up to the given month
    let is_leap = is_leap_year(year);
    let table = &CUMULATIVE_DAYS[is_leap as usize];
    days += table[(month as u8 - 1) as usize];

    // Add the day of the month (convert to 0-based)
    days += day as u32 - 1;
    Ok(days)
}

/// Counts the number of leap years in the half-open interval [start, end).
///
/// This function calculates how many leap years occur from `start` (inclusive)
/// to `end` (exclusive). For example, `count_leap_years(2024, 2025)` returns 1
/// because 2024 is a leap year and is included in the range.
///
/// The algorithm uses the standard Gregorian calendar leap year rules:
/// - A year is a leap year if divisible by 4
/// - Except if divisible by 100, unless also divisible by 400
///
/// Then computes: leap_years_up_to(end - 1) - leap_years_up_to(start - 1)
///
/// # Arguments
/// * `start` - Starting year (inclusive), must be >= 1970 per requirements
/// * `end` - Ending year (exclusive), must be >= start
#[inline]
fn count_leap_years(start: u16, end: u16) -> u32 {
    let count = |year: u16| -> u32 {
        let y = year as u32;
        (y / 4) - (y / 100) + (y / 400)
    };
    count(end - 1) - count(start - 1)
}

/// Converts the number of days since the Unix epoch into a Gregorian calendar date.
///
/// This algorithm is based on Howard Hinnant's efficient date algorithm.
/// The input `days` represents the number of days elapsed since the Unix epoch (1970-01-01). See: <https://howardhinnant.github.io/date_algorithms.html>
///
/// The algorithm shifts the year start to March 1st, placing the leap day at the end of the year,
/// which simplifies leap year handling.
///
/// Returns: (year, month, day)
pub(super) const fn days_to_ymd(days: u32) -> (u16, Month, u8) {
    // Convert input days to days since March 1 of year 0
    let days_from_base = days + UNIX_EPOCH_TO_MARCH_1_YEAR_0;

    // Compute the number of full 400-year cycles (eras) and remaining days within the current era
    let era = days_from_base / DAYS_PER_400_YEARS;
    let day_of_era = days_from_base % DAYS_PER_400_YEARS;

    // Compute the year within the current 400-year era (0–399)
    // This accounts for leap year rules: leap every 4 years, except centuries, unless divisible by 400
    let year_of_era = (day_of_era - day_of_era / (DAYS_PER_4_YEARS - 1)
        + day_of_era / DAYS_PER_100_YEARS
        - day_of_era / (DAYS_PER_400_YEARS - 1))
        / DAYS_PER_YEAR;

    // Compute the year counting from year 0 with March 1 as the start of the year
    let year_starting_march = year_of_era + era * 400;

    // Compute the day of the year (0-based, where March 1 = 0)
    let day_of_year =
        day_of_era - (DAYS_PER_YEAR * year_of_era + year_of_era / 4 - year_of_era / 100);

    // Map day-of-year to month index (March = 0, April = 1, ..., February = 11)
    let month_index = (5 * day_of_year + 2) / MONTH_DIVISOR;

    // Compute the day of the month (1-based)
    let day = day_of_year - (MONTH_DIVISOR * month_index + 2) / 5 + 1;

    // Convert to standard calendar month (March–December = 3–12; January–February = 1–2)
    let month = if month_index < 10 {
        month_index + 3 // March–December
    } else {
        month_index - 9 // January–February
    };

    // Adjust year: January and February belong to the next calendar year
    let year = year_starting_march + if month <= 2 { 1 } else { 0 };

    (
        year as u16,
        Month::from_u8_unchecked(month as u8),
        day as u8,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_leap_year() {
        // Standard leap years
        assert!(is_leap_year(2000));
        assert!(is_leap_year(2004));
        assert!(is_leap_year(2024));

        // Non-leap years
        assert!(!is_leap_year(1900));
        assert!(!is_leap_year(2100));
        assert!(!is_leap_year(2023));
        assert!(!is_leap_year(2025));

        // Edge cases
        assert!(is_leap_year(4)); // First leap year
        assert!(!is_leap_year(1)); // Year 1 is not leap
        assert!(is_leap_year(400)); // Divisible by 400
    }

    #[test]
    fn test_count_leap_years() {
        // From 1970 to 1970 → 0
        assert_eq!(count_leap_years(1970, 1970), 0);
        // From 1970 to 1973 → includes 1972 → 1
        assert_eq!(count_leap_years(1970, 1973), 1);
        // From 1970 to 1977 → includes 1972, 1976 → 2
        assert_eq!(count_leap_years(1970, 1977), 2);
        // From 1970 to 2001 → includes 1972,76,80,84,88,92,96,2000 → 8
        assert_eq!(count_leap_years(1970, 2001), 8);
        // Across century: 1896 to 1905 → only 1896 and 1904 (1900 not leap)
        assert_eq!(count_leap_years(1896, 1905), 2);
    }

    #[test]
    fn test_days_to_ymd() {
        let test_cases = vec![
            // Basic epoch
            (0, (1970, Month::January, 1)),
            // End of January
            (30, (1970, Month::January, 31)),
            // End of February (non-leap)
            (58, (1970, Month::February, 28)),
            // March 1
            (59, (1970, Month::March, 1)),
            // Leap year: 2000
            (10957, (2000, Month::January, 1)),   // 2000-01-01
            (11016, (2000, Month::February, 29)), // Feb 29, 2000
            (11017, (2000, Month::March, 1)),
            // Non-leap century: 2100
            (47540, (2100, Month::February, 28)),
            (47541, (2100, Month::March, 1)),
            // Far future
            (100000, (2243, Month::October, 17)),
            // Year 2038 boundary
            (29220, (2050, Month::January, 1)),
        ];

        for (days, expected) in test_cases {
            let (year, month, day) = days_to_ymd(days);
            assert_eq!((year, month, day), expected, "Failed for days = {}", days);
        }
    }

    #[test]
    fn test_days_since_epoch() {
        let test_cases = vec![
            (1970, Month::January, 1, 0),
            (1970, Month::January, 31, 30),
            (1970, Month::February, 28, 58),
            (1970, Month::March, 1, 59),
            (2000, Month::January, 1, 10957),
            (2000, Month::February, 29, 11016),
            (2000, Month::March, 1, 11017),
            (2100, Month::February, 28, 47540),
            (2100, Month::March, 1, 47541),
            (2243, Month::October, 17, 100000),
        ];

        for (year, month, day, expected_days) in test_cases {
            let days = days_since_epoch(year, month, day).unwrap();
            assert_eq!(
                days, expected_days,
                "Failed for {}-{:?}-{}",
                year, month, day
            );
        }
    }

    #[test]
    fn test_round_trip_consistency() {
        let dates = vec![
            (1970, Month::January, 1),
            (1970, Month::December, 31),
            (1999, Month::December, 31),
            (2000, Month::February, 29),
            (2001, Month::March, 1),
            (2024, Month::February, 29),
            (2100, Month::February, 28),
            (2200, Month::June, 15),
        ];

        for (year, month, day) in dates {
            let days = days_since_epoch(year, month, day).unwrap();
            let (y, m, d) = days_to_ymd(days);
            assert_eq!(
                (y, m, d),
                (year, month, day),
                "Round-trip failed for {}-{:?}-{}",
                year,
                month,
                day
            );
        }
    }

    #[test]
    fn test_february_29_edge_cases() {
        // Valid leap day
        let days = days_since_epoch(2024, Month::February, 29).unwrap();
        let (y, m, d) = days_to_ymd(days);
        assert_eq!((y, m, d), (2024, Month::February, 29));
    }
}
