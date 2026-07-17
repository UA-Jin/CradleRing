// Parses booleans and case-insensitive `true`/`false` string tokens.
// 翻译自 packages/normalization-core/src/boolean-coercion.ts

use serde_json::Value;

/// Parses booleans and case-insensitive `true`/`false` string tokens.
pub fn parse_boolean(value: &Value) -> Option<bool> {
    if let Some(b) = value.as_bool() {
        return Some(b);
    }
    if let Some(s) = value.as_str() {
        let normalized = s.trim().to_lowercase();
        match normalized.as_str() {
            "true" => return Some(true),
            "false" => return Some(false),
            _ => {}
        }
    }
    None
}
