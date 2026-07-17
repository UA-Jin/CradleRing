// LLM core tool-call validation.
// 翻译自 packages/llm-core/src/validation.ts
//
// 简化版实现：保留 TS 版的 coercion + validator 流程，但用 serde_json::Value
// 表示 schema 而非 TypeBox 编译对象。strict JSON-schema 校验留给上层 gateway
// 或 provider 适配层；本模块专注于参数 coercion 与结构性 sanity check。

use std::collections::BTreeMap;

use once_cell::sync::Lazy;

use regex::Regex;

use crate::types::{Tool, ToolCall};

/// Maximum string length accepted for schema-gated JSON coercion.
pub const MAX_JSON_COERCE_LENGTH: usize = 64 * 1024;

static JSON_NUMBER_TOKEN_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^[+-]?(?:(?:\d+\.?\d*)|(?:\.\d+))(?:e[+-]?\d+)?$").unwrap()
});

fn is_record(value: &serde_json::Value) -> bool {
    value.is_object()
}

fn get_schema_types(schema: &serde_json::Value) -> Vec<String> {
    match schema.get("type") {
        Some(serde_json::Value::String(s)) => vec![s.clone()],
        Some(serde_json::Value::Array(arr)) => arr
            .iter()
            .filter_map(|v| v.as_str().map(|s| s.to_string()))
            .collect(),
        _ => Vec::new(),
    }
}

fn matches_json_type(value: &serde_json::Value, type_: &str) -> bool {
    match type_ {
        "number" => value.is_number(),
        "integer" => value.is_i64() || value.is_u64(),
        "boolean" => value.is_boolean(),
        "string" => value.is_string(),
        "null" => value.is_null(),
        "array" => value.is_array(),
        "object" => is_record(value) && !value.is_array(),
        _ => false,
    }
}

fn parse_json_number_string(value: &str) -> Option<f64> {
    let trimmed = value.trim();
    if trimmed.is_empty() || !JSON_NUMBER_TOKEN_RE.is_match(trimmed) {
        return None;
    }
    let parsed: f64 = trimmed.parse().ok()?;
    if parsed.is_finite() {
        Some(parsed)
    } else {
        None
    }
}

fn parse_json_integer_string(value: &str) -> Option<i64> {
    let parsed = parse_json_number_string(value)?;
    if parsed.is_finite() && parsed.fract() == 0.0 && parsed >= i64::MIN as f64 && parsed <= i64::MAX as f64
    {
        Some(parsed as i64)
    } else {
        None
    }
}

fn coerce_primitive_by_type(value: serde_json::Value, type_: &str) -> serde_json::Value {
    match type_ {
        "number" => {
            if value.is_null() {
                return serde_json::json!(0);
            }
            if let Some(s) = value.as_str() {
                if !s.trim().is_empty() {
                    if let Some(parsed) = parse_json_number_string(s) {
                        return serde_json::json!(parsed);
                    }
                }
            }
            if let Some(b) = value.as_bool() {
                return serde_json::json!(if b { 1 } else { 0 });
            }
            value
        }
        "integer" => {
            if value.is_null() {
                return serde_json::json!(0);
            }
            if let Some(s) = value.as_str() {
                if !s.trim().is_empty() {
                    if let Some(parsed) = parse_json_integer_string(s) {
                        return serde_json::json!(parsed);
                    }
                }
            }
            if let Some(b) = value.as_bool() {
                return serde_json::json!(if b { 1 } else { 0 });
            }
            value
        }
        "boolean" => {
            if value.is_null() {
                return serde_json::json!(false);
            }
            if let Some(s) = value.as_str() {
                if s == "true" {
                    return serde_json::json!(true);
                }
                if s == "false" {
                    return serde_json::json!(false);
                }
            }
            if let Some(n) = value.as_f64() {
                if n == 1.0 {
                    return serde_json::json!(true);
                }
                if n == 0.0 {
                    return serde_json::json!(false);
                }
            }
            value
        }
        "string" => {
            if value.is_null() {
                return serde_json::json!("");
            }
            if value.is_number() || value.is_boolean() {
                return serde_json::json!(value.to_string());
            }
            value
        }
        "array" => {
            if let Some(s) = value.as_str() {
                let trimmed = s.trim();
                if !trimmed.is_empty() && trimmed.len() <= MAX_JSON_COERCE_LENGTH {
                    if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(trimmed) {
                        if parsed.is_array() {
                            return parsed;
                        }
                    }
                }
            }
            value
        }
        "object" => {
            if let Some(s) = value.as_str() {
                let trimmed = s.trim();
                if !trimmed.is_empty() && trimmed.len() <= MAX_JSON_COERCE_LENGTH {
                    if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(trimmed) {
                        if parsed.is_object() && !parsed.is_array() {
                            return parsed;
                        }
                    }
                }
            }
            value
        }
        "null" => {
            if value.is_string() && value == "" || value == serde_json::json!(0) || value == serde_json::json!(false) {
                return serde_json::json!(null);
            }
            value
        }
        _ => value,
    }
}

