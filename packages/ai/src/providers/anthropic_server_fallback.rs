//! Anthropic server-side fallback handling.
//! 翻译自 packages/ai/src/providers/anthropic-server-fallback.ts

use llm_core::utils::diagnostics::AssistantMessageDiagnostic;

/// Anthropic beta that re-serves safety refusals on an allowed fallback model.
pub const ANTHROPIC_SERVER_SIDE_FALLBACK_BETA: &str = "server-side-fallback-2026-06-01";

/// Anthropic documents claude-opus-4-8 as the allowed fallback for claude-fable-5.
pub const CLAUDE_FABLE_5_FALLBACK_MODEL: &str = "claude-opus-4-8";

/// Fallback-served turns bill at the serving model's rates.
pub const CLAUDE_FABLE_5_FALLBACK_MODEL_COST: AnthropicFallbackCost = AnthropicFallbackCost {
    input: 5.0,
    output: 25.0,
    cache_read: 0.5,
    cache_write: 6.25,
};

#[derive(Debug, Clone, Copy)]
pub struct AnthropicFallbackCost {
    pub input: f64,
    pub output: f64,
    pub cache_read: f64,
    pub cache_write: f64,
}

/// Builds the configured server-side fallback list.
pub fn build_anthropic_server_side_fallbacks() -> Vec<AnthropicFallbackModel> {
    vec![AnthropicFallbackModel {
        model: CLAUDE_FABLE_5_FALLBACK_MODEL.to_string(),
    }]
}

#[derive(Debug, Clone)]
pub struct AnthropicFallbackModel {
    pub model: String,
}

#[derive(Debug, Clone, Default)]
pub struct AnthropicFallbackBoundary {
    pub from_model: Option<String>,
    pub to_model: Option<String>,
}

fn read_boundary_model(value: &serde_json::Value) -> Option<String> {
    let model = value.get("model")?;
    model
        .as_str()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

/// Reads a `fallback` content block marking where one model's output gives way to the next.
pub fn read_anthropic_fallback_boundary(block: &serde_json::Value) -> Option<AnthropicFallbackBoundary> {
    if block.get("type")?.as_str() != Some("fallback") {
        return None;
    }
    Some(AnthropicFallbackBoundary {
        from_model: block.get("from").and_then(read_boundary_model),
        to_model: block.get("to").and_then(read_boundary_model),
    })
}

/// Apply an Anthropic fallback boundary to the in-flight assistant output.
pub fn apply_anthropic_fallback_boundary(params: FallbackBoundaryApplyParams<'_>) {
    let boundary = &params.boundary;
    let output = params.output;
    // Keep only text blocks.
    output
        .content
        .retain(|block| block.get("type").and_then(|v| v.as_str()) == Some("text"));

    if let Some(to) = &boundary.to_model {
        output.response_model = Some(to.clone());
    }

    let mut details = std::collections::BTreeMap::new();
    details.insert(
        "provider".to_string(),
        serde_json::Value::String(params.provider.to_string()),
    );
    if let Some(f) = &boundary.from_model {
        details.insert("fromModel".to_string(), serde_json::Value::String(f.clone()));
    }
    if let Some(t) = &boundary.to_model {
        details.insert("toModel".to_string(), serde_json::Value::String(t.clone()));
    }
    let diag = AssistantMessageDiagnostic {
        type_: "provider_fallback".to_string(),
        timestamp: chrono::Utc::now().timestamp_millis(),
        error: None,
        details: Some(details),
    };
    let list = output.diagnostics.get_or_insert_with(Vec::new);
    list.push(diag);
}

/// Parameters for `apply_anthropic_fallback_boundary`.
pub struct FallbackBoundaryApplyParams<'a> {
    pub output: &'a mut AnthropicFallbackOutput,
    pub boundary: AnthropicFallbackBoundary,
    pub provider: &'a str,
}

/// Mutable output shape consumed by `apply_anthropic_fallback_boundary`.
#[derive(Debug, Default)]
pub struct AnthropicFallbackOutput {
    pub content: Vec<serde_json::Value>,
    pub response_model: Option<String>,
    pub diagnostics: Option<Vec<AssistantMessageDiagnostic>>,
}