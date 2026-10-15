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
