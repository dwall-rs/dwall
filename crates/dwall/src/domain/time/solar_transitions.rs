use crate::{Position, utils::datetime::UtcDateTime};

use super::solar_calculator::SunPosition;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PolarState {
    Normal,
    PolarDay,
    PolarNight,
}

#[derive(Debug, Clone, Copy)]
pub struct SolarTransitions {
    pub sunrise: Option<UtcDateTime>,
    pub sunset: Option<UtcDateTime>,
    pub solar_noon: UtcDateTime,
    pub polar_state: PolarState,
}

struct FindCrossingBrackets(
    Option<(UtcDateTime, UtcDateTime)>,
    Option<(UtcDateTime, UtcDateTime)>,
);

impl SolarTransitions {
    pub fn calculate(position: &Position, day_start: UtcDateTime) -> Self {
        let lat = position.latitude();
        let lon = position.longitude();

        let polar_state = Self::detect_polar_state(lat, lon, day_start);
        let solar_noon = Self::find_solar_noon(lat, lon, day_start);

        if polar_state != PolarState::Normal {
            return Self {
                sunrise: None,
                sunset: None,
                solar_noon,
                polar_state,
            };
        }

        let FindCrossingBrackets(sunrise_bracket, sunset_bracket) =
            Self::find_crossing_brackets(lat, lon, day_start);

        Self {
            sunrise: sunrise_bracket.and_then(|(lo, hi)| Self::bisect(lat, lon, lo, hi, true)),
            sunset: sunset_bracket.and_then(|(lo, hi)| Self::bisect(lat, lon, lo, hi, false)),
            solar_noon,
            polar_state,
        }
    }

    fn detect_polar_state(lat: f64, lon: f64, day_start: UtcDateTime) -> PolarState {
        let all_above = (0..12).all(|i| {
            SunPosition::new(lat, lon, day_start.add_seconds_unchecked(i * 7200)).altitude() > 0.0
        });
        let all_below = (0..12).all(|i| {
            SunPosition::new(lat, lon, day_start.add_seconds_unchecked(i * 7200)).altitude() < 0.0
        });
        match (all_above, all_below) {
            (true, _) => PolarState::PolarDay,
            (_, true) => PolarState::PolarNight,
            _ => PolarState::Normal,
        }
    }

    fn find_solar_noon(lat: f64, lon: f64, day_start: UtcDateTime) -> UtcDateTime {
        let mut lo = day_start.add_seconds_unchecked(6 * 3600);
        let mut hi = day_start.add_seconds_unchecked(18 * 3600);
        for _ in 0..40 {
            if hi.timestamp() - lo.timestamp() <= 1 {
                break;
            }
            let m1 =
                UtcDateTime::from_timestamp(lo.timestamp() + (hi.timestamp() - lo.timestamp()) / 3);
            let m2 =
                UtcDateTime::from_timestamp(hi.timestamp() - (hi.timestamp() - lo.timestamp()) / 3);
            if SunPosition::new(lat, lon, m1).altitude() < SunPosition::new(lat, lon, m2).altitude()
            {
                lo = m1;
            } else {
                hi = m2;
            }
        }
        UtcDateTime::from_timestamp((lo.timestamp() + hi.timestamp()) / 2)
    }

    fn find_crossing_brackets(lat: f64, lon: f64, day_start: UtcDateTime) -> FindCrossingBrackets {
        let mut prev_t = day_start;
        let mut prev_alt = SunPosition::new(lat, lon, prev_t).altitude();
        let mut sunrise_bracket = None;
        let mut sunset_bracket = None;

        for i in 1..=144usize {
            let t = day_start.add_seconds_unchecked(i as u64 * 600);
            let alt = SunPosition::new(lat, lon, t).altitude();
            if prev_alt < 0.0 && alt > 0.0 {
                sunrise_bracket = Some((prev_t, t));
            } else if prev_alt > 0.0 && alt < 0.0 {
                sunset_bracket = Some((prev_t, t));
            }
            if sunrise_bracket.is_some() && sunset_bracket.is_some() {
                break;
            }
            prev_alt = alt;
            prev_t = t;
        }
        FindCrossingBrackets(sunrise_bracket, sunset_bracket)
    }

    fn bisect(
        lat: f64,
        lon: f64,
        mut lo: UtcDateTime,
        mut hi: UtcDateTime,
        rising: bool,
    ) -> Option<UtcDateTime> {
        for _ in 0..20 {
            if hi.timestamp() - lo.timestamp() <= 1 {
                break;
            }
            let mid = UtcDateTime::from_timestamp((lo.timestamp() + hi.timestamp()) / 2);
            let alt = SunPosition::new(lat, lon, mid).altitude();
            if rising == (alt < 0.0) {
                lo = mid;
            } else {
                hi = mid;
            }
        }
        Some(lo)
    }
}
