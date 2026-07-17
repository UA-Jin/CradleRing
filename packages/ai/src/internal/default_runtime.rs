//! Process-default registry/runtime retained for the OpenClaw compatibility facade.
//! 翻译自 packages/ai/src/internal/default-runtime.ts
//!
//! Deliberately not part of the public package API: external consumers
//! create isolated runtimes via `create_llm_runtime`; exporting these
//! from the root barrel would reintroduce the mutable process-global
//! registry.

use std::sync::OnceLock;

use crate::api_registry::{create_api_registry, ApiRegistry};
use crate::stream::{create_llm_runtime, LlmRuntime};

struct DefaultRuntimeState {
    registry: ApiRegistry,
    runtime: LlmRuntime,
}

static DEFAULT_RUNTIME: OnceLock<DefaultRuntimeState> = OnceLock::new();

fn resolve_default_runtime() -> &'static DefaultRuntimeState {
    DEFAULT_RUNTIME.get_or_init(|| {
        let registry = create_api_registry();
        let runtime = create_llm_runtime(None);
        DefaultRuntimeState { registry, runtime }
    })
}

/// The process-default API registry.
pub fn default_api_registry() -> &'static ApiRegistry {
    &resolve_default_runtime().registry
}

/// The process-default LLM runtime.
pub fn default_llm_runtime() -> &'static LlmRuntime {
    &resolve_default_runtime().runtime
}

/// Register an API provider against the default registry.
pub fn register_api_provider<TOptions>(
    provider: crate::api_registry::ApiProvider<TOptions>,
    source_id: Option<&str>,
) where
    TOptions: 'static,
{
    default_api_registry().register_api_provider(provider, source_id);
}

/// Get one API provider from the default registry.
pub fn get_api_provider(api: &str) -> Option<crate::api_registry::RegisteredApiProvider> {
    default_api_registry().get_api_provider(api)
}

/// Returns all API providers registered against the default registry.
pub fn get_api_providers() -> Vec<crate::api_registry::RegisteredApiProvider> {
    default_api_registry().get_api_providers()
}

/// Unregister all API providers owned by `source_id`.
pub fn unregister_api_providers(source_id: &str) {
    default_api_registry().unregister_api_providers(source_id);
}

/// Clear the default registry.
pub fn clear_api_providers() {
    default_api_registry().clear_api_providers();
}