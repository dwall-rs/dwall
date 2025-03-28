use windows::Win32::Foundation::WIN32_ERROR;

/// Monitor operation related errors
///
/// Contains all possible errors that may occur during interaction with monitor devices
#[derive(Debug, thiserror::Error)]
pub enum MonitorError {
    /// Unable to get monitor device collection
    #[error("Unable to get monitor device collection")]
    GetDeviceInfoSet,

    /// Unable to get monitor device information
    #[error("Unable to get monitor device information")]
    GetDeviceInfo,

    /// Unable to get target device name
    #[error("Unable to get target device name")]
    GetTargetName,

    /// Unable to find matching device
    #[error("Unable to find matching device")]
    MatchDevice,

    /// Unable to get device friendly name
    #[error("Unable to get device friendly name")]
    GetFriendlyName,

    /// Unable to get buffer sizes
    #[error("Unable to get buffer sizes")]
    GetBufferSizes,

    /// Failed to query display configuration
    #[error("Failed to query display configuration: {0:?}")]
    QueryDisplayConfig(WIN32_ERROR),
}
