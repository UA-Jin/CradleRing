// Embeddings types.
// 翻译自 packages/memory-host-sdk/src/host/embeddings.types.ts

pub type EmbeddingVector = Vec<f32>;

#[derive(Debug, Clone, Default)]
pub struct EmbeddingsConfig {
    pub model: String,
    pub dims: i64,
}