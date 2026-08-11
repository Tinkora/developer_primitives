use thiserror::Error;

/// Stable error type for UUID/ULID operations.
#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum CoreError {
    #[error("Invalid UUID")]
    InvalidUuid,

    #[error("Invalid ULID")]
    InvalidUlid,

    #[error("Invalid identifier")]
    InvalidIdentifier,

    #[error("Batch count must be between 1 and 10000: {0}")]
    BatchOutOfRange(u32),

    #[error("The operating system random source is unavailable")]
    RandomUnavailable,

    #[error("The system clock cannot provide a Unix timestamp")]
    ClockUnavailable,

    #[error("The result could not be serialized")]
    SerializationFailed,

    #[error("Unsupported identifier kind")]
    UnsupportedKind,
}

impl CoreError {
    /// Returns a stable machine error code for JS, CLI, and Agent consumers.
    pub const fn code(&self) -> &'static str {
        match self {
            Self::InvalidUuid => "INVALID_UUID",
            Self::InvalidUlid => "INVALID_ULID",
            Self::InvalidIdentifier => "INVALID_IDENTIFIER",
            Self::BatchOutOfRange(_) => "BATCH_OUT_OF_RANGE",
            Self::RandomUnavailable => "RANDOM_UNAVAILABLE",
            Self::ClockUnavailable => "CLOCK_UNAVAILABLE",
            Self::SerializationFailed => "SERIALIZATION_FAILED",
            Self::UnsupportedKind => "UNSUPPORTED_KIND",
        }
    }
}
