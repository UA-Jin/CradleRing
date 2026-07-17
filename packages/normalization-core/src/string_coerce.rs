// String coercion helpers.
// 翻译自 packages/normalization-core/src/string-coerce.ts

use serde_json::Value;

/// Reads a value only when it is already a string, preserving whitespace.
pub fn read_string_value(value: &Value) -> Option<String> {
    value.as_str().map(|s| s.to_string())
}

/// Trims string input and returns None for non-strings or empty strings.
/// (TS 返回 null，Rust 用 None 对应)
pub fn normalize_nullable_string(value: &Value) -> Option<String> {
    let s = value.as_str()?;
    let trimmed = s.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

/// Trims string input and returns None for non-strings or empty strings.
pub fn normalize_optional_string(value: &Value) -> Option<String> {
    normalize_nullable_string(value)
}

/// Stringifies primitive ids/flags before applying optional string normalization.
pub fn normalize_stringified_optional_string(value: &Value) -> Option<String> {
    if let Some(s) = value.as_str() {
        return normalize_optional_string(&Value::String(s.to_string()));
    }
    if let Some(n) = value.as_f64() {
        if value.is_i64() {
            return normalize_optional_string(&Value::String(value.as_i64().unwrap().to_string()));
        }
        return normalize_optional_string(&Value::String(n.to_string()));
    }
    if let Some(b) = value.as_bool() {
        return normalize_optional_string(&Value::String(b.to_string()));
    }
    None
}

/// Normalizes an optional array of primitive-ish values into non-empty strings.
pub fn normalize_stringified_entries(values: Option<&[Value]>) -> Vec<String> {
    let values = values.unwrap_or(&[]);
    values
        .iter()
        .filter_map(|entry| normalize_stringified_optional_string(entry))
        .collect()
}

/// Lowercases a normalized optional string.
pub fn normalize_optional_lowercase_string(value: &Value) -> Option<String> {
    normalize_optional_string(value).map(|s| s.to_lowercase())
}

/// Lowercases a normalized string or returns an empty string when absent.
pub fn normalize_lowercase_string_or_empty(value: &Value) -> String {
    normalize_optional_lowercase_string(value).unwrap_or_default()
}

/// FastMode = bool | "auto"
#[derive(Debug, Clone, PartialEq)]
pub enum FastMode {
    Bool(bool),
    Auto,
}

/// Parses loose boolean/fast-mode flags from strings or booleans.
pub fn normalize_fast_mode(raw: &Value) -> Option<FastMode> {
    if let Some(b) = raw.as_bool() {
        return Some(FastMode::Bool(b));
    }
    if raw.is_null() {
        return None;
    }
    let key = normalize_lowercase_string_or_empty(raw);
    if key.is_empty() {
        return None;
    }
    if ["off", "false", "no", "0", "disable", "disabled", "normal"].contains(&key.as_str()) {
        return Some(FastMode::Bool(false));
    }
    if ["on", "true", "yes", "1", "enable", "enabled", "fast"].contains(&key.as_str()) {
        return Some(FastMode::Bool(true));
    }
    if ["auto", "automatic"].contains(&key.as_str()) {
        return Some(FastMode::Auto);
    }
    None
}

/// Lowercases text while intentionally preserving surrounding whitespace.
pub fn lowercase_preserving_whitespace(value: &str) -> String {
    value.to_lowercase()
}

/// Locale-aware lowercase helper that still preserves surrounding whitespace.
pub fn locale_lowercase_preserving_whitespace(value: &str) -> String {
    value.to_lowercase()
}

/// Reads a string directly or from an object's `primary` field.
pub fn resolve_primary_string_value(value: &Value) -> Option<String> {
    if let Some(s) = value.as_str() {
        return normalize_optional_string(&Value::String(s.to_string()));
    }
    if !value.is_object() {
        return None;
    }
    let primary = value.get("primary")?;
    normalize_optional_string(primary)
}

/// Normalizes thread ids that may be numeric or string-backed.
pub fn normalize_optional_thread_value(value: &Value) -> Option<thread_value::ThreadValue> {
    if let Some(n) = value.as_f64() {
        if n.is_finite() {
            return Some(thread_value::ThreadValue::Number(n.trunc() as i64));
        }
        return None;
    }
    normalize_optional_string(value).map(thread_value::ThreadValue::String)
}

/// Normalizes a thread/id value and stringifies finite numeric ids.
pub fn normalize_optional_stringified_id(value: &Value) -> Option<String> {
    match normalize_optional_thread_value(value)? {
        thread_value::ThreadValue::Number(n) => Some(n.to_string()),
        thread_value::ThreadValue::String(s) => Some(s),
    }
}

/// Type guard for strings that remain non-empty after trimming.
pub fn has_non_empty_string(value: &Value) -> bool {
    normalize_optional_string(value).is_some()
}

pub mod thread_value {
    #[derive(Debug, Clone)]
    pub enum ThreadValue {
        Number(i64),
        String(String),
    }
}
