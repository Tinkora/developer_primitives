use thiserror::Error;

/// Stable failures returned by the time conversion core.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum TimeError {
    #[error("Invalid timestamp")]
    InvalidTimestamp,
    #[error("Invalid RFC 3339 timestamp")]
    InvalidRfc3339,
    #[error("Invalid local date and time")]
    InvalidLocalDateTime,
    #[error("Invalid IANA time zone")]
    InvalidTimezone,
    #[error("Duplicate IANA time zone")]
    DuplicateTimezone,
    #[error("Time zone count must be between 1 and 8")]
    TimezoneLimitExceeded,
    #[error("Input exceeds the supported length")]
    InputTooLong,
    #[error("Could not serialize the time result")]
    SerializationFailed,
}

impl TimeError {
    /// Return the stable machine-readable code for this failure.
    pub const fn code(self) -> &'static str {
        match self {
            Self::InvalidTimestamp => "INVALID_TIMESTAMP",
            Self::InvalidRfc3339 => "INVALID_RFC3339",
            Self::InvalidLocalDateTime => "INVALID_LOCAL_DATETIME",
            Self::InvalidTimezone => "INVALID_TIMEZONE",
            Self::DuplicateTimezone => "DUPLICATE_TIMEZONE",
            Self::TimezoneLimitExceeded => "TIMEZONE_LIMIT_EXCEEDED",
            Self::InputTooLong => "INPUT_TOO_LONG",
            Self::SerializationFailed => "SERIALIZATION_FAILED",
        }
    }
}
