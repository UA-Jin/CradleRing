//! LLM Runtime module — runtime entry point.
//! 翻译自 packages/ai/src/stream.ts
//!
//! `create_llm_runtime` builds an isolated runtime backed by a provider
//! registry. It exposes `stream` / `complete` (full) and `stream_simple` /
//! `complete_simple` (simple) helpers.

use std::future::Future;
use std::pin::Pin;

use llm_core::types::{
    AssistantMessage, AssistantMessageEventStreamContract, Context, Model, ProviderStreamOptions,
    SimpleStreamOptions, StreamOptions,
};

use crate::api_registry::{create_api_registry, ApiRegistry, RegisteredApiProvider};

/// One LLM runtime bound to a registry.
pub struct LlmRuntime {
    pub registry: ApiRegistry,
}

impl LlmRuntime {
    fn resolve_api_provider(&self, api: &str) -> RegisteredApiProvider {
        self.registry
            .get_api_provider(api)
            .unwrap_or_else(|| panic!("No API provider registered for api: {}", api))
    }

    /// Full streaming adapter.
    pub fn stream(
        &self,
        model: Model,
        context: Context,
        options: Option<ProviderStreamOptions>,
    ) -> Box<dyn AssistantMessageEventStreamContract> {
        let api = model.api.clone();
        let provider = self.resolve_api_provider(&api);
        let opts: Option<StreamOptions> = options.map(|m| StreamOptions {
            temperature: None,
            max_tokens: None,
            stop: None,
            transport: None,
            cache_retention: None,
            session_id: None,
            request_id: None,
            prompt_cache_key: None,
            headers: None,
            timeout_ms: None,
            max_retries: None,
            max_retry_delay_ms: None,
            metadata: Some(m),
        });
        (provider.stream)(model, context, opts)
    }

    /// Full non-streaming helper that awaits the streaming contract's result.
    pub fn complete(
        &self,
        model: Model,
        context: Context,
        options: Option<ProviderStreamOptions>,
    ) -> Pin<Box<dyn Future<Output = AssistantMessage> + Send>> {
        let stream = self.stream(model, context, options);
        Box::pin(async move { stream.result().await })
    }

    /// Simple streaming adapter.
    pub fn stream_simple(
        &self,
        model: Model,
        context: Context,
        options: Option<SimpleStreamOptions>,
    ) -> Box<dyn AssistantMessageEventStreamContract> {
        let api = model.api.clone();
        let provider = self.resolve_api_provider(&api);
        (provider.stream_simple)(model, context, options)
    }

    /// Simple non-streaming helper.
    pub fn complete_simple(
        &self,
        model: Model,
        context: Context,
        options: Option<SimpleStreamOptions>,
    ) -> Pin<Box<dyn Future<Output = AssistantMessage> + Send>> {
        let stream = self.stream_simple(model, context, options);
        Box::pin(async move { stream.result().await })
    }
}

/// Creates an isolated LLM runtime backed by the supplied provider registry.
pub fn create_llm_runtime(registry: Option<ApiRegistry>) -> LlmRuntime {
    LlmRuntime {
        registry: registry.unwrap_or_else(create_api_registry),
    }
}