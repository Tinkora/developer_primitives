#![cfg(target_arch = "wasm32")]

use js_sys::{Array, Reflect};
use uuid_factory_web::{batch_generate, generate, inspect_identifier};
use wasm_bindgen::JsValue;
use wasm_bindgen_test::wasm_bindgen_test;

fn string_field(value: &JsValue, name: &str) -> String {
    Reflect::get(value, &JsValue::from_str(name))
        .unwrap()
        .as_string()
        .unwrap()
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
