// String normalization helpers.
// 翻译自 packages/normalization-core/src/string-normalization.ts

use crate::string_coerce::{
    normalize_optional_lowercase_string, normalize_optional_string,
};
use serde_json::Value;
use std::collections::HashSet;

/// Coerces entries to strings, trims them, and drops empty results.
pub fn normalize_string_entries(list: Option<&[Value]>) -> Vec<String> {
    let list = list.unwrap_or(&[]);
    list.iter()
        .map(|entry| {
            let s = entry.to_string();
            normalize_optional_string(&Value::String(s)).unwrap_or_default()
        })
        .filter(|s| !s.is_empty())
        .collect()
}

/// Normalizes string entries and lowercases each retained value.
pub fn normalize_string_entries_lower(list: Option<&[Value]>) -> Vec<String> {
    normalize_string_entries(list)
        .iter()
        .map(|entry| normalize_optional_lowercase_string(&Value::String(entry.clone())).unwrap_or_default())
        .collect()
}

/// Returns first-seen unique values while preserving insertion order.
pub fn unique_values<T: std::hash::Hash + Eq + Clone>(values: impl IntoIterator<Item = T>) -> Vec<T> {
    let mut seen = HashSet::new();
    let mut result = Vec::new();
    for v in values {
        if seen.insert(v.clone()) {
            result.push(v);
        }
    }
    result
}

/// Returns first-seen unique strings while preserving insertion order.
pub fn unique_strings(values: impl IntoIterator<Item = String>) -> Vec<String> {
    unique_values(values)
}

/// Returns unique strings sorted with stable ASCII comparison.
pub fn sort_unique_strings(values: impl IntoIterator<Item = String>) -> Vec<String> {
    let mut v = unique_strings(values);
    v.sort();
    v
}

/// Normalizes entries, removes duplicates, and preserves first-seen order.
pub fn normalize_unique_string_entries(values: Option<&[Value]>) -> Vec<String> {
    unique_strings(normalize_string_entries(values))
}

/// Lowercases normalized entries, removes empties/duplicates, and preserves first-seen order.
pub fn normalize_unique_string_entries_lower(values: Option<&[Value]>) -> Vec<String> {
    let lowered = normalize_string_entries_lower(values);
    unique_strings(lowered.into_iter().filter(|s| !s.is_empty()))
}

/// Normalizes entries, removes duplicates, and returns sorted output.
pub fn normalize_sorted_unique_string_entries(values: Option<&[Value]>) -> Vec<String> {
    sort_unique_strings(normalize_unique_string_entries(values))
}

/// Normalizes array-backed string lists and rejects non-array input as empty.
pub fn normalize_trimmed_string_list(value: &Value) -> Vec<String> {
    let arr = match value.as_array() {
        Some(a) => a,
        None => return vec![],
    };
    arr.iter()
        .filter_map(|entry| normalize_optional_string(entry))
        .collect()
}

/// Normalizes an array-backed string list and removes duplicates.
pub fn normalize_unique_trimmed_string_list(value: &Value) -> Vec<String> {
    unique_strings(normalize_trimmed_string_list(value))
}

/// Normalizes an array-backed string list, removes duplicates, and sorts it.
pub fn normalize_sorted_unique_trimmed_string_list(value: &Value) -> Vec<String> {
    sort_unique_strings(normalize_trimmed_string_list(value))
}

/// Returns None instead of an empty normalized array-backed string list.
pub fn normalize_optional_trimmed_string_list(value: &Value) -> Option<Vec<String>> {
    let normalized = normalize_trimmed_string_list(value);
    if normalized.is_empty() {
        None
    } else {
        Some(normalized)
    }
}

/// Returns None for non-arrays but preserves an empty array for explicit arrays.
pub fn normalize_array_backed_trimmed_string_list(value: &Value) -> Option<Vec<String>> {
    if !value.is_array() {
        return None;
    }
    Some(normalize_trimmed_string_list(value))
}

/// Normalizes either a single string-like value or an array-backed string list.
pub fn normalize_single_or_trimmed_string_list(value: &Value) -> Vec<String> {
    if value.is_array() {
        return normalize_trimmed_string_list(value);
    }
    match normalize_optional_string(value) {
        Some(s) => vec![s],
        None => vec![],
    }
}

/// Normalizes single-or-array string input and removes duplicates.
pub fn normalize_unique_single_or_trimmed_string_list(value: &Value) -> Vec<String> {
    unique_strings(normalize_single_or_trimmed_string_list(value))
}

/// Parses either array entries or comma-separated string entries into trimmed values.
pub fn normalize_csv_or_loose_string_list(value: &Value) -> Vec<String> {
    if let Some(arr) = value.as_array() {
        return normalize_string_entries(Some(arr));
    }
    if let Some(s) = value.as_str() {
        return s
            .split(',')
            .map(|entry| entry.trim().to_string())
            .filter(|entry| !entry.is_empty())
            .collect();
    }
    vec![]
}

fn normalize_slug_input(raw: &Value) -> String {
    let lower = normalize_optional_lowercase_string(raw).unwrap_or_default();
    // NFC normalization: Rust 的 unicode-normalization crate 可做，但为保持依赖最小化
    // Rust String 默认是 NFC 兼容的（大多数情况）。如需精确 NFC，后续加 unicode-normalization。
    lower
}

/// Normalizes user-facing names into permissive lowercase slugs that may keep #/@/._+.
pub fn normalize_hyphen_slug(raw: &Value) -> String {
    let trimmed = normalize_slug_input(raw);
    if trimmed.is_empty() {
        return String::new();
    }
    let dashed = regex::Regex::new(r"\s+").unwrap().replace_all(&trimmed, "-").to_string();
    // [^\p{L}\p{M}\p{N}#@._+-]+ → 匹配非字母/数字/标记/#@._+- 的字符
    let cleaned = regex::Regex::new(r"[^\p{L}\p{M}\p{N}#@._+\-]+")
        .unwrap()
        .replace_all(&dashed, "-")
        .to_string();
    let cleaned = regex::Regex::new(r"-{2,}").unwrap().replace_all(&cleaned, "-").to_string();
    regex::Regex::new(r"^[-.]+|[-.]+$").unwrap().replace_all(&cleaned, "").to_string()
}

/// Normalizes @/#-prefixed channel names into strict lowercase hyphen slugs without the prefix.
pub fn normalize_at_hash_slug(raw: &Value) -> String {
    let trimmed = normalize_slug_input(raw);
    if trimmed.is_empty() {
        return String::new();
    }
    let without_prefix = regex::Regex::new(r"^[@#]+").unwrap().replace_all(&trimmed, "").to_string();
    let dashed = regex::Regex::new(r"[\s_]+").unwrap().replace_all(&without_prefix, "-").to_string();
    let cleaned = regex::Regex::new(r"[^\p{L}\p{M}\p{N}\-]+")
        .unwrap()
        .replace_all(&dashed, "-")
        .to_string();
    let cleaned = regex::Regex::new(r"-{2,}").unwrap().replace_all(&cleaned, "-").to_string();
    regex::Regex::new(r"^-+|-+$").unwrap().replace_all(&cleaned, "").to_string()
}
