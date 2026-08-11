use crate::error::CoreError;
use serde::Serialize;

/// Maximum UTF-8 byte length accepted by identifier inspection.
pub const MAX_IDENTIFIER_INPUT_LEN: usize = 128;

/// Parsed UUID metadata, suitable for serialization to JSON.
#[derive(Clone, Debug, Serialize)]
pub struct ParsedUuid {
    /// UUID version (4 or 7), or `None` for nil/unknown.
    pub version: Option<u32>,
    /// RFC variant name.
    pub variant: String,
    /// Unix timestamp in milliseconds (only for v7), or `None`.
    pub timestamp: Option<i64>,
}

/// Versioned metadata returned by strict identifier inspection.
#[derive(Clone, Debug, Serialize)]
pub struct IdentifierInspection {
    pub schema_version: u32,
    pub input: String,
    pub canonical: String,
    pub kind: String,
    pub version: Option<u32>,
    pub variant: Option<String>,
    pub timestamp_ms: Option<u64>,
}

/// Inspect a supported identifier and return its canonical metadata.
pub fn inspect_identifier(input: &str) -> Result<IdentifierInspection, CoreError> {
    if input.len() > MAX_IDENTIFIER_INPUT_LEN {
        return Err(CoreError::InvalidIdentifier);
    }

    if input.len() == 26 {
        if input
            .chars()
            .all(|character| character.is_ascii_uppercase() || character.is_ascii_digit())
            && let Ok(ulid) = input.parse::<ulid::Ulid>()
            && ulid.to_string() == input
        {
            return Ok(IdentifierInspection {
                schema_version: 1,
                input: input.to_string(),
                canonical: ulid.to_string(),
                kind: "ulid".to_string(),
                version: None,
                variant: None,
                timestamp_ms: Some(ulid.timestamp_ms()),
            });
        }

        return Err(CoreError::InvalidUlid);
    }

    let uuid = uuid::Uuid::parse_str(input).map_err(|_| {
        if input.len() == 36 && input.chars().filter(|character| *character == '-').count() == 4 {
            CoreError::InvalidUuid
        } else {
            CoreError::InvalidIdentifier
        }
    })?;
    let parsed = parse_uuid(input)?;

    Ok(IdentifierInspection {
        schema_version: 1,
        input: input.to_string(),
        canonical: uuid.to_string(),
        kind: "uuid".to_string(),
        version: parsed.version,
        variant: Some(parsed.variant),
        timestamp_ms: parsed.timestamp.and_then(|value| u64::try_from(value).ok()),
    })
}

/// Parse a UUID string and extract version, variant, and v7 timestamp.
pub fn parse_uuid(uuid_str: &str) -> Result<ParsedUuid, CoreError> {
    let u = uuid::Uuid::parse_str(uuid_str).map_err(|_| CoreError::InvalidUuid)?;

    let version_num = u.get_version_num();

    let version = if version_num == 0 {
        None
    } else {
        // The upstream value comes from a four-bit UUID version field.
        Some(u32::try_from(version_num).expect("UUID version nibble must fit in u32"))
    };

    let variant = match u.get_variant() {
        uuid::Variant::RFC4122 => "RFC4122".to_string(),
        uuid::Variant::Future => "Future".to_string(),
        uuid::Variant::Microsoft => "Microsoft".to_string(),
        uuid::Variant::NCS => "NCS".to_string(),
        _ => "Unknown".to_string(),
    };

    let timestamp = if version_num == 7 {
        u.get_timestamp().map(|ts| {
            let (secs, nanos) = ts.to_unix();
            (secs as i64) * 1000 + (nanos as i64 / 1_000_000)
        })
    } else {
        None
    };

    Ok(ParsedUuid {
        version,
        variant,
        timestamp,
    })
}

/// Validate that a string is a well-formed UUID.
pub fn validate_uuid(uuid_str: &str) -> Result<(), CoreError> {
    uuid::Uuid::parse_str(uuid_str).map_err(|_| CoreError::InvalidUuid)?;
    Ok(())
}

/// Validate that a string is a well-formed ULID.
pub fn validate_ulid(ulid_str: &str) -> Result<(), CoreError> {
    ulid_str
        .parse::<ulid::Ulid>()
        .map_err(|_| CoreError::InvalidUlid)?;
    Ok(())
}

