//! Header utilities.
//! 翻译自 packages/ai/src/utils/headers.ts
//!
//! Minimal Rust re-export surface. Most "header" helper logic in the TS
//! package lives in providers (anthropic-auth-headers, github-copilot-headers,
//! azure deployment map, etc.). We keep a small helper here for lowercased
//! keys to keep parity with `Object.fromEntries(headers.map(...))` style
//! helpers in the JS source.

use std::collections::BTreeMap;

/// Returns a copy of `headers` with all keys lowercased (case-insensitive lookup map).
pub fn lowercase_header_keys(headers: &BTreeMap<String, String>) -> BTreeMap<String, String> {
    headers
        .iter()
        .map(|(k, v)| (k.to_lowercase(), v.clone()))
        .collect()
}

/// Looks up a header value in a case-insensitive way.
pub fn get_header<'a>(headers: &'a BTreeMap<String, String>, name: &str) -> Option<&'a String> {
    headers.get(&name.to_lowercase())
}