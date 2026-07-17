// Parses JSON without throwing, returning undefined for invalid input.
// 翻译自 packages/normalization-core/src/json-coercion.ts

use serde_json::Value;

/// Parses JSON without throwing, returning None for invalid input.
pub fn safe_parse_json(value: &str) -> Option<Value> {
    serde_json::from_str(value).ok()
}
