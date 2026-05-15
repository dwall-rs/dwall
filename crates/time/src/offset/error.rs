#[derive(thiserror::Error, Debug)]
pub enum ParseOffsetError {
    #[error("invalid offset format, expected ±HH:MM[:SS]: {0}")]
    InvalidFormat(String),
    #[error("minutes must be 0..=59: {0}")]
    InvalidMinute(u32),
    #[error("seconds must be 0..=59: {0}")]
    InvalidSecond(u32),
    #[error("offset seconds out of allowed range [-50400, 50400]: {0}")]
    OutOfRange(i32),
}
