//! Google Vertex AI provider adapter.
//! 翻译自 packages/ai/src/providers/google-vertex.ts

use std::future::Future;
use std::pin::Pin;

use llm_core::types::{
    AssistantMessage, AssistantMessageEventStreamContract, Context, Model, SimpleStreamOptions,
    StreamOptions,
};

use crate::providers::google_shared as shared;

/// Streams a Google Vertex request.
pub fn stream_google_vertex(
    _model: Model,
    _context: Context,
    _options: Option<StreamOptions>,
) -> Box<dyn AssistantMessageEventStreamContract> {
    unimplemented!("google-vertex stream adapter: full implementation pending")
}

/// Streams a simple Google Vertex request.
pub fn stream_simple_google_vertex(
    _model: Model,
    _context: Context,
    _options: Option<SimpleStreamOptions>,
) -> Box<dyn AssistantMessageEventStreamContract> {
    unimplemented!("google-vertex simple-stream adapter: full implementation pending")
}

/// Future-based complete helper.
pub fn complete_google_vertex(
    model: Model,
    context: Context,
    options: Option<StreamOptions>,
) -> Pin<Box<dyn Future<Output = AssistantMessage> + Send>> {
    let stream = stream_google_vertex(model, context, options);
    Box::pin(async move { stream.result().await })
}

/// Future-based simple-complete helper.
pub fn complete_simple_google_vertex(
    model: Model,
    context: Context,
    options: Option<SimpleStreamOptions>,
) -> Pin<Box<dyn Future<Output = AssistantMessage> + Send>> {
    let stream = stream_simple_google_vertex(model, context, options);
    Box::pin(async move { stream.result().await })
}

pub use shared::*;