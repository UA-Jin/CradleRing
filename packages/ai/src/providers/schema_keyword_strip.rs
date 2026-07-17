//! Recursively remove unsupported schema keywords.
//! 翻译自 packages/ai/src/providers/schema-keyword-strip.ts

use std::collections::HashSet;

/// Recursively remove schema keywords unsupported by a target provider/tool surface.
pub fn strip_unsupported_schema_keywords(
    schema: &serde_json::Value,
    unsupported_keywords: &HashSet<String>,
) -> serde_json::Value {
    if !schema.is_object() {
        return schema.clone();
    }
    let Some(obj) = schema.as_object() else {
        return schema.clone();
    };
    let mut cleaned = serde_json::Map::new();
    for (key, value) in obj {
        if unsupported_keywords.contains(key) {
            continue;
        }
        if key == "properties" && value.is_object() {
            let Some(props) = value.as_object() else {
                cleaned.insert(key.clone(), value.clone());
                continue;
            };
            let mut new_props = serde_json::Map::new();
            for (child_key, child_value) in props {
                new_props.insert(
                    child_key.clone(),
                    strip_unsupported_schema_keywords(child_value, unsupported_keywords),
                );
            }
            cleaned.insert(key.clone(), serde_json::Value::Object(new_props));
            continue;
        }
        if key == "items" {
            let new_value = if value.is_array() {
                let arr = value.as_array().unwrap();
                serde_json::Value::Array(
                    arr.iter()
                        .map(|v| strip_unsupported_schema_keywords(v, unsupported_keywords))
                        .collect(),
                )
            } else {
                strip_unsupported_schema_keywords(value, unsupported_keywords)
            };
            cleaned.insert(key.clone(), new_value);
            continue;
        }
        if (key == "anyOf" || key == "oneOf" || key == "allOf") && value.is_array() {
            let arr = value.as_array().unwrap();
            let new_value = serde_json::Value::Array(
                arr.iter()
                    .map(|v| strip_unsupported_schema_keywords(v, unsupported_keywords))
                    .collect(),
            );
            cleaned.insert(key.clone(), new_value);
            continue;
        }
        cleaned.insert(key.clone(), value.clone());
    }
    serde_json::Value::Object(cleaned)
}