//! Anthropic tool projection.
//! 翻译自 packages/ai/src/providers/anthropic-tool-projection.ts
//!
//! Snapshots direct/custom tool descriptors before Anthropic payload
//! construction and applies Anthropic-specific JSON-schema normalization.

use std::collections::{BTreeSet, HashSet};

use once_cell::sync::Lazy;

use normalization_core::is_record;

use crate::providers::tool_schema_json_projection::project_runtime_tool_input_schema;

/// One projected Anthropic tool.
#[derive(Debug, Clone)]
pub struct AnthropicProjectedTool {
    pub original_name: String,
    pub wire_name: String,
    pub description: Option<String>,
    pub input_schema: AnthropicInputSchema,
}

/// Wrapped JSON Schema input shape.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct AnthropicInputSchema {
    pub properties: serde_json::Map<String, serde_json::Value>,
    pub required: Vec<String>,
}

/// Aggregate projection result.
#[derive(Debug, Clone, Default)]
pub struct AnthropicToolProjection {
    pub input_tool_count: usize,
    pub unavailable_original_names: BTreeSet<String>,
    pub tools: Vec<AnthropicProjectedTool>,
}

/// Parallel-tool-disable choice shape.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct AnthropicParallelToolChoice {
    #[serde(skip_serializing_if = "Option::is_none", rename = "disable_parallel_tool_use")]
    pub disable_parallel_tool_use: Option<bool>,
}

/// Anthropic projected tool choice.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AnthropicProjectedToolChoice {
    Auto(AnthropicParallelToolChoice),
    Any(AnthropicParallelToolChoice),
    None,
    Tool {
        name: String,
        #[serde(flatten)]
        parallel: AnthropicParallelToolChoice,
    },
}

static SCHEMA_VALUE_KEYWORDS: Lazy<HashSet<&'static str>> = Lazy::new(|| {
    HashSet::from([
        "additionalProperties",
        "contains",
        "contentSchema",
        "else",
        "if",
        "items",
        "not",
        "propertyNames",
        "then",
        "unevaluatedItems",
        "unevaluatedProperties",
    ])
});

static SCHEMA_ARRAY_KEYWORDS: Lazy<HashSet<&'static str>> = Lazy::new(|| {
    HashSet::from(["allOf", "anyOf", "oneOf", "prefixItems"])
});

static SCHEMA_MAP_KEYWORDS: Lazy<HashSet<&'static str>> = Lazy::new(|| {
    HashSet::from([
        "$defs",
        "definitions",
        "dependencies",
        "dependentSchemas",
        "patternProperties",
        "properties",
    ])
});

/// Normalize a JSON Schema value into the Anthropic-accepted subset.
pub fn normalize_anthropic_json_schema(schema: &serde_json::Value) -> serde_json::Value {
    if !is_record(schema) {
        return schema.clone();
    }
    let mut changed = false;
    let mut normalized: serde_json::Map<String, serde_json::Value> = schema.as_object().unwrap().clone();
    for (key, value) in schema.as_object().unwrap() {
        if SCHEMA_VALUE_KEYWORDS.contains(key.as_str()) && !value.is_array() {
            let next = normalize_anthropic_json_schema(value);
            if &next != value {
                changed = true;
            }
            normalized.insert(key.clone(), next);
            continue;
        }
        if SCHEMA_ARRAY_KEYWORDS.contains(key.as_str()) && value.is_array() {
            let arr = value.as_array().unwrap();
            let next: Vec<serde_json::Value> = arr.iter().map(normalize_anthropic_json_schema).collect();
            if next.iter().zip(arr.iter()).any(|(a, b)| a != b) {
                changed = true;
            }
            normalized.insert(key.clone(), serde_json::Value::Array(next));
            continue;
        }
        if SCHEMA_MAP_KEYWORDS.contains(key.as_str()) && is_record(value) {
            let mut next = serde_json::Map::new();
            for (entry_key, entry_value) in value.as_object().unwrap() {
                next.insert(entry_key.clone(), normalize_anthropic_json_schema(entry_value));
            }
            if value
                .as_object()
                .unwrap()
                .iter()
                .any(|(k, v)| next.get(k) != Some(v))
            {
                changed = true;
            }
            normalized.insert(key.clone(), serde_json::Value::Object(next));
            continue;
        }
    }

    // Handle tuple arrays: schema.items: [...] -> prefixItems + items
    if let Some(serde_json::Value::Array(items)) = schema.get("items") {
        normalized.insert(
            "prefixItems".to_string(),
            serde_json::Value::Array(items.iter().map(normalize_anthropic_json_schema).collect()),
        );
        match schema.get("additionalItems") {
            Some(serde_json::Value::Bool(b)) => {
                normalized.insert(
                    "items".to_string(),
                    serde_json::Value::Bool(*b),
                );
            }
            Some(v) if is_record(v) => {
                normalized.insert("items".to_string(), normalize_anthropic_json_schema(v));
            }
            _ => {
                normalized.remove("items");
            }
        }
        normalized.remove("additionalItems");
        changed = true;
    }

    if changed {
        serde_json::Value::Object(normalized)
    } else {
        schema.clone()
    }
}

/// Project Anthropic tool descriptors into a wire-ready form.
pub fn project_anthropic_tools<F>(
    tools: &[AnthropicToolDescriptor],
    to_wire_name: F,
) -> AnthropicToolProjection
where
    F: Fn(&str) -> String,
{
    let mut projection = AnthropicToolProjection {
        input_tool_count: tools.len(),
        unavailable_original_names: BTreeSet::new(),
        tools: vec![],
    };
    for tool in tools {
        let projection_result = project_runtime_tool_input_schema(&tool.parameters);
        if !projection_result.violations.is_empty() {
            projection
                .unavailable_original_names
                .insert(tool.name.clone());
            continue;
        }
        let schema = normalize_anthropic_json_schema(&projection_result.schema);
        let props = if let serde_json::Value::Object(map) = &schema {
            map.get("properties")
                .and_then(|v| v.as_object())
                .cloned()
                .unwrap_or_default()
        } else {
            serde_json::Map::new()
        };
        let required = if let serde_json::Value::Object(map) = &schema {
            map.get("required")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|s| s.as_str().map(String::from))
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default()
        } else {
            vec![]
        };
        projection.tools.push(AnthropicProjectedTool {
            original_name: tool.name.clone(),
            wire_name: to_wire_name(&tool.name),
            description: Some(tool.description.clone()),
            input_schema: AnthropicInputSchema {
                properties: props,
                required,
            },
        });
    }
    projection
}

/// Minimal Anthropic tool descriptor input.
#[derive(Debug, Clone)]
pub struct AnthropicToolDescriptor {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value,
}