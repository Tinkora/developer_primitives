use crate::error::CoreError;

/// UUID version selector.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UuidVersion {
    /// Random (v4) — unguessable, no embedded timestamp.
    V4,
    /// Time-ordered (v7) — sortable, includes millisecond timestamp.
    V7,
}

/// Identifier type for batch generation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IdKind {
    /// UUID v4 — 36-char hex with hyphens.
    UuidV4,
    /// UUID v7 — 36-char hex with hyphens, time-ordered.
    UuidV7,
    /// ULID — 26-char Crockford Base32, time-ordered.
    Ulid,
}

trait GenerationContext {
    fn timestamp_millis(&mut self) -> Result<u64, CoreError>;
    fn fill_random(&mut self, destination: &mut [u8]) -> Result<(), CoreError>;
}

struct SystemGenerationContext;

fn unix_timestamp_millis(now: web_time::SystemTime) -> Result<u64, CoreError> {
    let elapsed = now
        .duration_since(web_time::UNIX_EPOCH)
        .map_err(|_| CoreError::ClockUnavailable)?;
    u64::try_from(elapsed.as_millis()).map_err(|_| CoreError::ClockUnavailable)
}

impl GenerationContext for SystemGenerationContext {
    fn timestamp_millis(&mut self) -> Result<u64, CoreError> {
        unix_timestamp_millis(web_time::SystemTime::now())
    }

    fn fill_random(&mut self, destination: &mut [u8]) -> Result<(), CoreError> {
        getrandom::fill(destination).map_err(|_| CoreError::RandomUnavailable)
    }
}

fn generate_uuid_with(
    version: UuidVersion,
    context: &mut impl GenerationContext,
) -> Result<String, CoreError> {
    match version {
        UuidVersion::V4 => {
            let mut random_bytes = [0_u8; 16];
            context.fill_random(&mut random_bytes)?;
            Ok(uuid::Builder::from_random_bytes(random_bytes)
                .into_uuid()
                .to_string())
        }
        UuidVersion::V7 => {
            let timestamp_ms = context.timestamp_millis()?;
            let mut random_bytes = [0_u8; 10];
            context.fill_random(&mut random_bytes)?;
            Ok(
                uuid::Builder::from_unix_timestamp_millis(timestamp_ms, &random_bytes)
                    .into_uuid()
                    .to_string(),
            )
        }
    }
}

fn generate_ulid_with(context: &mut impl GenerationContext) -> Result<String, CoreError> {
    let timestamp_ms = context.timestamp_millis()?;
    let mut random_bytes = [0_u8; 10];
    context.fill_random(&mut random_bytes)?;

    let mut random_value = [0_u8; 16];
    random_value[6..].copy_from_slice(&random_bytes);

    Ok(ulid::Ulid::from_parts(timestamp_ms, u128::from_be_bytes(random_value)).to_string())
}

/// Generate a single UUID string (lowercase, with hyphens).
pub fn generate_uuid(version: UuidVersion) -> Result<String, CoreError> {
    generate_uuid_with(version, &mut SystemGenerationContext)
}

/// Generate a single ULID string (26-char Crockford Base32).
pub fn generate_ulid() -> Result<String, CoreError> {
    generate_ulid_with(&mut SystemGenerationContext)
}

fn batch_generate_with(
    kind: IdKind,
    count: u32,
    context: &mut impl GenerationContext,
) -> Result<Vec<String>, CoreError> {
    const MAX_BATCH: u32 = 10_000;

    if !(1..=MAX_BATCH).contains(&count) {
        return Err(CoreError::BatchOutOfRange(count));
    }

    let mut ids = Vec::with_capacity(count as usize);
    for _ in 0..count {
        let id = match kind {
            IdKind::UuidV4 => generate_uuid_with(UuidVersion::V4, context)?,
            IdKind::UuidV7 => generate_uuid_with(UuidVersion::V7, context)?,
            IdKind::Ulid => generate_ulid_with(context)?,
        };
        ids.push(id);
    }

    Ok(ids)
}

