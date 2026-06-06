use serde::{Deserialize, Serialize};
use time::UtcDateTime;

use crate::Position;

mod constants {
    pub(super) const EPOCH_J2000: f64 = 2451545.0;

    pub(super) const JULIAN_CENTURY_DAYS: f64 = 36525.0;

    pub(super) const EARTH_ROTATION_RATE: f64 = 15.0;

    pub(super) const HOURS_PER_DAY: f64 = 24.0;

    pub(super) const MINUTES_PER_HOUR: f64 = 60.0;

    pub(super) const SECONDS_PER_HOUR: f64 = 3600.0;

    pub(super) const MINUTES_PER_DAY: f64 = 1440.0;

    pub(super) const SECONDS_PER_DAY: f64 = 86400.0;

    pub(super) const ATMOSPHERIC_REFRACTION_MAX: f64 = 0.575;
}

struct SolarContext {
    latitude_rad: f64,
    declination_rad: f64,
    hour_angle_rad: f64,
}

impl SolarContext {
    fn new(position: &Position, date_time: &UtcDateTime) -> Self {
        let t = SolarCalc::julian_century_t(date_time);

        let epsilon_deg = SolarCalc::earth_axial_tilt(t);

        let declination_deg = SolarCalc::solar_declination(t, epsilon_deg);

        let eot_hours = SolarCalc::solar_time_correction(t, epsilon_deg);

        let decimal_hours = SolarCalc::decimal_hours(date_time);
        let hour_angle_deg = SolarCalc::hour_angle(decimal_hours, position.longitude(), eot_hours);

        Self {
            latitude_rad: position.latitude().to_radians(),
            declination_rad: declination_deg.to_radians(),
            hour_angle_rad: hour_angle_deg.to_radians(),
        }
    }
}

struct SolarCalc;

impl SolarCalc {
    fn julian_day(date_time: &UtcDateTime) -> f64 {
        let (year, month, day, hour, minute, second) = date_time.ymd_hms();

        let y = year as i32;
        let m = month as i32;
        let d = day as i32;

        let a = (14 - m) / 12;
        let y2 = y + 4800 - a;
        let m2 = m + 12 * a - 3;
        let jdn = d + (153 * m2 + 2) / 5 + 365 * y2 + y2 / 4 - y2 / 100 + y2 / 400 - 32045;

        jdn as f64 - 0.5
            + hour as f64 / constants::HOURS_PER_DAY
            + minute as f64 / constants::MINUTES_PER_DAY
            + second as f64 / constants::SECONDS_PER_DAY
    }

    fn julian_century_t(date_time: &UtcDateTime) -> f64 {
        let jd = Self::julian_day(date_time);
        (jd - constants::EPOCH_J2000) / constants::JULIAN_CENTURY_DAYS
    }

    #[inline]
    fn earth_axial_tilt(t: f64) -> f64 {
        23.43929111 - 0.0130042 * t - 0.00000164 * t.powi(2) + 0.000000503 * t.powi(3)
    }

    fn solar_ecliptic_longitude(t: f64) -> f64 {
        let l0 = 280.4664567 + 36000.7698278 * t + 0.0003032028 * t.powi(2);

        let l1 = 1.914602 - 0.004817 * t - 0.000014 * t.powi(2);
        let l2 = 0.019993 - 0.000101 * t;
        let l3 = 0.000289;

        let m = (357.5291092 + 35999.0502909 * t).to_radians();

        let lambda = l0 + l1 * m.sin() + l2 * (2.0 * m).sin() + l3 * (3.0 * m).sin();

        ((lambda % 360.0) + 360.0) % 360.0
    }

    fn solar_declination(t: f64, epsilon_deg: f64) -> f64 {
        let lambda = Self::solar_ecliptic_longitude(t);
        let sin_delta = epsilon_deg.to_radians().sin() * lambda.to_radians().sin();
        sin_delta.asin().to_degrees()
    }