fn apply_schema_object_coercion(value: &mut serde_json::Value, schema: &serde_json::Value) {
    let Some(obj) = value.as_object_mut() else {
        return;
    };
    let defined_keys: std::collections::BTreeSet<String> = match schema.get("properties") {
        Some(serde_json::Value::Object(props)) => props.keys().cloned().collect(),
        _ => std::collections::BTreeSet::new(),
    };

    if let Some(serde_json::Value::Object(props)) = schema.get("properties") {
        for (key, property_schema) in props {
            if let Some(entry) = obj.get_mut(key) {
                let coerced = coerce_with_json_schema(entry.clone(), property_schema);
                *entry = coerced;
            }
        }
    }

    if let Some(serde_json::Value::Object(additional)) = schema.get("additionalProperties") {
        let keys: Vec<String> = obj.keys().cloned().collect();
        for key in keys {
            if !defined_keys.contains(&key) {
                if let Some(entry) = obj.get_mut(&key) {
                    let coerced = coerce_with_json_schema(entry.clone(), &serde_json::Value::Object(additional.clone()));
                    *entry = coerced;
                }
            }
        }
    }
}

fn apply_schema_array_coercion(value: &mut serde_json::Value, schema: &serde_json::Value) {
    let Some(arr) = value.as_array_mut() else {
        return;
    };
    if let Some(serde_json::Value::Array(items)) = schema.get("items") {
        for (index, item) in arr.iter_mut().enumerate() {
            if let Some(item_schema) = items.get(index) {
                let coerced = coerce_with_json_schema(item.clone(), item_schema);
                *item = coerced;
            }
        }
        return;
    }
    if let Some(items) = schema.get("items") {
        for item in arr.iter_mut() {
            let coerced = coerce_with_json_schema(item.clone(), items);
            *item = coerced;
        }
    }
}

fn coerce_with_union_schema(
    value: serde_json::Value,
    schemas: &[serde_json::Value],
) -> serde_json::Value {
    if value.is_null() {
        for schema in schemas {
            let types = get_schema_types(schema);
            if types.iter().any(|t| t == "null") {
                return value;
            }
        }
    }
    for schema in schemas {
        let candidate = value.clone();
        let coerced = coerce_with_json_schema(candidate, schema);
        // Simplified: structural match (presence of required fields) is best-effort.
        if structural_check(&coerced, schema) {
            return coerced;
        }
    }
    value
}

fn structural_check(value: &serde_json::Value, schema: &serde_json::Value) -> bool {
    let types = get_schema_types(schema);
    if !types.is_empty() && !types.iter().any(|t| matches_json_type(value, t)) {
        return false;
    }
    if let Some(serde_json::Value::Object(required)) = schema.get("required") {
        if let Some(obj) = value.as_object() {
            for key in required.keys() {
                if !obj.contains_key(key) {
                    return false;
                }
            }
        } else {
            return false;
        }
    }
    if let Some(serde_json::Value::Object(props)) = schema.get("properties") {
        if let Some(obj) = value.as_object() {
            for (key, prop_schema) in props {
                if let Some(child) = obj.get(key) {
                    if !structural_check(child, prop_schema) {
                        return false;
                    }
                }
            }
        }
    }
    true
}

fn coerce_with_json_schema(
    value: serde_json::Value,
    schema: &serde_json::Value,
) -> serde_json::Value {
    let mut next_value = value;

    if let Some(serde_json::Value::Array(all_of)) = schema.get("allOf") {
        for nested in all_of {
            next_value = coerce_with_json_schema(next_value, nested);
        }
    }

    if let Some(serde_json::Value::Array(any_of)) = schema.get("anyOf") {
        next_value = coerce_with_union_schema(next_value, any_of);
    }

    if let Some(serde_json::Value::Array(one_of)) = schema.get("oneOf") {
        next_value = coerce_with_union_schema(next_value, one_of);
    }

    let schema_types = get_schema_types(schema);
    let matches_union_member = schema_types.len() > 1
        && schema_types
            .iter()
            .any(|t| matches_json_type(&next_value, t));
    if !schema_types.is_empty() && !matches_union_member {
        for type_ in &schema_types {
            let candidate = coerce_primitive_by_type(next_value.clone(), type_);
            if candidate != next_value {
                next_value = candidate;
                break;
            }
        }
    }

    if schema_types.iter().any(|t| t == "object") && is_record(&next_value) && !next_value.is_array() {
        apply_schema_object_coercion(&mut next_value, schema);
    }

    if schema_types.iter().any(|t| t == "array") && next_value.is_array() {
        apply_schema_array_coercion(&mut next_value, schema);
    }

    next_value
}

