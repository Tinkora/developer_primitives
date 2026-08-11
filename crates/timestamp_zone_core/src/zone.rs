use std::collections::HashSet;

use chrono::{Datelike, NaiveDateTime, Timelike};
use jiff::{
    Timestamp,
    civil::DateTime,
    tz::{AmbiguousOffset, Offset, TimeZone},
};
use serde::Serialize;

use crate::{
    InstantInputKind, MAX_TIME_INPUT_BYTES, MAX_TIMEZONE_COUNT, MAX_TIMEZONE_NAME_BYTES,
    ParsedInstant, TimeError, instant::parsed_from_timestamp, parse_instant,
};

/// One instant rendered in a requested IANA time zone.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ZonedTime {
    pub zone: String,
    pub local_datetime: String,
    pub offset: String,
    pub abbreviation: String,
    pub is_dst: Option<bool>,
}

/// Versioned comparison of one instant across IANA time zones.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct TimeConversion {
    pub schema_version: u32,
    pub tzdb_version: String,
    pub instant: ParsedInstant,
    pub zones: Vec<ZonedTime>,
}

/// One explicit candidate for a local civil time.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CandidateInstant {
    pub unix_seconds: i64,
    pub unix_milliseconds: i64,
    pub utc_rfc3339: String,
    pub offset: String,
    pub abbreviation: String,
    pub is_dst: Option<bool>,
}

/// Result of applying IANA rules to a local civil date and time.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "status", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum LocalResolution {
    Unambiguous {
        instant: CandidateInstant,
    },
    Gap {
        before_offset: String,
        after_offset: String,
    },
    Fold {
        earlier: CandidateInstant,
        later: CandidateInstant,
    },
}

/// Versioned resolution of one civil time in one IANA zone.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct LocalTimeResult {
    pub schema_version: u32,
    pub tzdb_version: String,
    pub zone: String,
    pub local_datetime: String,
    pub resolution: LocalResolution,
}

/// Return the bundled IANA Time Zone Database release.
pub fn time_zone_database_version() -> &'static str {
    jiff_tzdb::VERSION.unwrap_or("unknown")
}

/// Convert one explicitly typed instant into one through eight IANA zones.
pub fn convert_instant(
    kind: InstantInputKind,
    input: &str,
    zone_names: &[&str],
) -> Result<TimeConversion, TimeError> {
    if !(1..=MAX_TIMEZONE_COUNT).contains(&zone_names.len()) {
        return Err(TimeError::TimezoneLimitExceeded);
    }

    let instant = parse_instant(kind, input)?;
    let timestamp = Timestamp::from_millisecond(instant.unix_milliseconds)
        .map_err(|_| TimeError::InvalidTimestamp)?;
    let mut seen = HashSet::with_capacity(zone_names.len());
    let mut zones = Vec::with_capacity(zone_names.len());

    for name in zone_names {
        let (zone, canonical_name) = load_zone(name)?;
        if !seen.insert(canonical_name.to_ascii_lowercase()) {
            return Err(TimeError::DuplicateTimezone);
        }
        zones.push(zoned_time(timestamp, &zone, &canonical_name));
    }

    Ok(TimeConversion {
        schema_version: 1,
        tzdb_version: time_zone_database_version().to_string(),
        instant,
        zones,
    })
}