/// Detect whether a string looks like a UUID or ULID, without full validation.
///
/// Returns `("uuid"|"ulid"|"unknown", is_valid)`.
pub fn detect_kind(id_str: &str) -> (&'static str, bool) {
    // ULIDs are exactly 26 chars and use Crockford Base32.
    if id_str.len() == 26 {
        let looks_like_base32 = id_str
            .chars()
            .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit());
        if looks_like_base32 {
            let valid = id_str.parse::<ulid::Ulid>().is_ok();
            return ("ulid", valid);
        }
    }

    // UUIDs are 36 chars with 4 hyphens.
    if id_str.len() == 36 && id_str.chars().filter(|&c| c == '-').count() == 4 {
        if uuid::Uuid::parse_str(id_str).is_ok() {
            return ("uuid", true);
        }
        return ("uuid", false);
    }

    ("unknown", false)
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- parse_uuid ---

    #[test]
    fn test_parse_v4() {
        let id = crate::generate::generate_uuid(crate::generate::UuidVersion::V4).unwrap();
        let parsed = parse_uuid(&id).unwrap();
        let _: Option<u32> = parsed.version;
        assert_eq!(parsed.version, Some(4));
        assert_eq!(parsed.variant, "RFC4122");
        assert!(parsed.timestamp.is_none());
    }

    #[test]
    fn inspect_canonical_uuid() {
        let input = "550e8400-e29b-41d4-a716-446655440000";
        let inspection = inspect_identifier(input).unwrap();

        assert_eq!(inspection.schema_version, 1);
        assert_eq!(inspection.input, input);
        assert_eq!(inspection.canonical, input);
        assert_eq!(inspection.kind, "uuid");
        assert_eq!(inspection.version, Some(4));
        assert_eq!(inspection.variant.as_deref(), Some("RFC4122"));
        assert_eq!(inspection.timestamp_ms, None);
    }

    #[test]
    fn inspect_uuid_v7_timestamp() {
        let inspection = inspect_identifier("01234567-89ab-7dcd-8dcd-cdcdcdcdcdcd").unwrap();

        assert_eq!(inspection.version, Some(7));
        assert_eq!(inspection.timestamp_ms, Some(0x0123_4567_89ab));
    }

    #[test]
    fn inspect_canonical_ulid() {
        let input = "01ARZ3NDEKTSV4RRFFQ69G5FAV";
        let inspection = inspect_identifier(input).unwrap();

        assert_eq!(inspection.schema_version, 1);
        assert_eq!(inspection.input, input);
        assert_eq!(inspection.canonical, input);
        assert_eq!(inspection.kind, "ulid");
        assert_eq!(inspection.version, None);
        assert_eq!(inspection.variant, None);
        assert_eq!(inspection.timestamp_ms, Some(1_469_922_850_259));
    }

    #[test]
    fn inspect_rejects_lowercase_ulid() {
        let error = inspect_identifier("01arz3ndektsv4rrffq69g5fav").unwrap_err();
        assert_eq!(error.code(), "INVALID_ULID");
    }

    #[test]
    fn inspect_rejects_overlong_input_before_parsing() {
        assert_eq!(MAX_IDENTIFIER_INPUT_LEN, 128);
        let input = "A".repeat(MAX_IDENTIFIER_INPUT_LEN + 1);

        let error = inspect_identifier(&input).unwrap_err();
        assert_eq!(error.code(), "INVALID_IDENTIFIER");
    }

    #[test]
    fn inspect_classifies_malformed_uuid_like_input() {
        let error = inspect_identifier("xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx").unwrap_err();
        assert_eq!(error.code(), "INVALID_UUID");
    }

    #[test]
    fn inspect_rejects_unknown_input() {
        let error = inspect_identifier("hello").unwrap_err();
        assert_eq!(error.code(), "INVALID_IDENTIFIER");
    }

    #[test]
    fn inspection_errors_do_not_echo_invalid_input() {
        let input = "xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx";
        let message = inspect_identifier(input).unwrap_err().to_string();

        assert!(!message.contains(input));
    }

    #[test]
    fn test_parse_v7() {
        let id = crate::generate::generate_uuid(crate::generate::UuidVersion::V7).unwrap();
        let parsed = parse_uuid(&id).unwrap();
        assert_eq!(parsed.version, Some(7));
        assert_eq!(parsed.variant, "RFC4122");
        assert!(parsed.timestamp.is_some());
        // timestamp should be within the last minute
        let now_ms = chrono::Utc::now().timestamp_millis();
        let ts = parsed.timestamp.unwrap();
        assert!((now_ms - ts).abs() < 60_000);
    }

    #[test]
    fn test_parse_nil() {
        let id = "00000000-0000-0000-0000-000000000000";
        let parsed = parse_uuid(id).unwrap();
        assert_eq!(parsed.version, None);
        assert!(parsed.timestamp.is_none());
    }

    #[test]
    fn test_parse_invalid() {
        let result = parse_uuid("not-a-uuid");
        assert!(result.is_err());
    }

    // --- validate_uuid ---

    #[test]
    fn test_validate_uuid_ok() {
        let id = crate::generate::generate_uuid(crate::generate::UuidVersion::V4).unwrap();
        assert!(validate_uuid(&id).is_ok());
    }

    #[test]
    fn test_validate_uuid_err() {
        assert!(validate_uuid("bad").is_err());
    }

    #[test]
    fn test_validate_uuid_uppercase() {
        let id = crate::generate::generate_uuid(crate::generate::UuidVersion::V4)
            .unwrap()
            .to_uppercase();
        assert!(validate_uuid(&id).is_ok());
    }

    // --- validate_ulid ---

    #[test]
    fn test_validate_ulid_ok() {
        let id = crate::generate::generate_ulid().unwrap();
        assert!(validate_ulid(&id).is_ok());
    }

    #[test]
    fn test_validate_ulid_err() {
        assert!(validate_ulid("too-short").is_err());
    }

    // --- detect_kind ---

    #[test]
    fn test_detect_uuid() {
        let id = crate::generate::generate_uuid(crate::generate::UuidVersion::V4).unwrap();
        let (kind, valid) = detect_kind(&id);
        assert_eq!(kind, "uuid");
        assert!(valid);
    }

    #[test]
    fn test_detect_ulid() {
        let id = crate::generate::generate_ulid().unwrap();
        let (kind, valid) = detect_kind(&id);
        assert_eq!(kind, "ulid");
        assert!(valid);
    }

    #[test]
    fn test_detect_malformed_uuid() {
        // 36 chars but wrong structure
        let (kind, valid) = detect_kind("xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx");
        assert_eq!(kind, "uuid");
        assert!(!valid);
    }

    #[test]
    fn test_detect_malformed_ulid() {
        // 26 chars that look like base32 but contain invalid ULID characters (I, L, O, U)
        let (kind, valid) = detect_kind("IIIIIIIIIIIIIIIIIIIIIIIIII");
        assert_eq!(kind, "ulid");
        assert!(!valid);
    }

    #[test]
    fn test_detect_unknown() {
        let (kind, valid) = detect_kind("hello");
        assert_eq!(kind, "unknown");
        assert!(!valid);
    }
}
