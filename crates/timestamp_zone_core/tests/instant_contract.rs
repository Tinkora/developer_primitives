use timestamp_zone_core::{InstantInputKind, TimeError, parse_instant};

#[test]
fn parses_explicit_seconds_without_unit_guessing() {
    let value = parse_instant(InstantInputKind::UnixSeconds, "0").unwrap();

    assert_eq!(value.schema_version, 1);
    assert_eq!(value.unix_seconds, 0);
    assert_eq!(value.unix_milliseconds, 0);
    assert_eq!(value.utc_rfc3339, "1970-01-01T00:00:00Z");
}

#[test]
fn parses_negative_milliseconds_as_the_instant_before_epoch() {
    let value = parse_instant(InstantInputKind::UnixMilliseconds, "-1").unwrap();

    assert_eq!(value.unix_seconds, -1);
    assert_eq!(value.unix_milliseconds, -1);
    assert_eq!(value.utc_rfc3339, "1969-12-31T23:59:59.999Z");
}

#[test]
fn preserves_rfc3339_millisecond_precision() {
    let value = parse_instant(InstantInputKind::Rfc3339, "2026-11-01T01:30:00.125-04:00").unwrap();

    assert_eq!(value.unix_seconds, 1_793_511_000);
    assert_eq!(value.unix_milliseconds, 1_793_511_000_125);
    assert_eq!(value.utc_rfc3339, "2026-11-01T05:30:00.125Z");
}

#[test]
fn rejects_naive_rfc3339_input() {
    let error = parse_instant(InstantInputKind::Rfc3339, "2026-11-01T01:30:00").unwrap_err();

    assert_eq!(error, TimeError::InvalidRfc3339);
    assert_eq!(error.code(), "INVALID_RFC3339");
}

#[test]
fn rejects_sub_millisecond_rfc3339_precision() {
    let error = parse_instant(
        InstantInputKind::Rfc3339,
        "2026-11-01T01:30:00.000001-04:00",
    )
    .unwrap_err();

    assert_eq!(error, TimeError::InvalidRfc3339);
}

#[test]
fn rejects_rfc3339_fraction_with_more_than_three_digits_even_when_trailing_zero() {
    let error = parse_instant(
        InstantInputKind::Rfc3339,
        "2026-11-01T01:30:00.123000-04:00",
    )
    .unwrap_err();

    assert_eq!(error, TimeError::InvalidRfc3339);
}

#[test]
fn rejects_integer_overflow() {
    let error = parse_instant(InstantInputKind::UnixSeconds, "9223372036854775808").unwrap_err();

    assert_eq!(error, TimeError::InvalidTimestamp);
    assert_eq!(error.code(), "INVALID_TIMESTAMP");
}

#[test]
fn rejects_whitespace_only_numeric_input() {
    let error = parse_instant(InstantInputKind::UnixMilliseconds, "  \n").unwrap_err();

    assert_eq!(error, TimeError::InvalidTimestamp);
}

#[test]
fn rejects_input_longer_than_the_public_limit() {
    let input = "1".repeat(129);
    let error = parse_instant(InstantInputKind::UnixSeconds, &input).unwrap_err();

    assert_eq!(error, TimeError::InputTooLong);
    assert_eq!(error.code(), "INPUT_TOO_LONG");
}
