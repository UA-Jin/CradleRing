//! Anthropic refusal handling.
//! 翻译自 packages/ai/src/providers/anthropic-refusal.ts

use llm_core::utils::diagnostics::AssistantMessageDiagnostic;

#[derive(Debug, Clone, Default)]
pub struct AnthropicRefusalOutput {
    pub stop_reason: String,
    pub error_message: Option<String>,
    pub diagnostics: Option<Vec<AssistantMessageDiagnostic>>,
}

#[derive(Debug, Clone, Default)]
pub struct AnthropicRefusalDetails {
    pub category: Option<String>,
    pub explanation: Option<String>,
}

fn read_nullable_string(value: &serde_json::Value) -> Option<String> {
    value
        .as_str()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

fn read_anthropic_refusal_details(value: &serde_json::Value) -> AnthropicRefusalDetails {
    if !value.is_object() {
        return AnthropicRefusalDetails::default();
    }
    let obj = value.as_object().unwrap();
    AnthropicRefusalDetails {
        category: obj.get("category").and_then(read_nullable_string),
        explanation: obj.get("explanation").and_then(read_nullable_string),
    }
}

fn format_anthropic_refusal_message(details: &AnthropicRefusalDetails) -> String {
    let category = details
        .category
        .as_deref()
        .map(|c| format!(" (category: {})", c))
        .unwrap_or_default();
    let explanation = match details.explanation.as_deref() {
        Some(e) => format!(": {}", e),
        None => ".".to_string(),
    };
    format!("Anthropic refusal{}{}", category, explanation)
}

/// Apply an Anthropic refusal event to the in-flight assistant message.
pub fn apply_anthropic_refusal(
    output: &mut AnthropicRefusalOutput,
    stop_details: &serde_json::Value,
    provider: &str,
) {
    let details = read_anthropic_refusal_details(stop_details);
    output.stop_reason = "error".to_string();
    output.error_message = Some(format_anthropic_refusal_message(&details));

    let mut details_map = std::collections::BTreeMap::new();
    details_map.insert("provider".to_string(), serde_json::Value::String(provider.to_string()));
    if let Some(c) = &details.category {
        details_map.insert("category".to_string(), serde_json::Value::String(c.clone()));
    }
    if let Some(e) = &details.explanation {
        details_map.insert("explanation".to_string(), serde_json::Value::String(e.clone()));
    }
    let diagnostic = AssistantMessageDiagnostic {
        type_: "provider_refusal".to_string(),
        timestamp: chrono::Utc::now().timestamp_millis(),
        error: None,
        details: Some(details_map),
    };
    let list = output.diagnostics.get_or_insert_with(Vec::new);
    list.push(diagnostic);
}