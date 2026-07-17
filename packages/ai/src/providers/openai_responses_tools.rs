//! OpenAI Responses API tool normalization.
//! 翻译自 packages/ai/src/providers/openai-responses-tools.ts
//!
//! Normalizes tool descriptors for Responses API payloads, including
//! strict-mode schema normalization.

use serde_json::Value;

use crate::providers::agent_tools_parameter_schema::{
    normalize_tool_parameter_schema, ToolSchemaModelCompat,
};
use crate::providers::openai_tool_schema::normalize_strict_openai_json_schema;

/// One normalized Responses tool descriptor.
#[derive(Debug, Clone, Default)]
pub struct ResponsesTool {
    pub name: String,
    pub description: Option<String>,
    pub parameters: Value,
    pub strict: bool,
}

/// Normalize a list of tool descriptors for Responses API use.
pub fn normalize_responses_tools(
    tools: &[ResponsesToolInput],
    model_compat: Option<&ToolSchemaModelCompat>,
) -> Vec<ResponsesTool> {
    let mut out = Vec::new();
    for tool in tools {
        let normalized = normalize_tool_parameter_schema(&tool.parameters, model_compat);
        let strict = normalize_strict_openai_json_schema(Some(&normalized), None);
        out.push(ResponsesTool {
            name: tool.name.clone(),
            description: tool.description.clone(),
            parameters: strict,
            strict: true,
        });
    }
    out
}

/// Raw tool descriptor input for `normalize_responses_tools`.
#[derive(Debug, Clone)]
pub struct ResponsesToolInput {
    pub name: String,
    pub description: Option<String>,
    pub parameters: Value,
}