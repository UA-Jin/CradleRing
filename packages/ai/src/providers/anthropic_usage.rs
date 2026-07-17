//! Anthropic usage parsing.
//! 翻译自 packages/ai/src/providers/anthropic-usage.ts

use serde_json::Value;

#[derive(Debug, Clone, Default)]
pub struct AnthropicUsagePayload {
    pub input_tokens: Option<i64>,
    pub output_tokens: Option<i64>,
    pub cache_read_input_tokens: Option<i64>,
    pub cache_creation_input_tokens: Option<i64>,
    pub iterations: Option<Value>,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct AnthropicPromptUsageSnapshot {
    pub input: i64,
    pub cache_read: i64,
    pub cache_write: i64,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct AnthropicIterationUsageSnapshot {
    pub context_prompt_tokens: i64,
    pub total_tokens: i64,
}

#[derive(Debug, Clone)]
pub enum AnthropicIterationUsageResult {
    Absent,
    Invalid,
    Valid(AnthropicIterationUsageSnapshot),
}

fn read_token_count(value: Option<&Value>) -> Option<i64> {
    let v = value?;
    if let Some(n) = v.as_i64() {
        if n >= 0 {
            return Some(n);
        }
    }
    if let Some(f) = v.as_f64() {
        if f.is_finite() && f >= 0.0 {
            return Some(f as i64);
        }
    }
    None
}

fn parse_usage_payload(value: &Value) -> Option<AnthropicUsagePayload> {
    Some(AnthropicUsagePayload {
        input_tokens: read_token_count(value.get("input_tokens")),
        output_tokens: read_token_count(value.get("output_tokens")),
        cache_read_input_tokens: read_token_count(value.get("cache_read_input_tokens")),
        cache_creation_input_tokens: read_token_count(value.get("cache_creation_input_tokens")),
        iterations: value.get("iterations").cloned(),
    })
}

/// Reads a prompt token usage snapshot from an Anthropic usage payload.
pub fn read_anthropic_prompt_usage_snapshot(
    usage: &AnthropicUsagePayload,
) -> Option<AnthropicPromptUsageSnapshot> {
    let input = usage.input_tokens?;
    let cache_read = usage.cache_read_input_tokens.unwrap_or(0);
    let cache_write = usage.cache_creation_input_tokens.unwrap_or(0);
    Some(AnthropicPromptUsageSnapshot {
        input,
        cache_read,
        cache_write,
    })
}

/// Reads the last Anthropic iteration usage snapshot.
pub fn read_last_anthropic_iteration_usage(
    usage: &AnthropicUsagePayload,
) -> AnthropicIterationUsageResult {
    let Some(iterations) = usage.iterations.as_ref() else {
        return AnthropicIterationUsageResult::Absent;
    };
    let Some(arr) = iterations.as_array() else {
        return AnthropicIterationUsageResult::Invalid;
    };
    let Some(last) = arr.last() else {
        return AnthropicIterationUsageResult::Invalid;
    };
    let Some(obj) = last.as_object() else {
        return AnthropicIterationUsageResult::Invalid;
    };
    let context = match read_token_count(obj.get("context_prompt_tokens")) {
        Some(v) => v,
        None => return AnthropicIterationUsageResult::Invalid,
    };
    let total = match read_token_count(obj.get("total_tokens")) {
        Some(v) => v,
        None => return AnthropicIterationUsageResult::Invalid,
    };
    AnthropicIterationUsageResult::Valid(AnthropicIterationUsageSnapshot {
        context_prompt_tokens: context,
        total_tokens: total,
    })
}

/// Parse a raw JSON value into an Anthropic usage payload.
pub fn parse_anthropic_usage_payload(value: &Value) -> Option<AnthropicUsagePayload> {
    parse_usage_payload(value)
}