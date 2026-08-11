//! WASM bridge for the shared identifier core.

use timestamp_zone_core::{self as time_core, TimeError};
use uuid_factory_core::{self as core, CoreError};
use wasm_bindgen::prelude::*;

fn error_object(code: &str, message: &str) -> JsValue {
    let object = js_sys::Object::new();
    let code_set = js_sys::Reflect::set(
        &object,
        &JsValue::from_str("code"),
        &JsValue::from_str(code),
    );
    let message_set = js_sys::Reflect::set(
        &object,
        &JsValue::from_str("message"),
        &JsValue::from_str(message),
    );

    if code_set.is_err() || message_set.is_err() {
        return JsValue::from_str(code);
    }

    object.into()
}

fn core_error(error: CoreError) -> JsValue {
    error_object(error.code(), &error.to_string())
}

fn time_error(error: TimeError) -> JsValue {
    error_object(error.code(), &error.to_string())
}

fn serialize<T: serde::Serialize>(value: &T) -> Result<JsValue, JsValue> {
    serde_wasm_bindgen::to_value(value).map_err(|_| core_error(CoreError::SerializationFailed))
}

fn serialize_time<T: serde::Serialize>(value: &T) -> Result<JsValue, JsValue> {
    serde_wasm_bindgen::to_value(value).map_err(|_| time_error(TimeError::SerializationFailed))
}

fn parse_kind(kind: &str) -> Result<core::IdKind, JsValue> {
    match kind {
        "uuid-v4" => Ok(core::IdKind::UuidV4),
        "uuid-v7" => Ok(core::IdKind::UuidV7),
        "ulid" => Ok(core::IdKind::Ulid),
        _ => Err(core_error(CoreError::UnsupportedKind)),
    }
}

fn parse_time_kind(kind: &str) -> Result<time_core::InstantInputKind, JsValue> {
    match kind {
        "unix-seconds" => Ok(time_core::InstantInputKind::UnixSeconds),
        "unix-milliseconds" => Ok(time_core::InstantInputKind::UnixMilliseconds),
        "rfc3339" => Ok(time_core::InstantInputKind::Rfc3339),
        _ => Err(time_error(TimeError::InvalidTimestamp)),
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

/// Convert an explicitly typed instant into ordered IANA time-zone results.
#[wasm_bindgen]
pub fn convert_timestamp(kind: &str, input: &str, zones: JsValue) -> Result<JsValue, JsValue> {
    let zones: Vec<String> = serde_wasm_bindgen::from_value(zones)
        .map_err(|_| time_error(TimeError::InvalidTimezone))?;
    let zone_names: Vec<_> = zones.iter().map(String::as_str).collect();
    let result = time_core::convert_instant(parse_time_kind(kind)?, input, &zone_names)
        .map_err(time_error)?;
    serialize_time(&result)
}

/// Resolve a local timestamp without silently choosing a DST gap or fold outcome.
#[wasm_bindgen]
pub fn resolve_local_timestamp(local_datetime: &str, zone: &str) -> Result<JsValue, JsValue> {
    let result = time_core::resolve_local_time(local_datetime, zone).map_err(time_error)?;
    serialize_time(&result)
}

/// Search the bundled IANA time-zone names with a bounded case-insensitive filter.
#[wasm_bindgen]
pub fn search_time_zones(filter: &str) -> Result<JsValue, JsValue> {
    let zones = time_core::search_time_zones(filter).map_err(time_error)?;
    serialize_time(&zones)
}

/// Return the IANA time-zone database version bundled into this WebAssembly module.
#[wasm_bindgen]
pub fn time_zone_database_version() -> String {
    time_core::time_zone_database_version().to_string()
}
