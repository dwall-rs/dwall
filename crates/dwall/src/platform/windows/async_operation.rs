//! Bounded waiting for WinRT asynchronous operations.
//!
//! WinRT hands back `IAsyncOperation`/`IAsyncAction` values that the caller has to
//! wait for. The helpers `windows-future` offers for that are unusable here:
//!
//! - `IAsyncOperation::get()` (0.2) / `join()` (0.3) wait with
//!   `WaitForSingleObject(…, INFINITE)`, so a wedged shell, an unreachable network
//!   share or an unresponsive location service blocks this single-threaded daemon
//!   forever;
//! - the `IntoFuture` implementations need an executor to drive them. This daemon
//!   is deliberately synchronous and single-threaded, and it has nothing to run
//!   concurrently anyway: awaiting would park the same thread on a waker instead of
//!   on a wait handle, for the same latency plus an executor and its memory
//!   footprint.
//!
//! Waiting here polls `Status()` and reads the outcome once the operation leaves
//! `Started`. That bounds every WinRT call, needs no completion handler (which
//! would require the result type to be `Send`), and leaves `windows-future` as a
//! pure source of the WinRT types.

use std::time::{Duration, Instant};

use windows_future::{AsyncStatus, IAsyncAction, IAsyncOperation};

/// Poll interval while an operation is still running.
const POLL_INTERVAL: Duration = Duration::from_millis(50);

/// Why a WinRT operation produced no usable result.
#[derive(Debug, thiserror::Error)]
pub enum AsyncWaitError {
    /// The operation completed but failed.
    #[error("WinRT operation failed: {0}")]
    Failed(#[from] windows::core::Error),

    /// The operation was still running when its budget ran out.
    #[error("WinRT operation did not finish within {0:?}")]
    Timeout(Duration),
}

impl From<AsyncWaitError> for crate::error::DwallError {
    fn from(error: AsyncWaitError) -> Self {
        match error {
            AsyncWaitError::Failed(error) => Self::Windows(error),
            AsyncWaitError::Timeout(timeout) => Self::AsyncTimeout(timeout),
        }
    }
}

/// Waits for an `IAsyncOperation<T>` for at most `timeout`.
pub fn wait<T: windows::core::RuntimeType>(
    operation: &IAsyncOperation<T>,
    timeout: Duration,
) -> Result<T, AsyncWaitError> {
    poll(|| operation.Status(), timeout)?;

    operation.GetResults().map_err(AsyncWaitError::Failed)
}

/// Waits for an `IAsyncAction` (no result value) for at most `timeout`.
pub fn wait_action(operation: &IAsyncAction, timeout: Duration) -> Result<(), AsyncWaitError> {
    poll(|| operation.Status(), timeout)?;

    operation.GetResults().map_err(AsyncWaitError::Failed)
}

/// Polls `status` until it leaves [`AsyncStatus::Started`] or `timeout` elapses.
///
/// A terminal state — or a failure of `status` itself — is reported by
/// `GetResults`, which the callers run next.
fn poll(
    status: impl Fn() -> windows::core::Result<AsyncStatus>,
    timeout: Duration,
) -> Result<(), AsyncWaitError> {
    let started = Instant::now();

    loop {
        match status() {
            Ok(AsyncStatus::Started) => {
                if started.elapsed() >= timeout {
                    return Err(AsyncWaitError::Timeout(timeout));
                }
                std::thread::sleep(POLL_INTERVAL);
            }
            _ => return Ok(()),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;

    use super::*;

    /// A status source that yields `statuses` in order, repeating the last one.
    fn statuses<'a>(
        statuses: &'a [AsyncStatus],
        calls: &'a Cell<usize>,
    ) -> impl Fn() -> windows::core::Result<AsyncStatus> + 'a {
        move || {
            let index = calls.get().min(statuses.len() - 1);
            calls.set(calls.get() + 1);

            Ok(statuses[index])
        }
    }

    #[test]
    fn poll_returns_as_soon_as_the_operation_is_done() {
        let calls = Cell::new(0);

        let result = poll(
            statuses(
                &[
                    AsyncStatus::Started,
                    AsyncStatus::Started,
                    AsyncStatus::Completed,
                ],
                &calls,
            ),
            Duration::from_secs(5),
        );

        assert!(result.is_ok());
        assert_eq!(calls.get(), 3, "it must stop at the terminal status");
    }

    /// The case the whole module exists for: a shell or location service that never
    /// answers has to end the wait instead of blocking the daemon forever.
    #[test]
    fn poll_times_out_on_a_wedged_operation() {
        let calls = Cell::new(0);
        let budget = Duration::from_millis(120);
        let started = Instant::now();

        let error = poll(statuses(&[AsyncStatus::Started], &calls), budget)
            .expect_err("a status that never leaves Started has to time out");

        match error {
            AsyncWaitError::Timeout(reported) => assert_eq!(reported, budget),
            other => panic!("expected a timeout, got {other}"),
        }
        assert!(started.elapsed() >= budget);
        assert!(calls.get() > 1, "it polls until the budget runs out");
    }

    /// `Error` and `Canceled` are terminal: polling them would burn the budget and then
    /// report a timeout instead of the failure the caller is about to read.
    #[test]
    fn a_terminal_failure_status_ends_the_wait() {
        for status in [AsyncStatus::Error, AsyncStatus::Canceled] {
            let calls = Cell::new(0);

            assert!(poll(statuses(&[status], &calls), Duration::from_secs(5)).is_ok());
            assert_eq!(calls.get(), 1);
        }
    }

    /// A status call that fails is also reported by `GetResults`; spinning on it would
    /// waste the whole timeout first.
    #[test]
    fn a_status_call_that_fails_ends_the_wait() {
        let calls = Cell::new(0);

        let result = poll(
            || {
                calls.set(calls.get() + 1);

                Err(windows::core::Error::from_hresult(
                    windows::core::HRESULT::from_win32(5),
                ))
            },
            Duration::from_secs(5),
        );

        assert!(result.is_ok());
        assert_eq!(calls.get(), 1);
    }

    /// The two failure kinds reach the daemon as different errors, and the location
    /// check depends on the second one being recognisable.
    #[test]
    fn the_two_failure_kinds_reach_the_daemon_differently() {
        let timeout: crate::error::DwallError =
            AsyncWaitError::Timeout(Duration::from_secs(30)).into();
        assert!(matches!(timeout, crate::error::DwallError::AsyncTimeout(_)));

        let failed: crate::error::DwallError = AsyncWaitError::Failed(
            windows::core::Error::from_hresult(windows::core::HRESULT::from_win32(5)),
        )
        .into();
        assert!(matches!(failed, crate::error::DwallError::Windows(_)));
        assert!(
            failed.location_unavailable(),
            "an access-denied wait is what makes the daemon stop for a missing location"
        );
    }
}
