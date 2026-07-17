// Embedding defaults.
// 翻译自 packages/memory-host-sdk/src/host/embedding-defaults.ts

pub const DEFAULT_EMBEDDING_MODEL: &str = "text-embedding-3-small";
pub const DEFAULT_EMBEDDING_DIMS: i64 = 1536;

pub fn default_embedding_model() -> String {
    DEFAULT_EMBEDDING_MODEL.to_string()
}