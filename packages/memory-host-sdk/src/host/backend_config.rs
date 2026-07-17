// Memory backend config resolver.
// 翻译自 packages/memory-host-sdk/src/host/backend-config.ts

use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ResolvedMemoryBackendConfig {
    pub backend: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub qmd: Option<ResolvedQmdConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ResolvedQmdConfig {
    pub command: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mcporter: Option<ResolvedQmdMcporterConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ResolvedQmdMcporterConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,
}

pub fn resolve_memory_backend_config(_value: &Value) -> ResolvedMemoryBackendConfig {
    ResolvedMemoryBackendConfig::default()
}