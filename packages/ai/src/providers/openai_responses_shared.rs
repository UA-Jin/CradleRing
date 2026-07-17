//! OpenAI Responses API shared helpers.
//! 翻译自 packages/ai/src/providers/openai-responses-shared.ts
//!
//! Provides payload-construction and response-parsing utilities shared
//! between `openai-responses` and `azure-openai-responses`.

use std::collections::BTreeMap;

use serde_json::Value;

/// One content part shape used by the Responses API.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct ResponsesContentPart {
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub type_: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub delta: Option<String>,
}

/// One input message accepted by the Responses API.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct ResponsesInputMessage {
    pub role: String,
    pub content: Vec<ResponsesContentPart>,
}

/// Build a minimal Responses API request body.
pub fn build_responses_request_body(
    model: &str,
    input: Vec<ResponsesInputMessage>,
) -> Value {
    serde_json::json!({
        "model": model,
        "input": input,
    })
}

/// Extracts the visible text from a Responses output array.
pub fn extract_responses_output_text(output: &[Value]) -> String {
    let mut buf = String::new();
    for item in output {
        if let Some(Value::Array(content)) = item.get("content") {
            for part in content {
                if let Some(text) = part.get("text").and_then(|v| v.as_str()) {
                    buf.push_str(text);
                }
            }
        }
    }
    buf
}

/// Map Responses API stop reason to normalized stop reason.
pub fn map_responses_finish_reason(reason: Option<&str>) -> (String, Option<String>) {
    match reason {
        Some("stop") | Some("completed") => ("stop".to_string(), None),
        Some("length") | Some("max_output_tokens") => ("length".to_string(), None),
        Some("tool_calls") | Some("tool_use") => ("toolUse".to_string(), None),
        Some("content_filter") => (
            "error".to_string(),
            Some("Provider finish_reason: content_filter".to_string()),
        ),
        Some(other) => (
            "error".to_string(),
            Some(format!("Provider finish_reason: {}", other)),
        ),
        None => ("stop".to_string(), None),
    }
}

/// Build a tool list payload for the Responses API.
pub fn build_responses_tools(tools: &[Value]) -> Value {
    serde_json::Value::Array(tools.to_vec())
}

/// Helper to convert BTreeMap to JSON Value (convenience).
pub fn btreemap_to_object(map: &BTreeMap<String, Value>) -> Value {
    let mut obj = serde_json::Map::new();
    for (k, v) in map {
        obj.insert(k.clone(), v.clone());
    }
    Value::Object(obj)
}