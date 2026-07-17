// Type guard for non-array object records at browser-safe boundaries.
// 翻译自 packages/normalization-core/src/record-coerce.ts

use serde_json::{Map, Value};

/// Type guard for non-array object records at browser-safe boundaries.
pub fn is_record(value: &Value) -> bool {
    value.is_object() && !value.is_array()
}

/// Coerces object-like values to records, falling back to an empty record.
pub fn as_record<'a>(value: &'a Value) -> &'a Map<String, Value> {
    static EMPTY: std::sync::OnceLock<Map<String, Value>> = std::sync::OnceLock::new();
    value.as_object().unwrap_or_else(|| EMPTY.get_or_init(|| Map::new()))
}

/// Coerces object-like values to owned records, falling back to an empty record.
pub fn as_record_owned(value: &Value) -> Map<String, Value> {
    value.as_object().cloned().unwrap_or_default()
}

/// Reads a field only when it exists as a string.
pub fn read_string_field(record: Option<&Map<String, Value>>, key: &str) -> Option<String> {
    let record = record?;
    let value = record.get(key)?;
    value.as_str().map(|s| s.to_string())
}

/// Returns a non-array record or None.
pub fn as_optional_record(value: &Value) -> Option<&Map<String, Value>> {
    if is_record(value) {
        value.as_object()
    } else {
        None
    }
}

/// Returns any object-backed record, including arrays, or None.
pub fn as_optional_object_record(value: &Value) -> Option<&Map<String, Value>> {
    if value.is_object() {
        value.as_object()
    } else {
        None
    }
}
