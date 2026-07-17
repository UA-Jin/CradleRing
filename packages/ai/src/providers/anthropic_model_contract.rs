//! Anthropic model-bound thinking contract.
//! 翻译自 packages/ai/src/providers/anthropic-model-contract.ts
//!
//! Mirrors the TS module: model-bound thinking cannot be exposed or
//! replayed after a model switch. Re-exports the Claude resolvers from
//! llm-core and provides helpers that decide whether streamed output
//! must wait for terminal refusal, whether thinking must use the
//! adaptive contract, etc.

use once_cell::sync::Lazy;
use regex::Regex;
use serde_json::Value;

pub use llm_core::model_contracts::anthropic::{
    requires_claude_default_sampling, requires_claude_mandatory_adaptive_thinking,
    resolve_claude_fable_5_model_identity, resolve_claude_model_identity,
    resolve_claude_mythos_5_model_identity, resolve_claude_native_thinking_level_map,
    resolve_claude_sonnet_5_model_identity, supports_claude_adaptive_thinking,
    supports_claude_native_max_effort, supports_claude_native_xhigh_effort,
};

fn normalize_model_id(model_id: Option<&str>) -> String {
    let raw = model_id.unwrap_or("").trim().to_lowercase();
    let unprefixed = raw
        .strip_prefix("anthropic/")
        .unwrap_or(&raw)
        .to_string();
    static SEP_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"[._\s]+").unwrap());
    SEP_RE.replace_all(&unprefixed, "-").into_owned()
}

fn normalize_api(api: Option<&str>) -> String {
    let normalized = api.unwrap_or("").trim().to_lowercase();
    if normalized == "openclaw-anthropic-messages-transport" {
        return "anthropic-messages".to_string();
    }
    normalized
}

fn has_concrete_response_model(model_id: Option<&str>, response_model_id: Option<&str>) -> bool {
    let resp = normalize_model_id(response_model_id);
    if resp.is_empty() {
        return false;
    }
    let req = normalize_model_id(model_id);
    resp != req
}

/// Narrow model-ref view used by these helpers.
#[derive(Debug, Clone, Default)]
pub struct ModelRefShape {
    pub id: Option<String>,
    pub api: Option<String>,
    pub params: Option<std::collections::BTreeMap<String, Value>>,
}

impl ModelRefShape {
    fn claude_ref(&self) -> llm_core::model_contracts::anthropic::ClaudeModelRef {
        llm_core::model_contracts::anthropic::ClaudeModelRef {
            id: self.id.clone(),
            params: self.params.clone(),
            thinking_level_map: None,
        }
    }
}

/// Returns true when the model uses the Claude Fable 5 messages contract.
pub fn uses_claude_fable_5_messages_contract(model: &ModelRefShape) -> bool {
    if normalize_api(model.api.as_deref()) != "anthropic-messages" {
        return false;
    }
    let r = model.claude_ref();
    resolve_claude_fable_5_model_identity(&r).is_some()
}

/// Return whether streamed output must wait for the terminal refusal decision.
pub fn uses_claude_streaming_refusal_contract(model: &ModelRefShape) -> bool {
    if normalize_api(model.api.as_deref()) != "anthropic-messages" {
        return false;
    }
    let r = model.claude_ref();
    resolve_claude_fable_5_model_identity(&r).is_some()
        || resolve_claude_mythos_5_model_identity(&r).is_some()
        || resolve_claude_sonnet_5_model_identity(&r).is_some()
}

/// Returns true when the model requires adaptive thinking.
pub fn requires_claude_adaptive_thinking(model: &ModelRefShape) -> bool {
    normalize_api(model.api.as_deref()) == "anthropic-messages"
        && requires_claude_mandatory_adaptive_thinking(&model.claude_ref())
}

/// Resolves the bound thinking replay mode for a context.
pub fn resolve_model_bound_thinking_replay_mode(
    request_model_id: Option<&str>,
    response_model_id: Option<&str>,
) -> &'static str {
    if has_concrete_response_model(request_model_id, response_model_id) {
        "drop"
    } else {
        "replay"
    }
}