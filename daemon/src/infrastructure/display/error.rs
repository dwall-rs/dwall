//! Display-related error types

use windows::{core::Error as WindowsError, Win32::Foundation::WIN32_ERROR};

/// Display operation related errors
#[derive(Debug, thiserror::Error)]
pub enum DisplayError {
    /// Unable to get monitor device collection
    #[error("Unable to get monitor device collection: {0:?}")]
    GetDeviceInfoSet(#[source] Option<WindowsError>),

    /// Unable to get monitor device information
    #[error("Unable to get monitor device information: {0:?}")]
    GetDeviceInfo(#[source] WindowsError),

    /// Unable to get target device name
    #[error("Unable to get target device name: {0:?}")]
    GetTargetName(#[source] WindowsError),

    /// Unable to find matching device
    #[error("Unable to find matching device")]
    MatchDevice,

    /// Unable to get device friendly name
    #[error("Unable to get device friendly name: {0:?}")]
    GetFriendlyName(#[source] WindowsError),

    /// Unable to get buffer sizes
    #[error("Unable to get buffer sizes: {0:?}")]
    GetBufferSizes(WIN32_ERROR),

    /// Failed to query display configuration
    #[error("Failed to query display configuration: {0:?}")]
    QueryDisplayConfig(WIN32_ERROR),

    /// Failed to get device registry property
    #[error("Failed to get device registry property: {0:?}")]
    GetDeviceRegistryProperty(#[source] WindowsError),
}
