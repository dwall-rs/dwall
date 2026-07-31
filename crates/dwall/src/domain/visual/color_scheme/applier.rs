//! Color scheme application logic for automatic light/dark mode switching

use std::time::Duration;

use time::OffsetDateTime;

use crate::{
    DwallResult,
    config::Config,
    domain::{
        geography::Position,
        time::solar_calculator::SolarPosition,
        visual::{
            DaylightState,
            color_scheme::{ColorSchemeProvider, ThresholdConfig},
        },
    },
    utils::cache::get_cache,
};

use crate::domain::visual::color_scheme::engine::determine_color_scheme_with_hysteresis;

/// Applies system color scheme based on solar position
pub(crate) struct ColorSchemeApplier<T: ColorSchemeProvider> {
    color_scheme_provider: T,
}

impl<T: ColorSchemeProvider> ColorSchemeApplier<T> {
    /// Creates a new ColorSchemeApplier with the given theme backend
    pub(crate) fn new(color_scheme_provider: T) -> Self {
        Self {
            color_scheme_provider,
        }
    }

    /// Updates system color scheme based on current solar position
    pub(crate) fn update_color_scheme(
        &self,
        config: &Config,
        now: &OffsetDateTime,
        geographic_position: &Position,
        solar_position: &SolarPosition,
    ) -> DwallResult<()> {
        if !config.auto_detect_color_scheme() {
            return Ok(());
        }

        let cache = get_cache();

        let threshold_config = match cache.get::<ThresholdConfig>() {
            Some(tc) => tc,
            None => {
                let tc = ThresholdConfig::from_position(geographic_position);
                cache.set(tc, std::time::Duration::from_hours(24));
                tc
            }
        };

        let daylight_state = match cache.get::<DaylightState>() {
            Some(ds) => ds,
            None => {
                let ds = DaylightState::detect(geographic_position, now.date(), &threshold_config);
                cache.set(ds, Duration::from_hours(24));
                ds
            }
        };

        let current_color_scheme = self.color_scheme_provider.get_current_scheme()?;
        let solar_based_color_scheme = determine_color_scheme_with_hysteresis(
            solar_position,
            &current_color_scheme,
            &threshold_config,
            now,
            &daylight_state,
        );

        debug!(
            color_scheme = ?solar_based_color_scheme,
            sun_altitude = solar_position.altitude(),
            "Automatically updating system color scheme based on solar position"
        );

        if let Err(color_scheme_error) = self
            .color_scheme_provider
            .set_color_scheme(solar_based_color_scheme)
        {
            warn!(
                error = %color_scheme_error,
                "Failed to update system color scheme, continuing with other operations"
            );
        }

        Ok(())
    }
}
