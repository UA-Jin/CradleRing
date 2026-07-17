// Agent Core module implements runtime deps behavior.
// 翻译自 packages/agent-core/src/runtime-deps.ts

use llm_core::types::{CompleteSimpleFn, StreamFn};

/// Runtime functions injected by host packages so agent-core stays provider-agnostic.
pub trait AgentCoreRuntimeDeps: Send + Sync {
    /// Streaming completion implementation used for normal agent turns.
    fn stream_simple(&self) -> StreamFn;
    /// Non-streaming completion implementation used by summarization helpers.
    fn complete_simple(&self) -> CompleteSimpleFn;
}

/// Runtime dependency subset required by streaming agent loops.
pub type AgentCoreStreamRuntimeDeps = Box<dyn StreamSimpleOnly>;

/// Runtime dependency subset required by summarization helpers.
pub type AgentCoreCompletionRuntimeDeps = Box<dyn CompleteSimpleOnly>;

pub trait StreamSimpleOnly: Send + Sync {
    fn stream_simple(&self) -> StreamFn;
}

pub trait CompleteSimpleOnly: Send + Sync {
    fn complete_simple(&self) -> CompleteSimpleFn;
}

fn missing_runtime_dep(name: &str) -> String {
    format!(
        "@cradle-ring/agent-core runtime dependency \"{}\" is not configured. Pass an AgentCoreRuntimeDeps instance or a streamFn explicitly.",
        name
    )
}

/// Resolve the stream function, preferring an explicit override over injected runtime deps.
pub fn resolve_agent_core_stream_fn(
    runtime: Option<&dyn StreamSimpleOnly>,
    stream_fn: Option<StreamFn>,
) -> StreamFn {
    if let Some(s) = stream_fn {
        return s;
    }
    if let Some(r) = runtime {
        return r.stream_simple();
    }
    panic!("{}", missing_runtime_dep("streamSimple"));
}

/// Resolve the completion function used by non-streaming helper flows.
pub fn resolve_agent_core_complete_fn(
    runtime: Option<&dyn CompleteSimpleOnly>,
) -> CompleteSimpleFn {
    if let Some(r) = runtime {
        return r.complete_simple();
    }
    panic!("{}", missing_runtime_dep("completeSimple"));
}