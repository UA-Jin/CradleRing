// Gateway Protocol: normalized validation error formatting.
// 翻译自 packages/gateway-protocol/src/validation-errors.ts
//
// Normalized validation error shape exposed by every protocol validator. The
// helpers convert arrays of validator errors into compact operator-facing
// failure text.

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Normalized validation error shape exposed by every protocol validator.
/// 对齐 TS: `type ValidationError = { ... }`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ValidationError {
    /// Failed schema keyword, when the validator can report one.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub keyword: Option<String>,
    /// JSON-pointer path to the failing data location.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instance_path: Option<String>,
    /// JSON-pointer path to the failing schema location.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schema_path: Option<String>,
    /// Validator-specific keyword parameters for richer diagnostics.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<Value>,
    /// Human-readable validation message.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

fn first_string_param(value: Option<&Value>) -> Option<String> {
    match value {
        Some(Value::String(s)) if !s.trim().is_empty() => Some(s.clone()),
        Some(Value::Array(arr)) => arr.iter().find_map(|entry| match entry {
            Value::String(s) if !s.trim().is_empty() => Some(s.clone()),
            _ => None,
        }),
        _ => None,
    }
}

/// Convert validator errors into compact operator-facing failure text.
/// 对齐 TS: `formatValidationErrors(errors)`.
pub fn format_validation_errors(errors: Option<&[ValidationError]>) -> String {
    let Some(errors) = errors else {
        return "unknown validation error".to_string();
    };
    if errors.is_empty() {
        return "unknown validation error".to_string();
    }

    let mut parts: Vec<String> = Vec::new();

    for err in errors {
        let keyword = err.keyword.as_deref().unwrap_or("");
        let instance_path = err.instance_path.as_deref().unwrap_or("");
        let params = err.params.as_ref();

        if keyword == "additionalProperties" {
            let additional_property = first_string_param(params.and_then(|p| p.get("additionalProperty")))
                .or_else(|| first_string_param(params.and_then(|p| p.get("additionalProperties"))));
            if let Some(prop) = additional_property {
                let where_ = if !instance_path.is_empty() {
                    format!("at {}", instance_path)
                } else {
                    "at root".to_string()
                };
                parts.push(format!("{}: unexpected property '{}'", where_, prop));
                continue;
            }
        }

        if keyword == "required" {
            let missing_property = first_string_param(params.and_then(|p| p.get("missingProperty")))
                .or_else(|| first_string_param(params.and_then(|p| p.get("requiredProperties"))));
            if let Some(prop) = missing_property {
                let where_ = if !instance_path.is_empty() {
                    format!("at {}: ", instance_path)
                } else {
                    String::new()
                };
                parts.push(format!("{}must have required property '{}'", where_, prop));
                continue;
            }
        }

        // TypeBox reports conditional required-property misses through if/then
        // keywords, which otherwise hide the actionable missing-property
        // context.
        let failing_keyword = params
            .and_then(|p| p.get("failingKeyword"))
            .and_then(Value::as_str)
            .unwrap_or("");
        let message = if keyword == "then" || (keyword == "if" && failing_keyword == "then") {
            "must have required conditional properties".to_string()
        } else {
            err.message
                .as_deref()
                .filter(|m| !m.trim().is_empty())
                .map(|m| m.to_string())
                .unwrap_or_else(|| "validation error".to_string())
        };
        let where_ = if !instance_path.is_empty() {
            format!("at {}: ", instance_path)
        } else {
            String::new()
        };
        parts.push(format!("{}{}", where_, message));
    }

    let unique: Vec<String> = parts
        .into_iter()
        .filter(|p| !p.trim().is_empty())
        .collect::<std::collections::BTreeSet<_>>()
        .into_iter()
        .collect();
    if unique.is_empty() {
        "unknown validation error".to_string()
    } else {
        unique.join("; ")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn empty_errors_returns_unknown() {
        assert_eq!(format_validation_errors(None), "unknown validation error");
        assert_eq!(format_validation_errors(Some(&[])), "unknown validation error");
    }

    #[test]
    fn additional_properties_message() {
        let err = ValidationError {
            keyword: Some("additionalProperties".to_string()),
            instance_path: Some("/foo".to_string()),
            params: Some(json!({"additionalProperty": "bar"})),
            message: None,
            schema_path: None,
        };
        assert_eq!(
            format_validation_errors(Some(&[err])),
            "at /foo: unexpected property 'bar'"
        );
    }

    #[test]
    fn required_property_message() {
        let err = ValidationError {
            keyword: Some("required".to_string()),
            instance_path: Some("/foo".to_string()),
            params: Some(json!({"missingProperty": "bar"})),
            message: None,
            schema_path: None,
        };
        assert_eq!(
            format_validation_errors(Some(&[err])),
            "at /foo: must have required property 'bar'"
        );
    }

    #[test]
    fn then_keyword_uses_conditional_message() {
        let err = ValidationError {
            keyword: Some("then".to_string()),
            instance_path: Some("/x".to_string()),
            params: Some(json!({})),
            message: None,
            schema_path: None,
        };
        assert_eq!(
            format_validation_errors(Some(&[err])),
            "at /x: must have required conditional properties"
        );
    }

    #[test]
    fn uses_message_field_as_fallback() {
        let err = ValidationError {
            keyword: Some("type".to_string()),
            instance_path: None,
            params: None,
            message: Some("expected number".to_string()),
            schema_path: None,
        };
        assert_eq!(format_validation_errors(Some(&[err])), "expected number");
    }
}
