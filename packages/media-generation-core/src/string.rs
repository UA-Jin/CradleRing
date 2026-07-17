// Shared string normalization helpers for media-generation packages.
// 翻译自 packages/media-generation-core/src/string.ts

/// Normalize optional strings, returning None for non-strings or empty values.
pub fn normalize_optional_string(value: &str) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

/// Return unique trimmed strings while preserving first-seen order.
pub fn unique_trimmed_strings(values: &[Option<String>]) -> Vec<String> {
    let mut seen = std::collections::HashSet::new();
    let mut result = Vec::new();
    for value in values {
        if let Some(normalized) = value.as_ref().and_then(|v| normalize_optional_string(v)) {
            if seen.insert(normalized.clone()) {
                result.push(normalized);
            }
        }
    }
    result
}