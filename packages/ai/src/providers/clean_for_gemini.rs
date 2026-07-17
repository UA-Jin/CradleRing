//! Gemini tool-schema cleaner.
//! 翻译自 packages/ai/src/providers/clean-for-gemini.ts
//!
//! Removes schema keywords that Cloud Code Assist rejects, normalizes
//! `enum` values to strings, and flattens literal `anyOf`/`oneOf` unions.

use std::collections::HashSet;

use once_cell::sync::Lazy;

/// Keywords that the Cloud Code Assist API rejects.
pub static GEMINI_UNSUPPORTED_SCHEMA_KEYWORDS: Lazy<HashSet<&'static str>> = Lazy::new(|| {
    HashSet::from([
        "patternProperties",
        "additionalProperties",
        "$schema",
        "$id",
        "$ref",
        "$defs",
        "definitions",
        "examples",
        "minLength",
        "maxLength",
        "minimum",
        "maximum",
        "multipleOf",
        "pattern",
        "format",
        "minItems",
        "maxItems",
        "uniqueItems",
        "minProperties",
        "maxProperties",
        "not",
    ])
});

const SCHEMA_META_KEYS: &[&str] = &["description", "title", "default"];

fn copy_schema_meta(from: &serde_json::Map<String, serde_json::Value>, to: &mut serde_json::Map<String, serde_json::Value>) {
    for key in SCHEMA_META_KEYS {
        if let Some(v) = from.get(*key) {
            if !v.is_null() {
                to.insert((*key).to_string(), v.clone());
            }
        }
    }
}

fn stringify_gemini_enum_value(value: &serde_json::Value) -> Option<String> {
    match value {
        serde_json::Value::String(s) => Some(s.clone()),
        serde_json::Value::Number(n) => Some(n.to_string()),
        serde_json::Value::Bool(b) => Some(b.to_string()),
        _ => None,
    }
}

fn clean_gemini_enum_values(value: &serde_json::Value) -> Option<Vec<String>> {
    let arr = value.as_array()?;
    let mut values: Vec<String> = Vec::new();
    for entry in arr {
        if let Some(s) = stringify_gemini_enum_value(entry) {
            values.push(s);
        }
    }
    values.sort();
    values.dedup();
    if values.is_empty() {
        None
    } else {
        Some(values)
    }
}

fn try_flatten_literal_any_of(variants: &[serde_json::Value]) -> Option<(String, Vec<serde_json::Value>)> {
    if variants.is_empty() {
        return None;
    }
    let mut common_type: Option<String> = None;
    let mut all_values: Vec<serde_json::Value> = Vec::new();
    for variant in variants {
        let obj = variant.as_object()?;
        let literal = if obj.contains_key("const") {
            obj.get("const")?.clone()
        } else if let Some(serde_json::Value::Array(arr)) = obj.get("enum") {
            if arr.len() != 1 {
                return None;
            }
            arr.first()?.clone()
        } else {
            return None;
        };
        let variant_type = obj.get("type")?.as_str()?.to_string();
        match &common_type {
            None => common_type = Some(variant_type),
            Some(t) if t == &variant_type => {}
            _ => return None,
        }
        all_values.push(literal);
    }
    Some((common_type?, all_values))
}

fn is_null_schema(variant: &serde_json::Value) -> bool {
    let Some(record) = variant.as_object() else {
        return false;
    };
    if let Some(v) = record.get("const") {
        if v.is_null() {
            return true;
        }
    }
    if let Some(serde_json::Value::Array(arr)) = record.get("enum") {
        if arr.len() == 1 && arr[0].is_null() {
            return true;
        }
    }
    match record.get("type") {
        Some(serde_json::Value::String(t)) if t == "null" => true,
        Some(serde_json::Value::Array(arr)) if arr.len() == 1 => {
            arr[0].as_str() == Some("null")
        }
        _ => false,
    }
}

fn strip_null_variants(variants: &[serde_json::Value]) -> (Vec<serde_json::Value>, bool) {
    if variants.is_empty() {
        return (variants.to_vec(), false);
    }
    let mut stripped = false;
    let non_null: Vec<serde_json::Value> = variants
        .iter()
        .filter(|v| {
            if is_null_schema(v) {
                stripped = true;
                false
            } else {
                true
            }
        })
        .cloned()
        .collect();
    (non_null, stripped)
}

/// Clean a tool schema for Gemini consumption.
pub fn clean_schema_for_gemini(schema: &serde_json::Value) -> serde_json::Value {
    if !schema.is_object() {
        return schema.clone();
    }
    let mut obj = schema.as_object().unwrap().clone();

    // Strip unsupported top-level keywords.
    for key in GEMINI_UNSUPPORTED_SCHEMA_KEYWORDS.iter() {
        obj.remove(*key);
    }

    // Convert `const` to enum.
    if let Some(const_value) = obj.remove("const") {
        if !obj.contains_key("enum") {
            obj.insert("enum".to_string(), serde_json::Value::Array(vec![const_value]));
        }
    }

    // Clean enum values.
    if let Some(enum_value) = obj.get("enum").cloned() {
        if let Some(cleaned) = clean_gemini_enum_values(&enum_value) {
            obj.insert(
                "enum".to_string(),
                serde_json::Value::Array(
                    cleaned.into_iter().map(serde_json::Value::String).collect(),
                ),
            );
        }
    }

    // Recurse into nested schemas.
    if let Some(serde_json::Value::Object(props)) = obj.get_mut("properties") {
        let keys: Vec<String> = props.keys().cloned().collect();
        for k in keys {
            if let Some(v) = props.get(&k) {
                props.insert(k, clean_schema_for_gemini(v));
            }
        }
    }
    if let Some(items) = obj.get("items").cloned() {
        obj.insert("items".to_string(), clean_schema_for_gemini(&items));
    }
    for key in ["oneOf", "anyOf", "allOf"] {
        if let Some(serde_json::Value::Array(arr)) = obj.get(key).cloned() {
            let new_arr = clean_variants(&arr);
            if let Some((common_type, all_values)) = try_flatten_literal_any_of(&new_arr) {
                let mut new_obj = serde_json::Map::new();
                copy_schema_meta(&obj, &mut new_obj);
                new_obj.insert(
                    "type".to_string(),
                    serde_json::Value::String(common_type),
                );
                new_obj.insert("enum".to_string(), serde_json::Value::Array(all_values));
                obj = new_obj;
            } else {
                obj.insert(key.to_string(), serde_json::Value::Array(new_arr));
            }
        }
    }
    serde_json::Value::Object(obj)
}

fn clean_variants(arr: &[serde_json::Value]) -> Vec<serde_json::Value> {
    let (stripped, _) = strip_null_variants(arr);
    stripped
        .into_iter()
        .map(|v| clean_schema_for_gemini(&v))
        .collect()
}