//! Wallpaper selection and application abstraction.

use std::path::Path;

use crate::{DwallResult, SolarAngle};

/// Trait for setting wallpapers on monitors and lock screen.
///
/// Implemented by the platform wallpaper backend (Windows IDesktopWallpaper).
pub trait WallpaperProvider {
    /// Sets the wallpaper for a specific monitor
    fn set_monitor_wallpaper(&self, monitor_id: &str, image_path: &Path) -> DwallResult<()>;

    /// Sets the lock screen image
    fn set_lock_screen_image(&self, image_path: &Path) -> DwallResult<()>;
}

/// Selects the theme image whose solar angle is closest to the current one.
pub struct WallpaperSelector;

impl WallpaperSelector {
    /// Finds the closest matching image index based on solar angles
    pub fn find_closest_image(
        solar_configs: &[SolarAngle],
        current_altitude: f64,
        current_azimuth: f64,
    ) -> Option<u8> {
        FrameIndex::new(solar_configs).find_closest_image(current_altitude, current_azimuth)
    }

    /// Finds the position of the solar-angle entry closest to the given position.
    ///
    /// A theme may reuse one image for several sun positions, so callers that
    /// need to highlight the matched point (rather than the image) use this.
    pub fn find_closest_entry(
        solar_configs: &[SolarAngle],
        current_altitude: f64,
        current_azimuth: f64,
    ) -> Option<usize> {
        FrameIndex::new(solar_configs).find_closest_entry(current_altitude, current_azimuth)
    }
}

/// A theme's frames with each declared direction precomputed.
///
/// The match runs on every cycle and, through the schedule, thousands of times per
/// recomputation, while a frame's own trigonometry never changes — so it is done once
/// here instead of once per comparison. The sun's direction is resolved once per query
/// for the same reason.
pub(crate) struct FrameIndex {
    frames: Vec<Frame>,
}

/// One declared frame: the image it shows and the direction it was authored for.
struct Frame {
    image_index: u8,
    entry: usize,
    sin_altitude: f64,
    cos_altitude: f64,
    sin_azimuth: f64,
    cos_azimuth: f64,
}

impl FrameIndex {
    pub(crate) fn new(solar_configs: &[SolarAngle]) -> Self {
        let frames = solar_configs
            .iter()
            .enumerate()
            .map(|(entry, angle)| {
                let altitude = angle.altitude().to_radians();
                let azimuth = angle.azimuth().to_radians();

                Frame {
                    image_index: angle.index(),
                    entry,
                    sin_altitude: altitude.sin(),
                    cos_altitude: altitude.cos(),
                    sin_azimuth: azimuth.sin(),
                    cos_azimuth: azimuth.cos(),
                }
            })
            .collect();

        Self { frames }
    }

    /// The image of the frame whose direction is the smallest angle away.
    pub(crate) fn find_closest_image(&self, altitude: f64, azimuth: f64) -> Option<u8> {
        self.find_closest_entry(altitude, azimuth)
            .map(|entry| self.frames[entry].image_index)
    }

