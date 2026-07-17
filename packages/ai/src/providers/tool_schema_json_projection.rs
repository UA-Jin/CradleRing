//! Runtime tool input schema projection.
//! 翻译自 packages/ai/src/providers/tool-schema-json-projection.ts
//!
//! Validates that arbitrary input values can be serialized to JSON before
//! embedding them in provider payloads, and emits diagnostic violations
//! when they cannot.

use serde_json::Value;

/// One projected tool input schema plus validation violations.
#[derive(Debug, Clone, Default)]
pub struct RuntimeToolInputSchemaProjection {
    pub schema: Value,
    pub violations: Vec<String>,
}

fn is_json_value(value: &Value) -> bool {
    match value {
        Value::Null => true,
        Value::Bool(_) => true,
        Value::Number(_) => true,
        Value::String(_) => true,
        Value::Array(arr) => arr.iter().all(is_json_value),
        Value::Object(obj) => obj.values().all(is_json_value),
    }
}

/// Validate that `value` can be embedded as a JSON tool-input schema.
pub fn serialize_tool_input_schema(value: &Value, path: &str) -> RuntimeToolInputSchemaProjection {
    if !is_json_value(value) {
        return RuntimeToolInputSchemaProjection {
            schema: Value::Object(Default::default()),
            violations: vec![format!("{} is not JSON-serializable", path)],
        };
    }
    RuntimeToolInputSchemaProjection {
        schema: value.clone(),
        violations: vec![],
    }
}

/// Project a runtime tool input schema to a JSON-safe representation.
pub fn project_runtime_tool_input_schema(value: &Value) -> RuntimeToolInputSchemaProjection {
    serialize_tool_input_schema(value, "tool.parameters")
}