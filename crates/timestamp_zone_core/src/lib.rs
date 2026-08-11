//! Reproducible timestamp and IANA time-zone primitives.

mod error;
mod instant;

pub use error::TimeError;
pub use instant::{InstantInputKind, ParsedInstant, parse_instant};

/// Maximum UTF-8 byte length accepted for one time input.
pub const MAX_TIME_INPUT_BYTES: usize = 128;
