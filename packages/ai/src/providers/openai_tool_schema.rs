//! OpenAI strict JSON-schema normalization.
//! 翻译自 packages/ai/src/providers/openai-tool-schema.ts
//!
//! Top-level entry points only — the underlying recursive normalization
//! is delegated to `normalize_strict_openai_json_schema_recursive`. The
//! helper cache mirrors the WeakMap-based cache used in the TS source.

use std::sync::RwLock;

use crate::providers::agent_tools_parameter_schema::{
    normalize_tool_parameter_schema, should_omit_empty_array_items, ToolSchemaModelCompat,
};

const MAX_STRICT_SCHEMA_CACHE_ENTRIES_PER_SCHEMA: usize = 8;

#[derive(Default)]
struct StrictSchemaCache {
    inner: RwLock<std::collections::HashMap<String, serde_json::Value>>,
}

static STRICT_OPENAI_SCHEMA_CACHE: once_cell::sync::Lazy<StrictSchemaCache> =
    once_cell::sync::Lazy::new(StrictSchemaCache::default);

fn resolve_tool_schema_model_compat(
    compat: Option<&ToolSchemaCompatInput>,
) -> Option<ToolSchemaModelCompat> {
    let compat = compat?;
    let unsupported: Vec<String> = compat
        .unsupported_tool_schema_keywords
        .clone()
        .unwrap_or_default();
    if unsupported.is_empty() && compat.omit_empty_array_items != Some(true) {
        return None;
    }
    let mut map: std::collections::BTreeMap<String, serde_json::Value> = std::collections::BTreeMap::new();
    if !unsupported.is_empty() {
        map.insert(
            "unsupportedToolSchemaKeywords".to_string(),
            serde_json::to_value(&unsupported).unwrap_or_default(),
        );
    }
    if compat.omit_empty_array_items == Some(true) {
        map.insert(
            "omitEmptyArrayItems".to_string(),
            serde_json::Value::Bool(true),
        );
    }
    Some(ToolSchemaModelCompat::from_value(serde_json::Value::Object(
        map.into_iter().collect(),
    ))?)
}

fn resolve_strict_openai_schema_cache_key(compat: Option<&ToolSchemaCompatInput>) -> String {
    let compat = resolve_tool_schema_model_compat(compat);
    let mut unsupported: Vec<String> = compat
        .as_ref()
        .and_then(|c| c.unsupported_tool_schema_keywords.clone())
        .unwrap_or_default();
    unsupported.sort();
    let compat_value = compat.as_ref().map(|c| serde_json::to_value(c).unwrap_or_default());
    let omit = should_omit_empty_array_items(compat_value.as_ref());
    serde_json::json!([unsupported, omit]).to_string()
}

fn read_cached_strict_openai_schema(key: &str) -> Option<serde_json::Value> {
    STRICT_OPENAI_SCHEMA_CACHE
        .inner
        .read()
        .expect("strict schema cache poisoned")
        .get(key)
        .cloned()
}

fn remember_strict_openai_schema(key: &str, value: serde_json::Value) -> serde_json::Value {
    let mut guard = STRICT_OPENAI_SCHEMA_CACHE
        .inner
        .write()
        .expect("strict schema cache poisoned");
    guard.insert(key.to_string(), value.clone());
    while guard.len() > MAX_STRICT_SCHEMA_CACHE_ENTRIES_PER_SCHEMA {
        if let Some(first_key) = guard.keys().next().cloned() {
            guard.remove(&first_key);
        } else {
            break;
        }
    }
    value
}

/// Tool-schema compatibility input from the model catalog.
#[derive(Debug, Clone, Default, serde::Deserialize)]
pub struct ToolSchemaCompatInput {
    #[serde(default)]
    pub unsupported_tool_schema_keywords: Option<Vec<String>>,
    #[serde(default)]
    pub omit_empty_array_items: Option<bool>,
}

