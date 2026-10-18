//! When the sun next changes what the daemon would do.
//!
//! The daemon used to recompute everything every `interval` seconds, which meant a
//! frame switch (and a light/dark switch) could be up to a whole interval late — ten
//! minutes for a configuration with `interval = 600`. This module answers the narrower
//! question "when does the answer next change?", so the loop can sleep exactly until
//! then and keep the interval only as the period after which it recomputes anyway.
//!
//! The search never re-implements the decision: the caller passes the same function
//! the cycle uses, evaluated at a future instant. A schedule that disagreed with the
//! cycle would be worse than no schedule at all.

use std::time::Duration;

use time::OffsetDateTime;

/// How coarsely the search looks for a change before narrowing it down.
///
/// Changes closer together than this can be skipped, which is deliberate: the finest
/// themes switch frames as often as every fifteen seconds, and waking for a frame that
/// lasts less than half a minute would cost more than it shows.
pub(crate) const SCAN_STEP: Duration = Duration::from_secs(30);

/// The wait returned when the decision changes immediately.
///
/// A whole second, because the alternative is a loop that runs the cycle, finds the
/// change is still "now", and runs it again.
pub(crate) const MIN_WAIT: Duration = Duration::from_secs(1);

/// The time until `state` first differs from its value at `now`, within `horizon`.
///
/// `state` returns `None` when it cannot be evaluated at all (a missing theme, a
/// provider error); the search then gives up instead of guessing, and the caller falls
/// back to its plain interval.
///
/// The result is exact to the second when the decision changes once per scan step or
/// more slowly. Two changes inside one step cancel out — see [`SCAN_STEP`].
pub(crate) fn next_change<T, F>(
    now: &OffsetDateTime,
    horizon: Duration,
    state: F,
) -> Option<Duration>
where
    T: PartialEq,
    F: FnMut(&OffsetDateTime) -> Option<T>,
{
    next_change_with_step(now, horizon, SCAN_STEP, state)
}

/// [`next_change`] with an explicit scan step, so a test can pin the resolution.
pub(crate) fn next_change_with_step<T, F>(
    now: &OffsetDateTime,
    horizon: Duration,
    step: Duration,
    mut state: F,
) -> Option<Duration>
where
    T: PartialEq,
    F: FnMut(&OffsetDateTime) -> Option<T>,
{
    let horizon_seconds = horizon.as_secs();

    if horizon_seconds == 0 {
        return None;
    }

    let step_seconds = step.as_secs().max(1);
    let unchanged = state(now)?;
    let mut offset = 0;

    while offset < horizon_seconds {
        let next_offset = (offset + step_seconds).min(horizon_seconds);
        let at = now.add_seconds(next_offset).ok()?;
        let current = state(&at)?;

        if current != unchanged {
            // Narrow the step down to the second. `unchanged` is the value everything
            // below `low` still has, so this converges on the first change even when
            // the state changes again later inside the same step — but not when it
            // returns to `unchanged` before the step ends, which is the miss described
            // on `SCAN_STEP`.
            let mut low = offset;
            let mut high = next_offset;

            while high - low > 1 {
                let middle = low + (high - low) / 2;
                let at_middle = now.add_seconds(middle).ok()?;

                if state(&at_middle)? == unchanged {
                    low = middle;
                } else {
                    high = middle;
                }
            }

            return Some(Duration::from_secs(high));
        }

        offset = next_offset;
    }

    None
}

#[cfg(test)]
mod tests {
    use time::{Month, Offset};

    use super::*;

    /// The instant the searches below start from. The offset only has to be
    /// consistent: the search works on differences.
    fn start() -> OffsetDateTime {
        OffsetDateTime::new(2026, Month::March, 20, 8, 0, 0, Offset::local_offset())
            .expect("a valid local time")
    }

    /// A decision that switches on at a fixed offset from the start of the search.
    fn switches_on_at(seconds: u64) -> impl FnMut(&OffsetDateTime) -> Option<bool> {
        let base = start().timestamp();

        move |at: &OffsetDateTime| Some(at.timestamp().saturating_sub(base) >= seconds)
    }

    #[test]
    fn finds_the_first_change_within_the_horizon() {
        let found = next_change(&start(), Duration::from_secs(600), switches_on_at(100));

        assert_eq!(found, Some(Duration::from_secs(100)));
    }

    #[test]
    fn narrows_the_change_down_to_the_second() {
        let found = next_change(&start(), Duration::from_secs(600), switches_on_at(137));

        assert_eq!(found, Some(Duration::from_secs(137)));
    }

    #[test]
    fn returns_nothing_when_the_decision_never_changes() {
        let found = next_change(&start(), Duration::from_secs(3600), |_| Some(false));

        assert_eq!(found, None);
    }

    #[test]
    fn returns_nothing_when_the_change_lands_past_the_horizon() {
        let found = next_change(&start(), Duration::from_secs(60), switches_on_at(3600));

        assert_eq!(found, None);
    }

    #[test]
    fn returns_nothing_when_the_decision_cannot_be_evaluated() {
        let found = next_change(
            &start(),
            Duration::from_secs(600),
            |_: &OffsetDateTime| -> Option<bool> { None },
        );

        assert_eq!(found, None);
    }

    #[test]
    fn gives_up_when_evaluating_a_later_instant_fails() {
        let mut calls = 0;

        let found = next_change(&start(), Duration::from_secs(600), move |_| {
            calls += 1;
            (calls <= 1).then_some(false)
        });

        assert_eq!(found, None);
    }

    /// A change and a recovery inside one scan step cancel out: this is the resolution
    /// the schedule accepts rather than searching every second of the horizon.
    #[test]
    fn a_change_that_recovers_inside_one_step_is_not_reported() {
        let start = start();
        let base = start.timestamp();
        let found = next_change_with_step(
            &start,
            Duration::from_secs(600),
            Duration::from_secs(30),
            move |at: &OffsetDateTime| {
                let elapsed = at.timestamp().saturating_sub(base);
                Some((40..60).contains(&elapsed))
            },
        );

        assert_eq!(found, None);
    }

    /// The second change after a recovery *is* found, and it is the first one after the
    /// step that still contains one.
    #[test]
    fn a_recovery_is_looked_past_to_the_next_change() {
        let start = start();
        let base = start.timestamp();
        let found = next_change_with_step(
            &start,
            Duration::from_secs(600),
            Duration::from_secs(30),
            move |at: &OffsetDateTime| {
                let elapsed = at.timestamp().saturating_sub(base);
                Some((40..60).contains(&elapsed) || elapsed >= 200)
            },
        );

        assert_eq!(found, Some(Duration::from_secs(200)));
    }

    #[test]
    fn a_zero_horizon_never_reports_a_change() {
        assert_eq!(next_change(&start(), Duration::ZERO, |_| Some(true)), None);
    }

    /// The wait is never zero: the loop would otherwise run its cycle again immediately.
    #[test]
    fn the_wait_is_never_shorter_than_a_second() {
        let start = start();
        let base = start.timestamp();
        let found = next_change_with_step(
            &start,
            Duration::from_secs(60),
            Duration::from_secs(1),
            move |at: &OffsetDateTime| {
                let elapsed = at.timestamp().saturating_sub(base);
                Some(elapsed >= 1)
            },
        );

        assert_eq!(found, Some(Duration::from_secs(1)));
        assert!(found.unwrap() >= MIN_WAIT);
    }
}
