use std::time::SystemTimeError;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum DateTimeError {
    #[error("Year must be >= 1970, got {0}")]
    InvalidYear(u16),

    #[error("Day {0} is invalid for the given month")]
    InvalidDay(u8),

    #[error("Hour must be < 24, got {0}")]
    InvalidHour(u8),

    #[error("Minute must be < 60, got {0}")]
    InvalidMinute(u8),

    #[error("Second must be < 60, got {0}")]
    InvalidSecond(u8),

    #[error("System time error: {0}")]
    SystemTime(#[from] SystemTimeError),

    #[error("Arithmetic overflow in date calculation")]
    Overflow,

    #[error("Invalid month value: {0}")]
    InvalidMonth(u8),

    #[error("Time difference would be negative")]
    NegativeDuration,
}

pub type DateTimeResult<T> = Result<T, DateTimeError>;
