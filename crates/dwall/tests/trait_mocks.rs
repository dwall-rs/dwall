//! Integration tests for the provider traits and the real code behind them.
//!
//! `dwall-settings` consumes these traits, so their signatures are part of the
//! public API and the mocks below fail to compile if one of them changes. The tests
//! drive `GeographicPositionProvider` — the public implementation of
//! `PositionProvider` — so they check the caching and the manual/automatic split
//! rather than mockall's own behaviour, which is what this file used to do.

use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};

use dwall::color_scheme::{ColorScheme, ColorSchemeProvider};
use dwall::config::PositionSource;
use dwall::error::DwallResult;
use dwall::monitor::MonitorProvider;
use dwall::platform::DisplayMonitor;
use dwall::solar::{GeographicPositionProvider, Position, PositionProvider};
use dwall::wallpaper::WallpaperProvider;

// ── Mock implementations ────────────────────────────────────────────────────

mockall::mock! {
    pub PositionProvider {}

    impl dwall::solar::PositionProvider for PositionProvider {
        fn get_current_position(&self) -> dwall::error::DwallResult<Position>;
        fn check_location_permission(&self) -> dwall::error::DwallResult<()>;
    }
}

mockall::mock! {
    pub MonitorProvider {}

    impl dwall::monitor::MonitorProvider for MonitorProvider {
        fn get_monitors(&self) -> dwall::error::DwallResult<HashMap<String, DisplayMonitor>>;
        fn refresh_monitors(&self) -> dwall::error::DwallResult<HashMap<String, DisplayMonitor>>;
        fn has_configuration_changed(&self) -> dwall::error::DwallResult<bool>;
    }
}

mockall::mock! {
    pub WallpaperSetter {}

    impl dwall::wallpaper::WallpaperProvider for WallpaperSetter {
        fn set_monitor_wallpaper(&self, monitor_id: &str, image_path: &Path) -> dwall::error::DwallResult<()>;
        fn set_lock_screen_image(&self, image_path: &Path) -> dwall::error::DwallResult<()>;
    }
}

mockall::mock! {
    pub ColorSchemeBackend {}

    impl dwall::color_scheme::ColorSchemeProvider for ColorSchemeBackend {
        fn get_current_scheme(&self) -> dwall::error::DwallResult<ColorScheme>;
        fn set_color_scheme(&self, scheme: ColorScheme) -> dwall::error::DwallResult<()>;
    }
}

// ── Tests ───────────────────────────────────────────────────────────────────

/// The signature guard: a mock of every provider trait has to implement it.
#[test]
fn every_provider_trait_can_be_implemented_from_outside_the_crate() {
    fn takes_position_provider(_: &impl PositionProvider) {}
    fn takes_monitor_provider(_: &impl MonitorProvider) {}
    fn takes_wallpaper_provider(_: &impl WallpaperProvider) {}
    fn takes_color_scheme_provider(_: &impl ColorSchemeProvider) {}

    takes_position_provider(&MockPositionProvider::new());
    takes_monitor_provider(&MockMonitorProvider::new());
    takes_wallpaper_provider(&MockWallpaperSetter::new());
    takes_color_scheme_provider(&MockColorSchemeBackend::new());
}

/// A source that counts how often the geolocation backend was really asked.
fn counting_source(calls: &Arc<AtomicU32>) -> MockPositionProvider {
    let mut mock = MockPositionProvider::new();
    let counter = Arc::clone(calls);

    mock.expect_get_current_position().returning(move || {
        counter.fetch_add(1, Ordering::SeqCst);

        Position::new(45.0, 116.0, 100.0)
    });

    mock
}

fn automatic(update_on_each_calculation: bool, cache_minutes: u64) -> PositionSource {
    PositionSource::Automatic {
        update_on_each_calculation,
        cache_minutes,
    }
}

#[test]
fn automatic_mode_takes_its_coordinates_from_the_backend() {
    let calls = Arc::new(AtomicU32::new(0));
    let source = automatic(false, 60);
    let provider = GeographicPositionProvider::new(&source, counting_source(&calls));

    let position = provider.get_current_position().unwrap();

    // The backend's coordinates, which automatic mode has no other source for.
    assert_eq!(position.latitude(), 45.0);
    assert_eq!(position.longitude(), 116.0);
    assert!(calls.load(Ordering::SeqCst) <= 1, "asked at most once");
}

