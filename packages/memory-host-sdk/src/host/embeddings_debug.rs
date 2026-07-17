// Embeddings debug helper.
// 翻译自 packages/memory-host-sdk/src/host/embeddings-debug.ts

#[derive(Debug, Clone, Default)]
pub struct EmbeddingsDebug {
    pub requests: i64,
    pub cache_hits: i64,
}