/// Resolve a local civil time without silently changing a gap or choosing a fold.
pub fn resolve_local_time(
    local_datetime: &str,
    zone_name: &str,
) -> Result<LocalTimeResult, TimeError> {
    let input = local_datetime.trim();
    if input.len() > MAX_TIME_INPUT_BYTES {
        return Err(TimeError::InputTooLong);
    }
    let civil = parse_local_datetime(input)?;
    let (zone, canonical_name) = load_zone(zone_name)?;
    let ambiguous = zone.to_ambiguous_timestamp(civil);

    let resolution = match ambiguous.offset() {
        AmbiguousOffset::Unambiguous { .. } => {
            let timestamp = ambiguous
                .unambiguous()
                .map_err(|_| TimeError::InvalidLocalDateTime)?;
            LocalResolution::Unambiguous {
                instant: candidate(timestamp, &zone),
            }
        }
        AmbiguousOffset::Gap { before, after } => LocalResolution::Gap {
            before_offset: format_offset(before),
            after_offset: format_offset(after),
        },
        AmbiguousOffset::Fold { .. } => {
            let earlier = ambiguous
                .earlier()
                .map_err(|_| TimeError::InvalidLocalDateTime)?;
            let later = ambiguous
                .later()
                .map_err(|_| TimeError::InvalidLocalDateTime)?;
            LocalResolution::Fold {
                earlier: candidate(earlier, &zone),
                later: candidate(later, &zone),
            }
        }
    };

    Ok(LocalTimeResult {
        schema_version: 1,
        tzdb_version: time_zone_database_version().to_string(),
        zone: canonical_name,
        local_datetime: civil.to_string(),
        resolution,
    })
}

fn load_zone(name: &str) -> Result<(TimeZone, String), TimeError> {
    let name = name.trim();
    if name.is_empty() || name.len() > MAX_TIMEZONE_NAME_BYTES || !name.is_ascii() {
        return Err(TimeError::InvalidTimezone);
    }

    let zone = TimeZone::get(name).map_err(|_| TimeError::InvalidTimezone)?;
    let canonical_name = zone
        .iana_name()
        .or_else(|| name.eq_ignore_ascii_case("UTC").then_some("UTC"))
        .ok_or(TimeError::InvalidTimezone)?
        .to_string();
    Ok((zone, canonical_name))
}

fn parse_local_datetime(input: &str) -> Result<DateTime, TimeError> {
    let datetime = NaiveDateTime::parse_from_str(input, "%Y-%m-%dT%H:%M:%S")
        .map_err(|_| TimeError::InvalidLocalDateTime)?;
    let year = i16::try_from(datetime.year()).map_err(|_| TimeError::InvalidLocalDateTime)?;

    DateTime::new(
        year,
        datetime.month() as i8,
        datetime.day() as i8,
        datetime.hour() as i8,
        datetime.minute() as i8,
        datetime.second() as i8,
        0,
    )
    .map_err(|_| TimeError::InvalidLocalDateTime)
}

fn zoned_time(timestamp: Timestamp, zone: &TimeZone, canonical_name: &str) -> ZonedTime {
    let info = zone.to_offset_info(timestamp);
    ZonedTime {
        zone: canonical_name.to_string(),
        local_datetime: info.offset().to_datetime(timestamp).to_string(),
        offset: format_offset(info.offset()),
        abbreviation: info.abbreviation().to_string(),
        is_dst: Some(info.dst().is_dst()),
    }
}

fn candidate(timestamp: Timestamp, zone: &TimeZone) -> CandidateInstant {
    let parsed = parsed_from_timestamp(timestamp);
    let info = zone.to_offset_info(timestamp);
    CandidateInstant {
        unix_seconds: parsed.unix_seconds,
        unix_milliseconds: parsed.unix_milliseconds,
        utc_rfc3339: parsed.utc_rfc3339,
        offset: format_offset(info.offset()),
        abbreviation: info.abbreviation().to_string(),
        is_dst: Some(info.dst().is_dst()),
    }
}

fn format_offset(offset: Offset) -> String {
    let total_seconds = offset.seconds();
    let sign = if total_seconds < 0 { '-' } else { '+' };
    let absolute = total_seconds.unsigned_abs();
    let hours = absolute / 3_600;
    let minutes = (absolute % 3_600) / 60;
    let seconds = absolute % 60;

    if seconds == 0 {
        format!("{sign}{hours:02}:{minutes:02}")
    } else {
        format!("{sign}{hours:02}:{minutes:02}:{seconds:02}")
    }
}
