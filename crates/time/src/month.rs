use super::{error::Error, math::is_leap_year};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Month {
    January = 1,
    February = 2,
    March = 3,
    April = 4,
    May = 5,
    June = 6,
    July = 7,
    August = 8,
    September = 9,
    October = 10,
    November = 11,
    December = 12,
}

impl Month {
    #[inline]
    pub(super) const fn days_in_month(self, year: u16) -> u8 {
        match self {
            Month::January
            | Month::March
            | Month::May
            | Month::July
            | Month::August
            | Month::October
            | Month::December => 31,
            Month::April | Month::June | Month::September | Month::November => 30,
            Month::February => {
                if is_leap_year(year) {
                    29
                } else {
                    28
                }
            }
        }
    }

    /// Internal use: creates a Month from a valid u8 value (without validation)
    ///
    /// # Safety
    /// The caller must ensure that `value` is within the range 1..=12.
    #[inline]
    pub(super) const fn from_u8_unchecked(value: u8) -> Self {
        match value {
            1 => Month::January,
            2 => Month::February,
            3 => Month::March,
            4 => Month::April,
            5 => Month::May,
            6 => Month::June,
            7 => Month::July,
            8 => Month::August,
            9 => Month::September,
            10 => Month::October,
            11 => Month::November,
            _ => Month::December,
        }
    }

    #[inline]
    pub const fn as_u8(self) -> u8 {
        self as u8
    }
}

impl TryFrom<u8> for Month {
    type Error = Error;

    #[inline]
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(Month::January),
            2 => Ok(Month::February),
            3 => Ok(Month::March),
            4 => Ok(Month::April),
            5 => Ok(Month::May),
            6 => Ok(Month::June),
            7 => Ok(Month::July),
            8 => Ok(Month::August),
            9 => Ok(Month::September),
            10 => Ok(Month::October),
            11 => Ok(Month::November),
            12 => Ok(Month::December),
            _ => Err(Error::InvalidMonth(value)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn month_days_handle_leap_years_and_centuries() {
        assert_eq!(Month::February.days_in_month(2024), 29);
        assert_eq!(Month::February.days_in_month(2023), 28);
        assert_eq!(Month::February.days_in_month(2000), 29);
        assert_eq!(Month::February.days_in_month(1900), 28);
        assert_eq!(Month::February.days_in_month(2100), 28);

        for month in [Month::April, Month::June, Month::September, Month::November] {
            assert_eq!(month.days_in_month(2024), 30, "{month:?} has 30 days");
        }

        for month in [
            Month::January,
            Month::March,
            Month::May,
            Month::July,
            Month::August,
            Month::October,
            Month::December,
        ] {
            assert_eq!(month.days_in_month(2024), 31, "{month:?} has 31 days");
        }
    }

    #[test]
    fn month_try_from_rejects_zero_and_thirteen() {
        assert!(matches!(Month::try_from(0), Err(Error::InvalidMonth(0))));
        assert!(matches!(Month::try_from(13), Err(Error::InvalidMonth(13))));
        assert_eq!(Month::try_from(1).unwrap(), Month::January);
        assert_eq!(Month::try_from(12).unwrap(), Month::December);
        assert_eq!(Month::December.as_u8(), 12);
    }
}
