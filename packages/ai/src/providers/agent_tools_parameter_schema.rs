//! Agent tool parameter schema normalization.
//! 翻译自 packages/ai/src/providers/agent-tools-parameter-schema.ts
//!
//! Normalizes model-facing tool parameter schemas across provider quirks:
//! handles local JSON-Schema refs, OpenAPI nullable syntax, top-level
//! unions, and provider-specific unsupported-keyword stripping.

use std::collections::BTreeSet;

use serde_json::Value;

use crate::providers::clean_for_gemini::clean_schema_for_gemini;
use crate::providers::schema_keyword_strip::strip_unsupported_schema_keywords;

/// Narrow structural view of the model compat config.
#[derive(Debug, Clone, Default, serde::Deserialize, serde::Serialize)]
pub struct ToolSchemaModelCompat {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_schema_profile: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unsupported_tool_schema_keywords: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub omit_empty_array_items: Option<bool>,
}

impl ToolSchemaModelCompat {
    /// Construct from a generic JSON value.
    pub fn from_value(value: Value) -> Option<Self> {
        serde_json::from_value(value).ok()
    }
}

/// Extract the compat record whether callers pass a model or the compat itself.
pub fn extract_tool_schema_model_compat(
    model_or_compat: Option<&Value>,
) -> Option<ToolSchemaModelCompat> {
    let obj = model_or_compat?.as_object()?;
    if obj.contains_key("compat") {
        let compat = obj.get("compat")?;
        return ToolSchemaModelCompat::from_value(compat.clone());
    }
    ToolSchemaModelCompat::from_value(model_or_compat.unwrap().clone())
}

/// JSON Schema keywords this model/provider rejects in tool schemas.
pub fn resolve_unsupported_tool_schema_keywords(
    model_or_compat: Option<&Value>,
) -> BTreeSet<String> {
    let mut out: BTreeSet<String> = BTreeSet::new();
    if let Some(compat) = extract_tool_schema_model_compat(model_or_compat) {
        for k in compat.unsupported_tool_schema_keywords.unwrap_or_default() {
            out.insert(k);
        }
    }
    out
}

/// Whether empty `items: {}` on array schemas must be omitted for this model/provider.
pub fn should_omit_empty_array_items(model_or_compat: Option<&Value>) -> bool {
    extract_tool_schema_model_compat(model_or_compat)
        .and_then(|c| c.omit_empty_array_items)
        .unwrap_or(false)
}

/// Options accepted by `normalize_tool_parameter_schema`.
#[derive(Debug, Clone, Default)]
pub struct ToolParameterSchemaOptions {
    pub model_provider: Option<String>,
    pub model_id: Option<String>,
    pub model_compat: Option<ToolSchemaModelCompat>,
}

/// Normalize a tool parameter schema for a given model/provider.
pub fn normalize_tool_parameter_schema(
    schema: &Value,
    model_compat: Option<&ToolSchemaModelCompat>,
) -> Value {
    let value = if schema.is_null() {
        Value::Object(Default::default())
    } else {
        schema.clone()
    };

    let unsupported = model_compat
        .and_then(|c| c.unsupported_tool_schema_keywords.clone())
        .unwrap_or_default();
    let unsupported_set: std::collections::HashSet<String> = unsupported.into_iter().collect();

    let mut cleaned = strip_unsupported_schema_keywords(&value, &unsupported_set);

    let omit_empty_array_items = model_compat
        .and_then(|c| c.omit_empty_array_items)
        .unwrap_or(false);
    if omit_empty_array_items {
        cleaned = omit_empty_array_items_in_schema(cleaned);
    }

    let profile = model_compat
        .and_then(|c| c.tool_schema_profile.clone())
        .unwrap_or_default();
    if profile == "gemini" {
        cleaned = clean_schema_for_gemini(&cleaned);
    }
    cleaned
}

fn omit_empty_array_items_in_schema(schema: Value) -> Value {
    match schema {
        Value::Object(mut obj) => {
            if let Some(Value::Object(items)) = obj.get("items") {
                if items.is_empty() {
                    obj.remove("items");
                }
            }
            if let Some(Value::Object(props)) = obj.get_mut("properties") {
                let keys: Vec<String> = props.keys().cloned().collect();
                for k in keys {
                    if let Some(v) = props.get(&k) {
                        let new_v = omit_empty_array_items_in_schema(v.clone());
                        props.insert(k, new_v);
                    }
                }
            }
            Value::Object(obj)
        }
        Value::Array(arr) => Value::Array(
            arr.into_iter()
                .map(omit_empty_array_items_in_schema)
                .collect(),
        ),
        other => other,
    }
}