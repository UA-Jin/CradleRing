//! Built-in provider registration.
//! 翻译自 packages/ai/src/providers/register-builtins.ts

use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use llm_core::types::{
    Api, AssistantMessage, AssistantMessageEventStreamContract, Context, Model, SimpleStreamOptions,
    StreamOptions,
};

use crate::api_registry::{ApiProvider, ApiRegistry};
use crate::utils::event_stream::AssistantMessageEventStream;

/// Source id used for built-in API provider registrations.
pub const BUILT_IN_API_PROVIDER_SOURCE_ID: &str = "core:built-in";

pub type StreamFunction = Arc<
    dyn Fn(Model, Context, Option<StreamOptions>) -> Box<dyn AssistantMessageEventStreamContract>
        + Send
        + Sync,
>;
pub type SimpleStreamFunction = Arc<
    dyn Fn(
        Model,
        Context,
        Option<SimpleStreamOptions>,
    ) -> Box<dyn AssistantMessageEventStreamContract>
        + Send
        + Sync,
>;

/// One provider module's stream pair.
pub struct ProviderStreams {
    pub stream: StreamFunction,
    pub stream_simple: SimpleStreamFunction,
}

pub type ProviderModuleFuture =
    Pin<Box<dyn Future<Output = Result<ProviderStreams, String>> + Send>>;
pub type LoadProviderModule = Box<dyn Fn() -> ProviderModuleFuture + Send + Sync>;

/// Register all built-in API providers against `registry`.
pub fn register_built_in_api_providers(registry: &ApiRegistry) {
    let loaders: Vec<(&str, LoadProviderModule)> = vec![
        (
            "anthropic-messages",
            Box::new(|| Box::pin(async { Err("anthropic provider not loaded".into()) })),
        ),
        (
            "openai-completions",
            Box::new(|| Box::pin(async { Err("openai-completions provider not loaded".into()) })),
        ),
        (
            "mistral-conversations",
            Box::new(|| Box::pin(async { Err("mistral provider not loaded".into()) })),
        ),
        (
            "openai-responses",
            Box::new(|| Box::pin(async { Err("openai-responses provider not loaded".into()) })),
        ),
        (
            "azure-openai-responses",
            Box::new(|| Box::pin(async { Err("azure-openai-responses provider not loaded".into()) })),
        ),
        (
            "openai-chatgpt-responses",
            Box::new(|| Box::pin(async { Err("openai-chatgpt-responses provider not loaded".into()) })),
        ),
        (
            "google-generative-ai",
            Box::new(|| Box::pin(async { Err("google provider not loaded".into()) })),
        ),
        (
            "google-vertex",
            Box::new(|| Box::pin(async { Err("google-vertex provider not loaded".into()) })),
        ),
    ];

    for (api, loader) in loaders {
        register_lazy(registry, api.to_string(), loader);
    }
}

fn register_lazy(registry: &ApiRegistry, api: Api, loader: LoadProviderModule) {
    let api_for_stream = api.clone();
    let api_for_simple = api.clone();
    let loader_arc: Arc<LoadProviderModule> = Arc::new(loader);

    let stream: StreamFunction = {
        let loader_arc = loader_arc.clone();
        let api_for_stream = api_for_stream.clone();
        Arc::new(move |model, context, options| {
            let outer = AssistantMessageEventStream::new();
            let _loader = loader_arc.clone();
            let _ = (api_for_stream.clone(), model, context, options, outer);
            Box::new(AssistantMessageEventStream::new())
        })
    };
    let stream_simple: SimpleStreamFunction = {
        let loader_arc = loader_arc.clone();
        let api_for_simple = api_for_simple.clone();
        Arc::new(move |model, context, options| {
            let outer = AssistantMessageEventStream::new();
            let _loader = loader_arc.clone();
            let _ = (api_for_simple.clone(), model, context, options, outer);
            Box::new(AssistantMessageEventStream::new())
        })
    };

    registry.register_api_provider(
        ApiProvider::<StreamOptions> {
            api,
            stream,
            stream_simple,
        },
        Some(BUILT_IN_API_PROVIDER_SOURCE_ID),
    );
}

/// Restore the built-in provider registry state (used by tests).
pub fn reset_api_providers(registry: &ApiRegistry) {
    registry.unregister_api_providers(BUILT_IN_API_PROVIDER_SOURCE_ID);
    register_built_in_api_providers(registry);
}

/// Construct a standardized assistant-message error for a lazy-load failure.
pub fn create_lazy_load_error_message(model: &Model, error: &str) -> AssistantMessage {
    AssistantMessage {
        role: "assistant".to_string(),
        content: vec![],
        api: model.api.clone(),
        provider: model.provider.clone(),
        model: model.id.clone(),
        response_model: None,
        response_id: None,
        diagnostics: None,
        usage: llm_core::types::Usage {
            input: 0,
            output: 0,
            cache_read: 0,
            cache_write: 0,
            context_usage: None,
            total_tokens: 0,
            cost: llm_core::types::UsageCost {
                input: 0.0,
                output: 0.0,
                cache_read: 0.0,
                cache_write: 0.0,
                total: 0.0,
                total_origin: None,
            },
        },
        stop_reason: "error".to_string(),
        error_message: Some(error.to_string()),
        error_code: None,
        error_type: None,
        error_body: None,
        timestamp: chrono::Utc::now().timestamp_millis(),
    }
}