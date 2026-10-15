//! Application usage tracking.
//!
//! Disabled: the app is documented as transmitting no data
//! (`docs/shared-comparison-content.md`), and the previous implementation posted to
//! a third-party endpoint on every start without consent, without a settings
//! toggle and without honouring the configured proxy.
//!
//! If usage telemetry is wanted, it must come back as an explicit, user-visible
//! opt-in (a settings switch persisted in the config) that routes through
//! [`crate::network::HttpClient`] so the user's proxy settings apply.

/// Whether telemetry may be sent in this build.
///
/// Kept as an explicit switch so re-enabling it is a deliberate, reviewable change
/// rather than a comment-out.
const TELEMETRY_ENABLED: bool = false;

pub async fn track() {
    if !TELEMETRY_ENABLED {
        return;
    }

    debug!("Telemetry enabled, reporting startup");
}
