//! Mistral provider adapter.
//! 翻译自 packages/ai/src/providers/mistral.ts

use std::future::Future;
use std::pin::Pin;

use llm_core::types::{
    AssistantMessage, AssistantMessageEventStreamContract, Context, Model, SimpleStreamOptions,
    StreamOptions,
};

/// Streams a Mistral Conversations request.
pub fn stream_mistral(
    _model: Model,
    _context: Context,
    _options: Option<StreamOptions>,
) -> Box<dyn AssistantMessageEventStreamContract> {
    unimplemented!("mistral stream adapter: full implementation pending")
}

/// Streams a simple Mistral Conversations request.
pub fn stream_simple_mistral(
    _model: Model,
    _context: Context,
    _options: Option<SimpleStreamOptions>,
) -> Box<dyn AssistantMessageEventStreamContract> {
    unimplemented!("mistral simple-stream adapter: full implementation pending")
}

/// Future-based complete helper.
pub fn complete_mistral(
    model: Model,
    context: Context,
    options: Option<StreamOptions>,
) -> Pin<Box<dyn Future<Output = AssistantMessage> + Send>> {
    let stream = stream_mistral(model, context, options);
    Box::pin(async move { stream.result().await })
}

/// Future-based simple-complete helper.
pub fn complete_simple_mistral(
    model: Model,
    context: Context,
    options: Option<SimpleStreamOptions>,
) -> Pin<Box<dyn Future<Output = AssistantMessage> + Send>> {
    let stream = stream_simple_mistral(model, context, options);
    Box::pin(async move { stream.result().await })
}