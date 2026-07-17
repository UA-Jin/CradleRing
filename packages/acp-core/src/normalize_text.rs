// ACP text normalization facade shared with older imports.
// 翻译自 packages/acp-core/src/normalize-text.ts

use normalization_core::string_coerce;
use serde_json::Value;

/// Trims string input and returns None for non-strings or empty strings.
pub fn normalize_text(value: &Value) -> Option<String> {
    string_coerce::normalize_optional_string(value)
}

/// Convenience helper that accepts an Option<&str>.
pub fn normalize_text_opt(value: Option<&str>) -> Option<String> {
    match value {
        Some(s) => normalize_text(&Value::String(s.to_string())),
        None => normalize_text(&Value::Null),
    }
}