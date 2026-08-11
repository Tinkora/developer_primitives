//! Reproducible timestamp and IANA time-zone primitives.

mod error;
mod instant;
mod zone;

pub use error::TimeError;
pub use instant::{InstantInputKind, ParsedInstant, parse_instant};
pub use zone::{
    CandidateInstant, LocalResolution, LocalTimeResult, TimeConversion, ZonedTime, convert_instant,
    resolve_local_time, search_time_zones, time_zone_database_version,
};

/// Maximum UTF-8 byte length accepted for one time input.
pub const MAX_TIME_INPUT_BYTES: usize = 128;

/// Maximum ASCII byte length accepted for one IANA time-zone name.
pub const MAX_TIMEZONE_NAME_BYTES: usize = 64;

/// Maximum number of zones compared by one conversion.
pub const MAX_TIMEZONE_COUNT: usize = 8;

/// Maximum number of time-zone names returned by one search.
pub const MAX_TIMEZONE_SEARCH_RESULTS: usize = 50;
