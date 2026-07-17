// Shared terminal string normalization helpers.
// 翻译自 packages/terminal-core/src/string.ts

/// Normalize string input to lowercase, returning empty string for non-strings.
pub fn normalize_lowercase_string_or_empty(value: &str) -> String {
    value.trim().to_lowercase()
}
