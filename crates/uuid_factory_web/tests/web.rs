#![cfg(target_arch = "wasm32")]

use js_sys::{Array, Reflect};
use uuid_factory_web::{
    batch_generate, convert_timestamp, generate, inspect_identifier, resolve_local_timestamp,
    search_time_zones, time_zone_database_version,
};
use wasm_bindgen::JsValue;
use wasm_bindgen_test::wasm_bindgen_test;

fn string_field(value: &JsValue, name: &str) -> String {
    Reflect::get(value, &JsValue::from_str(name))
        .unwrap()
        .as_string()
        .unwrap()
}

fn value_field(value: &JsValue, name: &str) -> JsValue {
    Reflect::get(value, &JsValue::from_str(name)).unwrap()
}

#[wasm_bindgen_test]
fn generation_success() {
    let identifier = generate("uuid-v4").unwrap();

    assert_eq!(identifier.len(), 36);
    assert_eq!(&identifier[14..15], "4");
}

#[wasm_bindgen_test]
fn invalid_batch_count_returns_stable_error() {
    let error = batch_generate("uuid-v4", 0).unwrap_err();

    assert_eq!(string_field(&error, "code"), "BATCH_OUT_OF_RANGE");
    assert!(!string_field(&error, "message").is_empty());
}

#[wasm_bindgen_test]
fn batch_generation_success() {
    let result = batch_generate("ulid", 2).unwrap();
    let identifiers = Array::from(&result);

    assert_eq!(identifiers.length(), 2);
}

#[wasm_bindgen_test]
fn inspection_success() {
    let result = inspect_identifier("550e8400-e29b-41d4-a716-446655440000").unwrap();

    assert_eq!(string_field(&result, "kind"), "uuid");
    assert_eq!(
        string_field(&result, "canonical"),
        "550e8400-e29b-41d4-a716-446655440000"
    );
}

#[wasm_bindgen_test]
fn invalid_inspection_returns_stable_error() {
    let error = inspect_identifier("not-an-id").unwrap_err();

    assert_eq!(string_field(&error, "code"), "INVALID_IDENTIFIER");
}

#[wasm_bindgen_test]
fn unsupported_kind_returns_stable_error() {
    let error = generate("uuid-v5").unwrap_err();

    assert_eq!(string_field(&error, "code"), "UNSUPPORTED_KIND");
}

#[wasm_bindgen_test]
fn timestamp_conversion_preserves_requested_zone_order() {
    let zones = serde_wasm_bindgen::to_value(&vec!["UTC", "Asia/Shanghai"]).unwrap();
    let result = convert_timestamp("unix-seconds", "0", zones).unwrap();

    assert_eq!(string_field(&result, "tzdb_version"), "2026c");
    let zone_values = Array::from(&value_field(&result, "zones"));
    assert_eq!(string_field(&zone_values.get(0), "zone"), "UTC");
    assert_eq!(string_field(&zone_values.get(1), "zone"), "Asia/Shanghai");
}

#[wasm_bindgen_test]
fn local_timestamp_resolution_reports_a_gap_without_an_invented_instant() {
    let result = resolve_local_timestamp("2026-03-08T02:30:00", "America/New_York").unwrap();
    let resolution = value_field(&result, "resolution");

    assert_eq!(string_field(&resolution, "status"), "GAP");
    assert_eq!(string_field(&resolution, "before_offset"), "-05:00");
    assert_eq!(string_field(&resolution, "after_offset"), "-04:00");
}

#[wasm_bindgen_test]
fn time_zone_search_and_database_version_are_exposed() {
    let result = search_time_zones("shanghai").unwrap();
    let zones = Array::from(&result);

    assert!(
        zones
            .to_vec()
            .iter()
            .filter_map(JsValue::as_string)
            .any(|zone| zone == "Asia/Shanghai")
    );
    assert_eq!(time_zone_database_version(), "2026c");
}

#[wasm_bindgen_test]
fn invalid_time_zone_returns_a_stable_time_error_object() {
    let zones = serde_wasm_bindgen::to_value(&vec!["Mars/Olympus"]).unwrap();
    let error = convert_timestamp("unix-seconds", "0", zones).unwrap_err();

    assert_eq!(string_field(&error, "code"), "INVALID_TIMEZONE");
    assert_eq!(string_field(&error, "message"), "Invalid IANA time zone");
}
