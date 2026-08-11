use chrono::DateTime;
use jiff::Timestamp;
use serde::{Deserialize, Serialize};

use crate::{MAX_TIME_INPUT_BYTES, TimeError};

/// Explicit unit or syntax used to interpret one instant.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum InstantInputKind {
    UnixSeconds,
    UnixMilliseconds,
    Rfc3339,
}

/// Canonical representations of one parsed instant.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ParsedInstant {
    pub schema_version: u32,
    pub unix_seconds: i64,
    pub unix_milliseconds: i64,
    pub utc_rfc3339: String,
}

/// Parse an instant without guessing its unit or applying a local time zone.
pub fn parse_instant(kind: InstantInputKind, input: &str) -> Result<ParsedInstant, TimeError> {
    let input = input.trim();
    if input.len() > MAX_TIME_INPUT_BYTES {
        return Err(TimeError::InputTooLong);
    }

    let timestamp = match kind {
        InstantInputKind::UnixSeconds => {
            let seconds = input
                .parse::<i64>()
                .map_err(|_| TimeError::InvalidTimestamp)?;
            Timestamp::from_second(seconds).map_err(|_| TimeError::InvalidTimestamp)?
        }
        InstantInputKind::UnixMilliseconds => {
            let milliseconds = input
                .parse::<i64>()
                .map_err(|_| TimeError::InvalidTimestamp)?;
            Timestamp::from_millisecond(milliseconds).map_err(|_| TimeError::InvalidTimestamp)?
        }
        InstantInputKind::Rfc3339 => parse_rfc3339(input)?,
    };

    Ok(parsed_from_timestamp(timestamp))
}

fn parse_rfc3339(input: &str) -> Result<Timestamp, TimeError> {
    let datetime = DateTime::parse_from_rfc3339(input).map_err(|_| TimeError::InvalidRfc3339)?;
    let fractional_digits = input.split_once('.').map_or(0, |(_, suffix)| {
        suffix.bytes().take_while(u8::is_ascii_digit).count()
    });
    if fractional_digits > 3 || datetime.timestamp_subsec_nanos() % 1_000_000 != 0 {
        return Err(TimeError::InvalidRfc3339);
    }

    Timestamp::from_millisecond(datetime.timestamp_millis()).map_err(|_| TimeError::InvalidRfc3339)
}

pub(crate) fn parsed_from_timestamp(timestamp: Timestamp) -> ParsedInstant {
    let unix_milliseconds = timestamp.as_millisecond();

    ParsedInstant {
        schema_version: 1,
        unix_seconds: unix_milliseconds.div_euclid(1_000),
        unix_milliseconds,
        utc_rfc3339: timestamp.to_string(),
    }
}