    /// The entry of the frame whose direction is the smallest angle away.
    ///
    /// The angle itself is never computed: the dot product of two unit vectors is the
    /// cosine of the angle between them, and `cos` decreases monotonically over `[0, π]`,
    /// so comparing `1 - cos` compares angles. That is what makes this a solar-*angle*
    /// match — the azimuth counts for `cos(altitude)` of an altitude degree, which
    /// matters near the zenith and not at all at the horizon, where an independently
    /// normalised azimuth would dominate.
    pub(crate) fn find_closest_entry(&self, altitude: f64, azimuth: f64) -> Option<usize> {
        let altitude = altitude.to_radians();
        let azimuth = azimuth.to_radians();
        let (sin_altitude, cos_altitude) = (altitude.sin(), altitude.cos());
        let (sin_azimuth, cos_azimuth) = (azimuth.sin(), azimuth.cos());

        let distance = |frame: &Frame| {
            let cosine = frame.sin_altitude * sin_altitude
                + frame.cos_altitude
                    * cos_altitude
                    * (frame.cos_azimuth * cos_azimuth + frame.sin_azimuth * sin_azimuth);

            1.0 - cosine
        };

        // `total_cmp` instead of `partial_cmp().unwrap()`: a theme file is external data
        // and TOML allows `nan`, which made the unwrap panic (and the daemon abort on the
        // next cycle). Compared as a *distance*, a `nan` frame is the largest value
        // rather than the smallest, so it simply never wins.
        self.frames
            .iter()
            .min_by(|a, b| distance(a).total_cmp(&distance(b)))
            .map(|frame| frame.entry)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn theme_angles() -> Vec<SolarAngle> {
        vec![
            SolarAngle::new(0, -18.0, 0.0),
            SolarAngle::new(1, -6.0, 90.0),
            SolarAngle::new(2, 20.0, 180.0),
            SolarAngle::new(3, 60.0, 270.0),
        ]
    }

    /// The angle between two directions, computed independently of the dot product the
    /// selector minimises, so a test can check the *choice* rather than the formula.
    fn angle_between(altitude: f64, azimuth: f64, other_altitude: f64, other_azimuth: f64) -> f64 {
        let (a1, z1) = (altitude.to_radians(), azimuth.to_radians());
        let (a2, z2) = (other_altitude.to_radians(), other_azimuth.to_radians());
        let cosine = a1.sin() * a2.sin() + a1.cos() * a2.cos() * (z1 - z2).cos();

        cosine.clamp(-1.0, 1.0).acos().to_degrees()
    }

    /// The entry a brute-force scan of the true angle picks.
    fn closest_by_true_angle(angles: &[SolarAngle], altitude: f64, azimuth: f64) -> Option<usize> {
        angles
            .iter()
            .enumerate()
            .min_by(|(_, a), (_, b)| {
                angle_between(a.altitude(), a.azimuth(), altitude, azimuth)
                    .total_cmp(&angle_between(b.altitude(), b.azimuth(), altitude, azimuth))
            })
            .map(|(entry, _)| entry)
    }

    #[test]
    fn finds_the_nearest_entry() {
        let angles = theme_angles();

        assert_eq!(
            WallpaperSelector::find_closest_entry(&angles, -17.0, 2.0),
            Some(0)
        );
        assert_eq!(
            WallpaperSelector::find_closest_entry(&angles, 21.0, 179.0),
            Some(2)
        );
        assert_eq!(
            WallpaperSelector::find_closest_entry(&angles, 59.0, 269.0),
            Some(3)
        );
    }

    #[test]
    fn azimuth_wraps_around_zero() {
        let angles = vec![SolarAngle::new(0, 0.0, 5.0), SolarAngle::new(1, 0.0, 355.0)];

        // 358° is much closer to 355° than to 5° once wrapping is taken into account.
        assert_eq!(
            WallpaperSelector::find_closest_entry(&angles, 0.0, 358.0),
            Some(1)
        );
    }

    #[test]
    fn nan_angles_do_not_panic() {
        // TOML accepts `altitude = nan`; a theme file must not be able to abort the
        // daemon through the distance comparison.
        let angles = vec![
            SolarAngle::new(0, f64::NAN, 0.0),
            SolarAngle::new(1, 10.0, 20.0),
        ];

        assert!(WallpaperSelector::find_closest_entry(&angles, 11.0, 21.0).is_some());
        assert_eq!(
            WallpaperSelector::find_closest_image(&angles, 11.0, 21.0),
            Some(1)
        );
    }

    #[test]
    fn empty_theme_has_no_match() {
        assert_eq!(WallpaperSelector::find_closest_entry(&[], 0.0, 0.0), None);
    }

    /// The measured failure of the previous metric: with the sun overhead on the
    /// equator the azimuth is ill-conditioned, and it chose the 52.4° frame because that
    /// frame's azimuth was 98° rather than 196° — while the theme also had a 77.1° frame.
    #[test]
    fn a_frame_near_the_sun_wins_even_when_its_azimuth_is_far() {
        let angles = vec![
            SolarAngle::new(0, 52.4, 98.0),
            SolarAngle::new(1, 77.1, 196.0),
        ];

        assert_eq!(
            WallpaperSelector::find_closest_entry(&angles, 87.1, 1.0),
            Some(1)
        );
    }

    /// The same failure from the other regime: a low winter sun at 55°N, where the sun's
    /// azimuth barely moves and the frame's azimuth used to dominate the choice.
    #[test]
    fn a_low_sun_does_not_lose_to_a_frame_that_merely_shares_its_azimuth() {
        let angles = vec![
            SolarAngle::new(0, 8.0, 180.0),
            SolarAngle::new(1, 12.0, 90.0),
        ];

        // The sun is at 11.8° due south: the frame 0.2° away in altitude is the match,
        // not the one 90° away in azimuth.
        assert_eq!(
            WallpaperSelector::find_closest_entry(&angles, 11.8, 179.0),
            Some(0)
        );
    }

    /// At the zenith the azimuth carries no information, so two frames that differ only
    /// in azimuth are equally good — and the first one wins.
    #[test]
    fn azimuth_is_irrelevant_at_the_zenith() {
        let angles = vec![
            SolarAngle::new(0, 89.0, 0.0),
            SolarAngle::new(1, 89.0, 180.0),
        ];

        assert_eq!(
            WallpaperSelector::find_closest_entry(&angles, 89.0, 90.0),
            Some(0)
        );
    }

    /// The same angular offset counts the same in both axes at the horizon. The previous
    /// metric measured 5° of azimuth as four times closer than 5° of altitude, so it
    /// preferred azimuth offsets no matter how far the frame's sun height was.
    #[test]
    fn the_same_angular_offset_counts_the_same_in_both_axes() {
        let angles = vec![SolarAngle::new(0, 5.0, 0.0), SolarAngle::new(1, 0.0, 5.0)];

        // The sun sits at the horizon and due north: 5° from the first frame in altitude
        // and 5° from the second in azimuth.
        let to_altitude_frame = angle_between(0.0, 0.0, 5.0, 0.0);
        let to_azimuth_frame = angle_between(0.0, 0.0, 0.0, 5.0);

        assert!((to_altitude_frame - to_azimuth_frame).abs() < 1e-9);
        assert!((to_altitude_frame - 5.0).abs() < 1e-9);

        // Either answer is right, and both have to be *equally* good. Compare the chosen
        // distance rather than the tie-break, which floating point decides.
        let chosen = WallpaperSelector::find_closest_entry(&angles, 0.0, 0.0).unwrap();
        let chosen_distance = angle_between(
            0.0,
            0.0,
            angles[chosen].altitude(),
            angles[chosen].azimuth(),
        );

        assert!(
            (chosen_distance - to_altitude_frame).abs() < 1e-9,
            "the chosen frame is {chosen_distance}° away, not {to_altitude_frame}°"
        );
    }

    /// The selector has to agree with a brute-force scan of the true angle across the
    /// sky, for a theme that covers it.
    #[test]
    fn the_choice_matches_a_brute_force_scan_of_the_true_angle() {
        let angles: Vec<SolarAngle> = (0..8u8)
            .map(|step| {
                SolarAngle::new(
                    step,
                    -60.0 + 20.0 * f64::from(step),
                    f64::from(step) * 43.0 % 360.0,
                )
            })
            .collect();

        for altitude_step in -9..=9 {
            for azimuth_step in 0..24 {
                let altitude = f64::from(altitude_step) * 10.0;
                let azimuth = f64::from(azimuth_step) * 15.0;

                assert_eq!(
                    WallpaperSelector::find_closest_entry(&angles, altitude, azimuth),
                    closest_by_true_angle(&angles, altitude, azimuth),
                    "at ({altitude}°, {azimuth}°)"
                );
            }
        }
    }

    /// A theme may reuse one image for several sun positions, so a match has to name
    /// the closest *entry*: returning the first entry that shares the winning image
    /// would highlight the wrong frame in the settings UI.
    #[test]
    fn a_reused_image_index_reports_the_closest_entry() {
        let angles = vec![
            SolarAngle::new(0, 10.0, 0.0),
            SolarAngle::new(0, 50.0, 0.0),
            SolarAngle::new(1, 80.0, 0.0),
        ];

        assert_eq!(
            WallpaperSelector::find_closest_entry(&angles, 49.0, 0.0),
            Some(1),
            "the second entry is closer even though it shares image 0"
        );
        assert_eq!(
            WallpaperSelector::find_closest_entry(&angles, 11.0, 0.0),
            Some(0)
        );
        assert_eq!(
            WallpaperSelector::find_closest_image(&angles, 79.0, 0.0),
            Some(1),
            "the image index is still what the wallpaper layer applies"
        );
    }
}
