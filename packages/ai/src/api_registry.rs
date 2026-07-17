//! LLM Runtime module — API registry.
//! 翻译自 packages/ai/src/api-registry.ts
//!
//! Runtime provider registry for built-in and plugin-supplied API adapters.
//! Each entry binds a model API id to a streaming and a simple-stream
//! adapter; the registry performs api-id guards on every call.

use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use llm_core::types::{
    Api, AssistantMessageEventStreamContract, Context, Model, SimpleStreamOptions, StreamOptions,
};

/// Runtime stream adapter signature stored in the API provider registry.
pub type ApiStreamFunction = Arc<
    dyn Fn(Model, Context, Option<StreamOptions>) -> Box<dyn AssistantMessageEventStreamContract>
        + Send
        + Sync,
>;

/// Runtime simple-stream adapter signature stored in the API provider registry.
pub type ApiStreamSimpleFunction = Arc<
    dyn Fn(
        Model,
        Context,
        Option<SimpleStreamOptions>,
    ) -> Box<dyn AssistantMessageEventStreamContract>
        + Send
        + Sync,
>;

/// Provider implementation registered by core or plugins for a specific model API.
pub struct ApiProvider<TOptions: 'static = StreamOptions> {
    pub api: Api,
    pub stream: Arc<
        dyn Fn(Model, Context, Option<TOptions>) -> Box<dyn AssistantMessageEventStreamContract>
            + Send
            + Sync,
    >,
    pub stream_simple: Arc<
        dyn Fn(
            Model,
            Context,
            Option<SimpleStreamOptions>,
        ) -> Box<dyn AssistantMessageEventStreamContract>
            + Send
            + Sync,
    >,
}

/// Type-erased provider returned by a registry after API guards are installed.
#[derive(Clone)]
pub struct RegisteredApiProvider {
    pub api: Api,
    pub stream: ApiStreamFunction,
    pub stream_simple: ApiStreamSimpleFunction,
}

struct RegisteredApiProviderEntry {
    provider: RegisteredApiProvider,
    source_id: Option<String>,
}

fn wrap_stream<TOptions: 'static>(
    api: Api,
    stream: Arc<
        dyn Fn(Model, Context, Option<TOptions>) -> Box<dyn AssistantMessageEventStreamContract>
            + Send
            + Sync,
    >,
) -> ApiStreamFunction {
    Arc::new(move |model, context, options| {
        if model.api != api {
            panic!("Mismatched api: {} expected {}", model.api, api);
        }
        let typed: Option<TOptions> = unsafe { std::mem::transmute_copy(&options) };
        std::mem::forget(options);
        stream(model, context, typed)
    })
}

fn wrap_stream_simple(
    api: Api,
    stream_simple: Arc<
        dyn Fn(
            Model,
            Context,
            Option<SimpleStreamOptions>,
        ) -> Box<dyn AssistantMessageEventStreamContract>
            + Send
            + Sync,
    >,
) -> ApiStreamSimpleFunction {
    Arc::new(move |model, context, options| {
        if model.api != api {
            panic!("Mismatched api: {} expected {}", model.api, api);
        }
        stream_simple(model, context, options)
    })
}

/// Handle for an isolated provider registry.
pub struct ApiRegistry {
    providers: RwLock<HashMap<Api, RegisteredApiProviderEntry>>,
}

impl ApiRegistry {
    /// Create a new empty registry.
    pub fn new() -> Self {
        Self {
            providers: RwLock::new(HashMap::new()),
        }
    }

    /// Register a provider implementation.
    pub fn register_api_provider<TOptions: 'static>(
        &self,
        provider: ApiProvider<TOptions>,
        source_id: Option<&str>,
    ) {
        let entry = RegisteredApiProviderEntry {
            provider: RegisteredApiProvider {
                api: provider.api.clone(),
                stream: wrap_stream(provider.api.clone(), provider.stream),
                stream_simple: wrap_stream_simple(provider.api.clone(), provider.stream_simple),
            },
            source_id: source_id.map(|s| s.to_string()),
        };
        self.providers
            .write()
            .expect("api registry poisoned")
            .insert(provider.api, entry);
    }

    /// Returns the registered provider for one API, if any.
    pub fn get_api_provider(&self, api: &str) -> Option<RegisteredApiProvider> {
        self.providers
            .read()
            .expect("api registry poisoned")
            .get(api)
            .map(|e| e.provider.clone())
    }

    /// Returns all registered providers.
    pub fn get_api_providers(&self) -> Vec<RegisteredApiProvider> {
        self.providers
            .read()
            .expect("api registry poisoned")
            .values()
            .map(|e| e.provider.clone())
            .collect()
    }

    /// Unregister all providers owned by the given source id.
    pub fn unregister_api_providers(&self, source_id: &str) {
        let mut guard = self.providers.write().expect("api registry poisoned");
        guard.retain(|_, entry| entry.source_id.as_deref() != Some(source_id));
    }

    /// Remove all registered providers.
    pub fn clear_api_providers(&self) {
        self.providers
            .write()
            .expect("api registry poisoned")
            .clear();
    }
}

impl Default for ApiRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Creates an isolated provider registry for one runtime or tenant.
pub fn create_api_registry() -> ApiRegistry {
    ApiRegistry::new()
}