/// Validates tool arguments against TypeBox or plain JSON-schema parameters.
///
/// Returns the (possibly coerced) arguments. In this Rust port, the function
/// falls back to coercion plus structural sanity checks; richer schema
/// validation (TypeBox `Compile` equivalent) is delegated to provider plugins.
pub fn validate_tool_arguments(tool: &Tool, tool_call: &ToolCall) -> Result<serde_json::Value, String> {
    let mut args = serde_json::Map::new();
    for (k, v) in &tool_call.arguments {
        args.insert(k.clone(), v.clone());
    }
    let mut args_value = serde_json::Value::Object(args);

    // Mirror the TS provider-facing coercions so model-emitted string numbers validate.
    let coerced = coerce_with_json_schema(args_value.clone(), &tool.parameters);
    args_value = coerced;

    // Structural sanity check.
    if !structural_check(&args_value, &tool.parameters) {
        return Err(format!(
            "Validation failed for tool \"{}\": structural schema check failed\n\nReceived arguments:\n{}",
            tool_call.name,
            serde_json::to_string_pretty(&tool_call.arguments).unwrap_or_default()
        ));
    }

    Ok(args_value)
}

/// Finds the target tool and validates/coerces a model-emitted tool call.
pub fn validate_tool_call(tools: &[Tool], tool_call: &ToolCall) -> Result<serde_json::Value, String> {
    let tool = tools
        .iter()
        .find(|t| t.name == tool_call.name)
        .ok_or_else(|| format!("Tool \"{}\" not found", tool_call.name))?;
    validate_tool_arguments(tool, tool_call)
}

/// Helper: create a BTreeMap-backed ToolCall.arguments from an iterator.
pub fn tool_call_arguments_from<I, K, V>(entries: I) -> BTreeMap<String, serde_json::Value>
where
    I: IntoIterator<Item = (K, V)>,
    K: Into<String>,
    V: Into<serde_json::Value>,
{
    entries
        .into_iter()
        .map(|(k, v)| (k.into(), v.into()))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_tool(parameters: serde_json::Value) -> Tool {
        Tool {
            name: "test-tool".to_string(),
            description: "test".to_string(),
            parameters,
        }
    }

    #[test]
    fn coerces_decimal_string_numbers() {
        let tool = make_tool(serde_json::json!({
            "type": "object",
            "properties": {
                "amount": { "type": "number" },
                "count": { "type": "integer" }
            },
            "required": ["amount", "count"],
            "additionalProperties": false
        }));
        let mut args = BTreeMap::new();
        args.insert("amount".to_string(), serde_json::json!("1e3"));
        args.insert("count".to_string(), serde_json::json!("+3"));
        let call = ToolCall {
            type_: "toolCall".to_string(),
            id: "call-1".to_string(),
            name: "test-tool".to_string(),
            arguments: args,
            thought_signature: None,
            execution_mode: None,
        };
        let result = validate_tool_arguments(&tool, &call).unwrap();
        assert_eq!(result["amount"], serde_json::json!(1000.0));
        assert_eq!(result["count"], serde_json::json!(3));
    }

    #[test]
    fn coerces_stringified_json_array() {
        let tool = make_tool(serde_json::json!({
            "type": "object",
            "properties": {
                "tags": { "type": "array", "items": { "type": "string" } }
            },
            "required": ["tags"],
            "additionalProperties": false
        }));
        let mut args = BTreeMap::new();
        args.insert("tags".to_string(), serde_json::json!("[\"test\",\"debug\"]"));
        let call = ToolCall {
            type_: "toolCall".to_string(),
            id: "call-2".to_string(),
            name: "test-tool".to_string(),
            arguments: args,
            thought_signature: None,
            execution_mode: None,
        };
        let result = validate_tool_arguments(&tool, &call).unwrap();
        assert_eq!(result["tags"], serde_json::json!(["test", "debug"]));
    }

    #[test]
    fn preserves_null_in_anyof() {
        let tool = make_tool(serde_json::json!({
            "type": "object",
            "properties": {
                "insight_id": { "anyOf": [{ "type": "string" }, { "type": "null" }] },
                "cluster_name": { "type": "string" }
            },
            "required": ["cluster_name"],
            "additionalProperties": false
        }));
        let mut args = BTreeMap::new();
        args.insert("insight_id".to_string(), serde_json::json!(null));
        args.insert("cluster_name".to_string(), serde_json::json!("testenv"));
        let call = ToolCall {
            type_: "toolCall".to_string(),
            id: "call-3".to_string(),
            name: "test-tool".to_string(),
            arguments: args,
            thought_signature: None,
            execution_mode: None,
        };
        let result = validate_tool_arguments(&tool, &call).unwrap();
        assert_eq!(result["insight_id"], serde_json::json!(null));
        assert_eq!(result["cluster_name"], serde_json::json!("testenv"));
    }
}