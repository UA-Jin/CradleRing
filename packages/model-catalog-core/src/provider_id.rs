// Provider id normalization helpers.
// 翻译自 packages/model-catalog-core/src/provider-id.ts

/// Normalize a value to a lowercase trimmed string, or empty string when not a string.
pub fn normalize_lowercase_string_or_empty(value: impl Into<String>) -> String {
    let s: String = value.into();
    s.trim().to_lowercase()
}

pub fn normalize_provider_id(provider: impl Into<String>) -> String {
    normalize_lowercase_string_or_empty(provider)
}

/// Normalize provider ID before manifest-owned auth alias lookup.
pub fn normalize_provider_id_for_auth(provider: impl Into<String>) -> String {
    normalize_provider_id(provider)
}

/// Find a value in a provider-keyed record using normalized provider ids.
pub fn find_normalized_provider_value<'a, T>(
    entries: Option<&'a std::collections::BTreeMap<String, T>>,
    provider: &str,
) -> Option<&'a T> {
    let entries = entries?;
    let provider_key = normalize_provider_id(provider);
    for (key, value) in entries {
        if normalize_provider_id(key.as_str()) == provider_key {
            return Some(value);
        }
    }
    None
}

/// Find the original key whose normalized form matches the supplied provider id.
pub fn find_normalized_provider_key(
    entries: Option<&std::collections::BTreeMap<String, serde_json::Value>>,
    provider: &str,
) -> Option<String> {
    let entries = entries?;
    let provider_key = normalize_provider_id(provider);
    for key in entries.keys() {
        if normalize_provider_id(key.as_str()) == provider_key {
            return Some(key.clone());
        }
    }
    None
}