/// Clear the strict-schema cache (testing only).
pub fn clear_openai_tool_schema_cache_for_test() {
    STRICT_OPENAI_SCHEMA_CACHE
        .inner
        .write()
        .expect("strict schema cache poisoned")
        .clear();
}

/// Normalizes a tool parameter schema into the OpenAI strict JSON-schema subset.
pub fn normalize_strict_openai_json_schema(
    schema: Option<&serde_json::Value>,
    model_compat: Option<&ToolSchemaCompatInput>,
) -> serde_json::Value {
    let schema_input = schema.cloned().unwrap_or_else(|| serde_json::json!({}));
    if !schema_input.is_object() {
        let normalized = normalize_tool_parameter_schema(
            &schema_input,
            resolve_tool_schema_model_compat(model_compat).as_ref(),
        );
        return normalize_strict_openai_json_schema_recursive(&normalized, 0);
    }
    let cache_key = resolve_strict_openai_schema_cache_key(model_compat);
    if let Some(cached) = read_cached_strict_openai_schema(&cache_key) {
        return cached;
    }
    let normalized = normalize_tool_parameter_schema(
        &schema_input,
        resolve_tool_schema_model_compat(model_compat).as_ref(),
    );
    let result = normalize_strict_openai_json_schema_recursive(&normalized, 0);
    remember_strict_openai_schema(&cache_key, result.clone());
    result
}

/// Recursively normalizes an arbitrary JSON-schema-like value for OpenAI strict mode.
///
/// Mirrors the deep recursion in the TS helper: object schemas receive
/// `additionalProperties: false` and all properties are marked required;
/// `enum` is preserved; arrays recurse into `items`; oneOf/anyOf/allOf
/// recurse into each branch.
pub fn normalize_strict_openai_json_schema_recursive(
    value: &serde_json::Value,
    depth: usize,
) -> serde_json::Value {
    const MAX_DEPTH: usize = 64;
    if depth > MAX_DEPTH {
        return value.clone();
    }
    match value {
        serde_json::Value::Object(_) => {
            let mut obj = value.as_object().unwrap().clone();
            if matches!(obj.get("type"), Some(serde_json::Value::String(t)) if t == "object") {
                if let Some(serde_json::Value::Object(props)) = obj.get("properties") {
                    let required: Vec<serde_json::Value> = props
                        .keys()
                        .map(|k| serde_json::Value::String(k.clone()))
                        .collect();
                    obj.insert("required".to_string(), serde_json::Value::Array(required));
                }
                obj.entry("additionalProperties".to_string())
                    .or_insert(serde_json::Value::Bool(false));
            }
            // recurse into properties
            if let Some(serde_json::Value::Object(props)) = obj.get_mut("properties") {
                let keys: Vec<String> = props.keys().cloned().collect();
                for k in keys {
                    if let Some(v) = props.get(&k) {
                        let new_v = normalize_strict_openai_json_schema_recursive(v, depth + 1);
                        props.insert(k, new_v);
                    }
                }
            }
            // recurse into items
            if let Some(items) = obj.get("items").cloned() {
                obj.insert(
                    "items".to_string(),
                    normalize_strict_openai_json_schema_recursive(&items, depth + 1),
                );
            }
            // recurse into combinators
            for key in ["oneOf", "anyOf", "allOf"] {
                if let Some(serde_json::Value::Array(arr)) = obj.get(key).cloned() {
                    let new_arr: Vec<serde_json::Value> = arr
                        .iter()
                        .map(|v| normalize_strict_openai_json_schema_recursive(v, depth + 1))
                        .collect();
                    obj.insert(key.to_string(), serde_json::Value::Array(new_arr));
                }
            }
            serde_json::Value::Object(obj)
        }
        serde_json::Value::Array(arr) => serde_json::Value::Array(
            arr.iter()
                .map(|v| normalize_strict_openai_json_schema_recursive(v, depth + 1))
                .collect(),
        ),
        _ => value.clone(),
    }
}