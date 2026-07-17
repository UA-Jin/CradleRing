//! Shared utilities for Google Generative AI / Vertex providers.
//! 翻译自 packages/ai/src/providers/google-shared.ts
//!
//! Contains helpers for payload construction, response handling, and
//! thinking-part classification shared between google.ts and google-vertex.ts.

use std::collections::BTreeMap;

use llm_core::types::{Api, ImageContent, StopReason, TextContent, ThinkingContent};

/// Provider API types for Google.
pub type GoogleApiType = &'static str;

/// Thinking level for Gemini 3 models (mirrors Google's ThinkingLevel).
pub type GoogleThinkingLevel = String;

/// Tool-choice shape accepted by Gemini providers.
pub type GoogleToolChoice = String;

/// Thinking options carried in the provider options bag.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct GoogleThinkingOptions {
    pub enabled: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub budget_tokens: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub level: Option<GoogleThinkingLevel>,
}

/// Provider stream options for Google with tool-choice and thinking.
#[derive(Debug, Clone, Default)]
pub struct GoogleProviderOptions {
    pub base: llm_core::types::StreamOptions,
    pub tool_choice: Option<GoogleToolChoice>,
    pub thinking: Option<GoogleThinkingOptions>,
}

impl GoogleProviderOptions {
    pub fn from_base(base: llm_core::types::StreamOptions) -> Self {
        Self {
            base,
            tool_choice: None,
            thinking: None,
        }
    }
}

/// Determines whether a Gemini `Part` should be treated as "thinking".
///
/// `thought: true` is the definitive marker for thinking content.
/// `thoughtSignature` is encrypted thought context and may appear on any
/// part type; it does NOT indicate the part itself is thinking.
pub fn is_thinking_part(part: &GooglePart) -> bool {
    part.thought.unwrap_or(false)
}

/// Minimal Google response part shape.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct GooglePart {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thought: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thought_signature: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub function_call: Option<GoogleFunctionCall>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inline_data: Option<GoogleInlineData>,
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct GoogleFunctionCall {
    pub name: String,
    pub args: BTreeMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct GoogleInlineData {
    pub mime_type: String,
    pub data: String,
}

/// Retain thought signatures during streaming.
pub fn retain_thought_signature(prev: Option<&str>, next: Option<&str>) -> Option<String> {
    next.map(|s| s.to_string()).or_else(|| prev.map(|s| s.to_string()))
}

/// Map a Gemini `FinishReason` to a normalized `StopReason`.
pub fn map_google_finish_reason(reason: &str) -> (StopReason, Option<String>) {
    match reason {
        "STOP" => ("stop".to_string(), None),
        "MAX_TOKENS" => ("length".to_string(), None),
        "SAFETY" | "RECITATION" | "BLOCKLIST" | "PROHIBITED_CONTENT" | "SPII" => (
            "error".to_string(),
            Some(format!("Provider finish_reason: {}", reason)),
        ),
        other => (
            "error".to_string(),
            Some(format!("Provider finish_reason: {}", other)),
        ),
    }
}

/// Convert a runtime `TextContent` to a Gemini text `Part`.
pub fn text_content_to_part(text: &TextContent) -> GooglePart {
    GooglePart {
        text: Some(text.text.clone()),
        ..Default::default()
    }
}

/// Convert a runtime `ThinkingContent` to a Gemini thinking `Part`.
pub fn thinking_content_to_part(thinking: &ThinkingContent) -> GooglePart {
    GooglePart {
        thought: Some(true),
        text: Some(thinking.thinking.clone()),
        thought_signature: thinking.thinking_signature.clone(),
        ..Default::default()
    }
}

/// Convert a runtime `ImageContent` to a Gemini inline-data `Part`.
pub fn image_content_to_part(image: &ImageContent) -> Option<GooglePart> {
    Some(GooglePart {
        inline_data: Some(GoogleInlineData {
            mime_type: image.mime_type.clone(),
            data: image.data.clone(),
        }),
        ..Default::default()
    })
}

/// Strips the system-prompt cache boundary marker before sending to Google.
pub fn strip_system_prompt_cache_boundary(input: &str) -> String {
    input.replace(crate::utils::system_prompt_cache_boundary::SYSTEM_PROMPT_CACHE_BOUNDARY, "")
}

/// Build the system instruction text from a runtime context.
pub fn build_system_instruction(system_prompt: Option<&str>) -> Option<String> {
    system_prompt
        .map(strip_system_prompt_cache_boundary)
        .filter(|s| !s.is_empty())
}

/// Convenience constant: canonical Gemini API id.
pub const GOOGLE_GENERATIVE_AI: &str = "google-generative-ai";
pub const GOOGLE_VERTEX: &str = "google-vertex";

/// Type alias re-export for cross-provider parity.
pub type ApiKind = Api;