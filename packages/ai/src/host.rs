//! Host policy ports for the reusable transport package.
//! 翻译自 packages/ai/src/host.ts
//!
//! Fetch guarding, secret redaction, strict-tool policy, and diagnostics
//! logging are owned by the embedding application. The library defaults
//! below are inert so external consumers get safe, dependency-free
//! behavior without wiring anything.

use std::collections::BTreeMap;
use std::sync::{Arc, RwLock};

use llm_core::types::Model;

/// Strict-tool policy inputs for OpenAI-compatible routes.
#[derive(Debug, Clone, Default)]
pub struct OpenAIStrictToolSettingOptions {
    pub transport: Option<String>,
    pub supports_strict_mode: Option<bool>,
}

/// Narrow host ports consumed by the built-in provider adapters.
pub trait AiTransportHost: Send + Sync {
    /// Builds a policy-guarded fetch for one model request.
    /// Returning `None` keeps the provider SDK's default fetch.
    fn build_model_fetch(
        &self,
        model: Model,
        timeout_ms: Option<u64>,
        options: Option<ModelFetchOptions>,
    ) -> Option<Box<dyn Fn() -> Option<()>>>;

    /// Resolves host-owned process-local secret sentinel substrings immediately before egress.
    fn resolve_secret_sentinel(&self, value: &str) -> String;

    /// Redacts secrets inside structured tool-result payloads.
    fn redact_secrets(&self, value: serde_json::Value) -> serde_json::Value;

    /// Redacts secret-bearing text in tool payload strings.
    fn redact_tool_payload_text(&self, text: &str) -> String;

    /// Resolves the host strict-tool default for OpenAI-compatible routes.
    /// `None` lets the request omit the strict flag entirely.
    fn resolve_openai_strict_tool_setting(
        &self,
        model: &Model,
        options: Option<&OpenAIStrictToolSettingOptions>,
    ) -> Option<bool>;

    /// Emits one transport diagnostic; build runs only when the host logs it and
    /// may return null to suppress the entry (e.g. de-duplication).
    fn log_debug(
        &self,
        subsystem: &str,
        build: Box<dyn Fn() -> Option<LogDebugEntry> + Send + Sync>,
    );
}

/// Options passed to `build_model_fetch`.
#[derive(Debug, Clone, Default)]
pub struct ModelFetchOptions {
    pub sanitize_sse: Option<bool>,
}

/// One transport diagnostic log entry.
#[derive(Debug, Clone, Default)]
pub struct LogDebugEntry {
    pub message: String,
    pub data: Option<BTreeMap<String, serde_json::Value>>,
}

/// Inert default `AiTransportHost` implementation. Used when no host is configured.
#[derive(Debug, Default, Clone)]
pub struct InertAiTransportHost;

impl AiTransportHost for InertAiTransportHost {
    fn build_model_fetch(
        &self,
        _model: Model,
        _timeout_ms: Option<u64>,
        _options: Option<ModelFetchOptions>,
    ) -> Option<Box<dyn Fn() -> Option<()>>> {
        None
    }

    fn resolve_secret_sentinel(&self, value: &str) -> String {
        value.to_string()
    }

    fn redact_secrets(&self, value: serde_json::Value) -> serde_json::Value {
        value
    }

    fn redact_tool_payload_text(&self, text: &str) -> String {
        text.to_string()
    }

    fn resolve_openai_strict_tool_setting(
        &self,
        _model: &Model,
        options: Option<&OpenAIStrictToolSettingOptions>,
    ) -> Option<bool> {
        options.and_then(|o| o.supports_strict_mode.map(|_| false))
    }

    fn log_debug(
        &self,
        _subsystem: &str,
        _build: Box<dyn Fn() -> Option<LogDebugEntry> + Send + Sync>,
    ) {
    }
}

type SharedHost = Arc<dyn AiTransportHost>;

static ACTIVE_HOST: once_cell::sync::Lazy<RwLock<SharedHost>> = once_cell::sync::Lazy::new(|| {
    RwLock::new(Arc::new(InertAiTransportHost))
});

/// Installs host implementations for the transport policy ports.
pub fn configure_ai_transport_host(host: impl AiTransportHost + 'static) {
    let mut guard = ACTIVE_HOST.write().expect("ai transport host poisoned");
    *guard = Arc::new(host);
}

/// Installs host implementations from a partial trait-object-friendly closure bundle.
pub fn configure_ai_transport_host_arc(host: SharedHost) {
    let mut guard = ACTIVE_HOST.write().expect("ai transport host poisoned");
    *guard = host;
}

/// Returns the active transport host (inert defaults unless configured).
pub fn get_ai_transport_host() -> SharedHost {
    ACTIVE_HOST
        .read()
        .expect("ai transport host poisoned")
        .clone()
}

/// Resolves sentinel substrings in custom headers at a no-fetch adapter boundary.
pub fn resolve_ai_transport_header_sentinels(
    headers: Option<&BTreeMap<String, String>>,
) -> Option<BTreeMap<String, String>> {
    let headers = headers?;
    let host = get_ai_transport_host();
    let mut resolved: Option<BTreeMap<String, String>> = None;
    for (name, value) in headers {
        let resolved_value = host.resolve_secret_sentinel(value);
        if &resolved_value != value {
            let map = resolved.get_or_insert_with(|| headers.clone());
            map.insert(name.clone(), resolved_value);
        }
    }
    Some(resolved.unwrap_or_else(|| headers.clone()))
}