/// Batch-generate identifiers of a given kind.
///
/// Capped at 10,000 to prevent browser tab freezes.
pub fn batch_generate(kind: IdKind, count: u32) -> Result<Vec<String>, CoreError> {
    batch_generate_with(kind, count, &mut SystemGenerationContext)
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestContext {
        timestamp_ms: Result<u64, CoreError>,
        random_bytes: Result<[u8; 16], CoreError>,
    }

    impl TestContext {
        fn fixed(timestamp_ms: u64, random_bytes: [u8; 16]) -> Self {
            Self {
                timestamp_ms: Ok(timestamp_ms),
                random_bytes: Ok(random_bytes),
            }
        }
    }

    impl GenerationContext for TestContext {
        fn timestamp_millis(&mut self) -> Result<u64, CoreError> {
            self.timestamp_ms.clone()
        }

        fn fill_random(&mut self, destination: &mut [u8]) -> Result<(), CoreError> {
            let source = self.random_bytes.clone()?;
            destination.copy_from_slice(&source[..destination.len()]);
            Ok(())
        }
    }

    #[test]
    fn fixed_random_bytes_produce_uuid_v4_layout() {
        let mut context = TestContext::fixed(0, [0xaa; 16]);
        let id = generate_uuid_with(UuidVersion::V4, &mut context).unwrap();

        assert_eq!(id, "aaaaaaaa-aaaa-4aaa-aaaa-aaaaaaaaaaaa");
    }

    #[test]
    fn fixed_timestamp_and_random_bytes_produce_uuid_v7_layout() {
        let mut context = TestContext::fixed(0x0123_4567_89ab, [0xcd; 16]);
        let id = generate_uuid_with(UuidVersion::V7, &mut context).unwrap();

        assert_eq!(id, "01234567-89ab-7dcd-8dcd-cdcdcdcdcdcd");
    }

    #[test]
    fn fixed_timestamp_and_random_bytes_produce_ulid_layout() {
        let mut context = TestContext::fixed(0x0123_4567_89ab, [0xef; 16]);
        let id = generate_ulid_with(&mut context).unwrap();
        let parsed = id.parse::<ulid::Ulid>().unwrap();

        assert_eq!(parsed.timestamp_ms(), 0x0123_4567_89ab);
        assert_eq!(
            parsed.random(),
            u128::from_be_bytes([
                0, 0, 0, 0, 0, 0, 0xef, 0xef, 0xef, 0xef, 0xef, 0xef, 0xef, 0xef, 0xef, 0xef,
            ])
        );
    }

    #[test]
    fn random_source_failure_is_returned() {
        let mut context = TestContext {
            timestamp_ms: Ok(0),
            random_bytes: Err(CoreError::RandomUnavailable),
        };

        let error = generate_uuid_with(UuidVersion::V4, &mut context).unwrap_err();
        assert_eq!(error.code(), "RANDOM_UNAVAILABLE");
    }

    #[test]
    fn pre_epoch_clock_failure_is_returned() {
        let mut context = TestContext {
            timestamp_ms: Err(CoreError::ClockUnavailable),
            random_bytes: Ok([0; 16]),
        };

        let error = generate_uuid_with(UuidVersion::V7, &mut context).unwrap_err();
        assert_eq!(error.code(), "CLOCK_UNAVAILABLE");
    }

    #[test]
    fn pre_epoch_system_time_is_rejected() {
        let before_epoch = std::time::UNIX_EPOCH
            .checked_sub(std::time::Duration::from_millis(1))
            .unwrap();

        let error = unix_timestamp_millis(before_epoch).unwrap_err();
        assert_eq!(error.code(), "CLOCK_UNAVAILABLE");
    }

    #[test]
    fn batch_generation_propagates_source_failures() {
        let mut context = TestContext {
            timestamp_ms: Ok(0),
            random_bytes: Err(CoreError::RandomUnavailable),
        };

        let error = batch_generate_with(IdKind::UuidV4, 1, &mut context).unwrap_err();
        assert_eq!(error.code(), "RANDOM_UNAVAILABLE");
    }

    #[test]
    fn test_generate_uuid_v4() {
        let generated: Result<String, CoreError> = generate_uuid(UuidVersion::V4);
        let id = generated.unwrap();
        assert_eq!(id.len(), 36);
        assert!(id.chars().filter(|&c| c == '-').count() == 4);
        // v4 has version nibble = 4
        assert_eq!(&id[14..15], "4");
    }

    #[test]
    fn test_generate_uuid_v7() {
        let id = generate_uuid(UuidVersion::V7).unwrap();
        assert_eq!(id.len(), 36);
        // v7 has version nibble = 7
        assert_eq!(&id[14..15], "7");
    }

    #[test]
    fn test_generate_ulid() {
        let generated: Result<String, CoreError> = generate_ulid();
        let id = generated.unwrap();
        assert_eq!(id.len(), 26);
        // ULID uses Crockford Base32 (uppercase letters + digits, no ILOU)
        assert!(
            id.chars()
                .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit())
        );
    }

    #[test]
    fn test_batch_generate_uuid_v4() {
        let ids = batch_generate(IdKind::UuidV4, 5).unwrap();
        assert_eq!(ids.len(), 5);
        for id in &ids {
            assert_eq!(id.len(), 36);
        }
    }

    #[test]
    fn test_batch_generate_ulid() {
        let ids = batch_generate(IdKind::Ulid, 3).unwrap();
        assert_eq!(ids.len(), 3);
        for id in &ids {
            assert_eq!(id.len(), 26);
        }
    }

    #[test]
    fn test_batch_too_large() {
        let error = batch_generate(IdKind::UuidV4, 10_001).unwrap_err();
        assert_eq!(error.code(), "BATCH_OUT_OF_RANGE");
    }

    #[test]
    fn test_batch_max_ok() {
        let result = batch_generate(IdKind::UuidV4, 10_000);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 10_000);
    }

    #[test]
    fn test_zero_count_is_rejected() {
        let error = batch_generate(IdKind::UuidV4, 0).unwrap_err();
        assert_eq!(error.code(), "BATCH_OUT_OF_RANGE");
    }

    #[test]
    fn test_uniqueness() {
        let ids = batch_generate(IdKind::UuidV4, 100).unwrap();
        let mut dedup = ids.clone();
        dedup.sort();
        dedup.dedup();
        assert_eq!(ids.len(), dedup.len());
    }
}
