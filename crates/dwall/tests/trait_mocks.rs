//! Integration tests for trait abstractions
//!
//! These tests verify that the trait abstractions work correctly
//! and can be mocked for testing purposes.

use std::collections::HashMap;
use std::path::Path;

use dwall::domain::geography::PositionProvider;
use dwall::domain::visual::ColorSchemeProvider;
use dwall::domain::visual::MonitorProvider;
use dwall::domain::visual::WallpaperProvider;
use mockall::Sequence;
use mockall::predicate::*;

use dwall::domain::geography::Position;
use dwall::domain::visual::ColorScheme;
use dwall::infrastructure::platform::windows::display::monitor_manager::DisplayMonitor;

// ── Mock implementations ────────────────────────────────────────────────────

mockall::mock! {
    pub PositionProvider {}

    impl dwall::domain::geography::PositionProvider for PositionProvider {
        fn get_current_position(&self) -> dwall::error::DwallResult<Position>;
        fn check_location_permission(&self) -> dwall::error::DwallResult<()>;
    }
}

mockall::mock! {
    pub MonitorProvider {}

    impl dwall::domain::visual::MonitorProvider for MonitorProvider {
        fn get_monitors(&self) -> dwall::error::DwallResult<HashMap<String, DisplayMonitor>>;
        fn refresh_monitors(&self) -> dwall::error::DwallResult<HashMap<String, DisplayMonitor>>;
        fn has_configuration_changed(&self) -> dwall::error::DwallResult<bool>;
    }
}

mockall::mock! {
    pub WallpaperSetter {}

    impl dwall::domain::visual::WallpaperProvider for WallpaperSetter {
        fn set_monitor_wallpaper(&self, monitor_id: &str, image_path: &Path) -> dwall::error::DwallResult<()>;
        fn set_lock_screen_image(&self, image_path: &Path) -> dwall::error::DwallResult<()>;
    }
}

mockall::mock! {
    pub ColorSchemeBackend {}

    impl dwall::domain::visual::ColorSchemeProvider for ColorSchemeBackend {
        fn get_current_scheme(&self) -> dwall::error::DwallResult<ColorScheme>;
        fn set_color_scheme(&self, scheme: ColorScheme) -> dwall::error::DwallResult<()>;
    }
}

// ── Tests ───────────────────────────────────────────────────────────────────

#[test]
fn mock_position_provider_returns_configured_value() {
    let mut mock = MockPositionProvider::new();
    let expected_position = Position::new(45.0, 116.0, 100.0).unwrap();

    mock.expect_get_current_position()
        .times(1)
        .returning(move || Ok(Position::new(45.0, 116.0, 100.0).unwrap()));

    let result = mock.get_current_position().unwrap();
    assert_eq!(result.latitude(), expected_position.latitude());
    assert_eq!(result.longitude(), expected_position.longitude());
    assert_eq!(result.altitude(), expected_position.altitude());
}

#[test]
fn mock_position_provider_can_return_errors() {
    let mut mock = MockPositionProvider::new();

    mock.expect_get_current_position().times(1).returning(|| {
        Err(dwall::error::DwallError::GeolocationAccess(
            dwall::domain::geography::GeolocationAccessError::Denied,
        ))
    });

    let result = mock.get_current_position();
    assert!(result.is_err());
}

#[test]
fn mock_monitor_provider_returns_monitors() {
    let mut mock = MockMonitorProvider::new();

    mock.expect_get_monitors().times(1).returning(move || {
        let mut m = HashMap::new();
        m.insert(
            "monitor1".to_string(),
            DisplayMonitor::new_test("path1".to_string(), "Display 1".to_string(), Some(0)),
        );
        Ok(m)
    });

    let result = mock.get_monitors().unwrap();
    assert_eq!(result.len(), 1);
    assert!(result.contains_key("monitor1"));
}

#[test]
fn mock_monitor_provider_detects_configuration_change() {
    let mut mock = MockMonitorProvider::new();

    mock.expect_has_configuration_changed()
        .times(1)
        .returning(|| Ok(true));

    assert!(mock.has_configuration_changed().unwrap());
}

#[test]
fn mock_wallpaper_setter_sets_wallpaper() {
    let mut mock = MockWallpaperSetter::new();

    mock.expect_set_monitor_wallpaper()
        .with(eq("monitor1"), eq(Path::new("/tmp/wallpaper.jpg")))
        .times(1)
        .returning(|_, _| Ok(()));

    let result = mock.set_monitor_wallpaper("monitor1", Path::new("/tmp/wallpaper.jpg"));
    assert!(result.is_ok());
}

#[test]
fn mock_color_scheme_backend_gets_and_sets_scheme() {
    let mut mock = MockColorSchemeBackend::new();

    mock.expect_get_current_scheme()
        .times(1)
        .returning(|| Ok(ColorScheme::Light));

    mock.expect_set_color_scheme()
        .with(eq(ColorScheme::Dark))
        .times(1)
        .returning(|_| Ok(()));

    assert_eq!(mock.get_current_scheme().unwrap(), ColorScheme::Light);
    assert!(mock.set_color_scheme(ColorScheme::Dark).is_ok());
}

#[test]
fn mock_color_scheme_backend_sequence() {
    let mut mock = MockColorSchemeBackend::new();
    let mut seq = Sequence::new();

    mock.expect_get_current_scheme()
        .times(1)
        .in_sequence(&mut seq)
        .returning(|| Ok(ColorScheme::Light));

    mock.expect_set_color_scheme()
        .times(1)
        .in_sequence(&mut seq)
        .returning(|_| Ok(()));

    mock.expect_get_current_scheme()
        .times(1)
        .in_sequence(&mut seq)
        .returning(|| Ok(ColorScheme::Dark));

    // Initial state: Light
    assert_eq!(mock.get_current_scheme().unwrap(), ColorScheme::Light);
    // Set to Dark
    assert!(mock.set_color_scheme(ColorScheme::Dark).is_ok());
    // Verify state changed
    assert_eq!(mock.get_current_scheme().unwrap(), ColorScheme::Dark);
}
