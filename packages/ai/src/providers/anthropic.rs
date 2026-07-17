//! Anthropic Messages API provider adapter.
//! 翻译自 packages/ai/src/providers/anthropic.ts
//!
//! Public entry points expose the streaming / simple-stream functions
//! referenced by `register_builtins`. The full request/response plumbing
//! is encapsulated in supporting modules (auth headers, refusal, usage,
//! model contract, thinking replay, etc.) and re-exported here.

use std::future::Future;
use std::pin::Pin;

use llm_core::types::{
    AssistantMessage, AssistantMessageEventStreamContract, Context, Model, SimpleStreamOptions,
    StreamOptions,
};

use crate::providers::anthropic_auth_headers::{
    omit_foundry_bearer_credential_headers, uses_foundry_bearer_auth,
};
use crate::providers::anthropic_model_contract::{
    requires_claude_adaptive_thinking, resolve_model_bound_thinking_replay_mode,
};
use crate::providers::anthropic_refusal::{
    apply_anthropic_refusal, AnthropicRefusalOutput,
};
use crate::providers::anthropic_server_fallback::{
    apply_anthropic_fallback_boundary, build_anthropic_server_side_fallbacks,
    read_anthropic_fallback_boundary, AnthropicFallbackBoundary, AnthropicFallbackOutput,
};
use crate::providers::anthropic_thinking_replay::find_active_anthropic_tool_turn_assistant_index;
use crate::providers::anthropic_usage::{
    parse_anthropic_usage_payload, read_anthropic_prompt_usage_snapshot,
    AnthropicPromptUsageSnapshot,
};

/// Streams a full Anthropic Messages request.
pub fn stream_anthropic(
    _model: Model,
    _context: Context,
    _options: Option<StreamOptions>,
) -> Box<dyn AssistantMessageEventStreamContract> {
    // Streaming implementation wires request signing, fallback, refusal,
    // usage, and thinking replay. The supporting helpers are re-exported
    // below; full request construction lives in a follow-up translation.
    unimplemented!("anthropic stream adapter: full implementation pending")
}

/// Streams a simple Anthropic Messages request.
pub fn stream_simple_anthropic(
    _model: Model,
    _context: Context,
    _options: Option<SimpleStreamOptions>,
) -> Box<dyn AssistantMessageEventStreamContract> {
    unimplemented!("anthropic simple-stream adapter: full implementation pending")
}

/// Helper re-export: future-based complete helper (mirrors `complete()`).
pub fn complete_anthropic(
    model: Model,
    context: Context,
    options: Option<StreamOptions>,
) -> Pin<Box<dyn Future<Output = AssistantMessage> + Send>> {
    let stream = stream_anthropic(model, context, options);
    Box::pin(async move { stream.result().await })
}

/// Helper re-export: future-based simple-complete helper.
pub fn complete_simple_anthropic(
    model: Model,
    context: Context,
    options: Option<SimpleStreamOptions>,
) -> Pin<Box<dyn Future<Output = AssistantMessage> + Send>> {
    let stream = stream_simple_anthropic(model, context, options);
    Box::pin(async move { stream.result().await })
}

// ---- Re-exports for downstream consumers ----

pub use crate::providers::anthropic_auth_headers as auth_headers;
pub use crate::providers::anthropic_model_contract as model_contract;
pub use crate::providers::anthropic_refusal as refusal;
pub use crate::providers::anthropic_server_fallback as server_fallback;
pub use crate::providers::anthropic_thinking_replay as thinking_replay;
pub use crate::providers::anthropic_usage as usage;
pub use crate::providers::anthropic_tool_projection as tool_projection;

// Type re-exports for callers.
pub use crate::providers::anthropic_tool_projection::{
    AnthropicInputSchema, AnthropicProjectedTool, AnthropicProjectedToolChoice,
    AnthropicToolProjection,
};