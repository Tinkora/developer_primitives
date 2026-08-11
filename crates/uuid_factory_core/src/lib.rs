pub mod error;
pub mod generate;
pub mod validate;

pub use error::CoreError;
pub use generate::{IdKind, UuidVersion, batch_generate, generate_ulid, generate_uuid};
pub use validate::{
    IdentifierInspection, MAX_IDENTIFIER_INPUT_LEN, ParsedUuid, detect_kind, inspect_identifier,
    parse_uuid, validate_ulid, validate_uuid,
};