    fn solar_time_correction(t: f64, epsilon_deg: f64) -> f64 {
        let m_deg = 357.52911 + 35999.05029 * t - 0.0001537 * t.powi(2);
        let m = m_deg.to_radians();

        let l0_deg = 280.46646 + 36000.76983 * t + 0.0003032 * t.powi(2);

        let epsilon = epsilon_deg.to_radians();

        let c = (1.914602 - 0.004817 * t - 0.000014 * t.powi(2)) * m.sin()
            + (0.019993 - 0.000101 * t) * (2.0 * m).sin()
            + 0.000289 * (3.0 * m).sin();

        let lambda = (l0_deg + c).to_radians();

        let alpha = (lambda.sin() * epsilon.cos()).atan2(lambda.cos());

        let mut diff = (l0_deg - alpha.to_degrees() - 0.0057183) % 360.0;
        if diff > 180.0 {
            diff -= 360.0;
        } else if diff < -180.0 {
            diff += 360.0;
        }

        diff * 4.0 / constants::MINUTES_PER_HOUR
    }

    #[inline]
    fn decimal_hours(date_time: &UtcDateTime) -> f64 {
        date_time.hour() as f64
            + date_time.minute() as f64 / constants::MINUTES_PER_HOUR
            + date_time.second() as f64 / constants::SECONDS_PER_HOUR
    }

    fn hour_angle(decimal_hours: f64, longitude: f64, eot_hours: f64) -> f64 {
        let local_apparent_time =
            decimal_hours + (longitude / constants::EARTH_ROTATION_RATE) + eot_hours;

        let raw_angle = constants::EARTH_ROTATION_RATE * (local_apparent_time - 12.0);

        let ha = ((raw_angle % 360.0) + 360.0) % 360.0;

        if ha > 180.0 { ha - 360.0 } else { ha }
    }

    fn atmospheric_refraction(apparent_altitude: f64) -> f64 {
        if apparent_altitude < -0.5 {
            return 0.0;
        }

        let tan_h = (apparent_altitude + 7.31 / (apparent_altitude + 4.4))
            .to_radians()
            .tan();

        let correction_deg = (1.0 / tan_h) / 60.0;
        debug_assert!(
            correction_deg >= 0.0,
            "Atmospheric refraction correction must not be negative (elevation = {}°)",
            apparent_altitude
        );

        correction_deg.clamp(0.0, constants::ATMOSPHERIC_REFRACTION_MAX)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct SolarPosition {
    altitude: f64,
    azimuth: f64,
}

impl SolarPosition {
    #[inline]
    pub(crate) fn new(position: &Position, date_time: &UtcDateTime) -> Self {
        let ctx = SolarContext::new(position, date_time);
        Self {
            altitude: Self::altitude_from_context(&ctx),
            azimuth: Self::azimuth_from_context(&ctx),
        }
    }

    fn altitude_from_context(ctx: &SolarContext) -> f64 {
        let sin_alt = ctx.latitude_rad.sin() * ctx.declination_rad.sin()
            + ctx.latitude_rad.cos() * ctx.declination_rad.cos() * ctx.hour_angle_rad.cos();

        let true_altitude = sin_alt.asin().to_degrees();
        true_altitude + SolarCalc::atmospheric_refraction(true_altitude)
    }

    fn azimuth_from_context(ctx: &SolarContext) -> f64 {
        let sin_h = ctx.hour_angle_rad.sin();
        let cos_h = ctx.hour_angle_rad.cos();
        let sin_phi = ctx.latitude_rad.sin();
        let cos_phi = ctx.latitude_rad.cos();
        let tan_delta = ctx.declination_rad.tan();

        let denominator = cos_h * sin_phi - tan_delta * cos_phi;

        let azimuth_deg = (sin_h.atan2(denominator).to_degrees() + 360.0) % 360.0;

        (azimuth_deg + 180.0) % 360.0
    }

    pub(crate) fn altitude(&self) -> f64 {
        self.altitude
    }

    pub(crate) fn azimuth(&self) -> f64 {
        self.azimuth
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct SolarAngle {
    index: u8,
    altitude: f64,
    azimuth: f64,
}

impl SolarAngle {
    pub(crate) fn index(&self) -> u8 {
        self.index
    }

    pub(crate) fn altitude(&self) -> f64 {
        self.altitude
    }

    pub(crate) fn azimuth(&self) -> f64 {
        self.azimuth
    }
}
