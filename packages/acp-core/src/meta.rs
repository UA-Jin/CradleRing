// ACP Core module implements meta behavior.
// 翻译自 packages/acp-core/src/meta.ts

use normalization_core::string_coerce;
use serde_json::{Map, Value};

fn read_meta_value<T>(
    meta: Option<&Map<String, Value>>,
    keys: &[&str],
    normalize: impl Fn(&Value) -> Option<T>,
) -> Option<T> {
    let meta = meta?;
    for key in keys {
        if let Some(v) = meta.get(*key) {
            if let Some(n) = normalize(v) {
                return Some(n);
            }
        }
    }
    None
}

/// Reads the first present string metadata value from a current-to-legacy key list.
pub fn read_string(meta: Option<&Map<String, Value>>, keys: &[&str]) -> Option<String> {
    read_meta_value(meta, keys, |v| string_coerce::normalize_optional_string(v))
}

/// Reads the first boolean metadata value without dropping false.
pub fn read_bool(meta: Option<&Map<String, Value>>, keys: &[&str]) -> Option<bool> {
    read_meta_value(meta, keys, |v| v.as_bool())
}

/// Reads the first finite numeric metadata value from a current-to-legacy key list.
pub fn read_number(meta: Option<&Map<String, Value>>, keys: &[&str]) -> Option<f64> {
    read_meta_value(meta, keys, |v| match v {
        Value::Number(n) => n.as_f64().filter(|f| f.is_finite()),
        _ => None,
    })
}

/// Reads the first safe non-negative integer metadata value, preserving zero.
pub fn read_non_negative_integer(meta: Option<&Map<String, Value>>, keys: &[&str]) -> Option<i64> {
    read_meta_value(meta, keys, |v| match v {
        Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                if i >= 0 {
                    Some(i)
                } else {
                    None
                }
            } else {
                None
            }
        }
        _ => None,
    })
}