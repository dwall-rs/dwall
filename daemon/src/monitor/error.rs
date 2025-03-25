use windows::Win32::Foundation::WIN32_ERROR;

#[derive(Debug, thiserror::Error)]
pub enum MonitorError {
    #[error("Failed to get monitor set")]
    GetDeviceInfoSet,
    #[error("Failed to get monitor")]
    GetDeviceInfo,
    #[error("Failed to get target name")]
    GetTargetName,
    #[error("Failed to find matching device")]
    MatchDevice,
    #[error("Failed to get friendly name")]
    GetFriendlyName,
    #[error("Failed to get buffer sizes")]
    GetBufferSizes,
    #[error("Failed to query display config: {0:?}")]
    QueryDisplayConfig(WIN32_ERROR),
}
