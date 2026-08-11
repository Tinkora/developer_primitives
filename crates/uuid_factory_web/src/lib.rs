//! WASM bridge for the shared identifier core.

use uuid_factory_core::{self as core, CoreError};
use wasm_bindgen::prelude::*;

fn core_error(error: CoreError) -> JsValue {
    let object = js_sys::Object::new();
    let code_set = js_sys::Reflect::set(
        &object,
        &JsValue::from_str("code"),
        &JsValue::from_str(error.code()),
    );
    let message_set = js_sys::Reflect::set(
        &object,
        &JsValue::from_str("message"),
        &JsValue::from_str(&error.to_string()),
    );

    if code_set.is_err() || message_set.is_err() {
        return JsValue::from_str(error.code());
    }

    object.into()
}

fn serialize<T: serde::Serialize>(value: &T) -> Result<JsValue, JsValue> {
    serde_wasm_bindgen::to_value(value).map_err(|_| core_error(CoreError::SerializationFailed))
}

fn parse_kind(kind: &str) -> Result<core::IdKind, JsValue> {
    match kind {
        "uuid-v4" => Ok(core::IdKind::UuidV4),
        "uuid-v7" => Ok(core::IdKind::UuidV7),
        "ulid" => Ok(core::IdKind::Ulid),
        _ => Err(core_error(CoreError::UnsupportedKind)),
    }
}

/// Generate one UUID v4, UUID v7, or ULID identifier.
#[wasm_bindgen]
pub fn generate(kind: &str) -> Result<String, JsValue> {
    match parse_kind(kind)? {
        core::IdKind::UuidV4 => core::generate_uuid(core::UuidVersion::V4).map_err(core_error),
        core::IdKind::UuidV7 => core::generate_uuid(core::UuidVersion::V7).map_err(core_error),
        core::IdKind::Ulid => core::generate_ulid().map_err(core_error),
    }
}

/// Generate an ordered JavaScript array of identifiers.
#[wasm_bindgen]
pub fn batch_generate(kind: &str, count: u32) -> Result<JsValue, JsValue> {
    let identifiers = core::batch_generate(parse_kind(kind)?, count).map_err(core_error)?;
    serialize(&identifiers)
}

/// Inspect a UUID or canonical uppercase ULID.
#[wasm_bindgen]
pub fn inspect_identifier(input: &str) -> Result<JsValue, JsValue> {
    let inspection = core::inspect_identifier(input).map_err(core_error)?;
    serialize(&inspection)
}