/// The documented caching: automatic mode does not re-ask the backend inside the
/// cache window. The cache itself is a thread-local shared by every test on this
/// thread, so the assertion is a delta rather than an absolute count.
#[test]
fn a_second_call_within_the_cache_minutes_reuses_the_position() {
    let calls = Arc::new(AtomicU32::new(0));
    let source = automatic(false, 60);
    let provider = GeographicPositionProvider::new(&source, counting_source(&calls));

    let first = provider.get_current_position().unwrap();
    let queries_after_first = calls.load(Ordering::SeqCst);

    let second = provider.get_current_position().unwrap();

    assert_eq!(first.latitude(), second.latitude());
    assert_eq!(
        calls.load(Ordering::SeqCst),
        queries_after_first,
        "the backend must not be queried twice inside the cache window"
    );
}

/// `update_on_each_calculation` is the documented opt-out of that cache.
#[test]
fn update_on_each_calculation_asks_the_backend_every_time() {
    let calls = Arc::new(AtomicU32::new(0));
    let source = automatic(true, 60);
    let provider = GeographicPositionProvider::new(&source, counting_source(&calls));

    provider.get_current_position().unwrap();
    provider.get_current_position().unwrap();

    assert_eq!(
        calls.load(Ordering::SeqCst),
        2,
        "every calculation has to be a fresh one"
    );
}

/// Manual mode never touches the geolocation backend: the coordinates come from the
/// configuration, which is what makes a machine without location services usable.
#[test]
fn manual_mode_never_queries_the_backend() {
    let calls = Arc::new(AtomicU32::new(0));
    let source = PositionSource::Manual {
        latitude: -33.9,
        longitude: 151.2,
        altitude: 20.0,
    };
    let provider = GeographicPositionProvider::new(&source, counting_source(&calls));

    let position = provider.get_current_position().unwrap();

    assert_eq!(position.latitude(), -33.9);
    assert_eq!(position.longitude(), 151.2);
    assert_eq!(position.altitude(), 20.0);
    assert_eq!(
        calls.load(Ordering::SeqCst),
        0,
        "manual coordinates need no geolocation"
    );
}

/// The trait implementation forwards to the backend rather than swallowing the
/// failure: a denied permission has to stay visible to the caller that decides what
/// to log and how long to back off.
#[test]
fn a_denied_permission_surfaces_through_the_provider() {
    let mut backend = MockPositionProvider::new();
    backend.expect_check_location_permission().returning(|| {
        Err(dwall::error::DwallError::GeolocationAccess(
            dwall::solar::GeolocationAccessError::Denied,
        ))
    });

    let source = automatic(false, 60);
    let provider = GeographicPositionProvider::new(&source, backend);

    // Called through the trait, not the inherent method, so the forwarding itself
    // is what is under test.
    assert!(PositionProvider::check_location_permission(&provider).is_err());
}

#[test]
fn a_failing_backend_is_reported_in_automatic_mode() {
    let mut backend = MockPositionProvider::new();
    backend.expect_get_current_position().returning(|| {
        Err(dwall::error::DwallError::GeolocationAccess(
            dwall::solar::GeolocationAccessError::Unspecified,
        ))
    });

    let source = automatic(true, 60);
    let provider = GeographicPositionProvider::new(&source, backend);

    assert!(provider.get_current_position().is_err());
}

/// The other three traits are consumed as objects the daemon drives; this pins that
/// a mock really is what the trait expects, including the error type.
#[test]
fn the_monitor_and_wallpaper_traits_can_be_driven_through_their_mocks() {
    let mut monitors = MockMonitorProvider::new();
    monitors.expect_get_monitors().times(1).returning(|| {
        let mut map = HashMap::new();
        map.insert(
            "MON-A".to_string(),
            DisplayMonitor::new_test("MON-A".to_string(), "Display 1".to_string(), Some(0)),
        );

        Ok(map)
    });

    let mut wallpapers = MockWallpaperSetter::new();
    wallpapers
        .expect_set_monitor_wallpaper()
        .withf(|monitor_id, image_path| monitor_id == "MON-A" && image_path.ends_with("1.jpg"))
        .times(1)
        .returning(|_, _| Ok(()));

    let available = monitors.get_monitors().unwrap();
    assert!(available.contains_key("MON-A"));

    let applied: DwallResult<()> =
        wallpapers.set_monitor_wallpaper("MON-A", Path::new(r"C:\themes\Big Sur\jpg\1.jpg"));
    assert!(applied.is_ok());
}
