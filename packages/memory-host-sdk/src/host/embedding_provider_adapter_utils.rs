// Embedding provider adapter utils.
// 翻译自 packages/memory-host-sdk/src/host/embedding-provider-adapter-utils.ts

pub fn normalize_provider_name(name: &str) -> String {
    name.trim().to_lowercase()
}