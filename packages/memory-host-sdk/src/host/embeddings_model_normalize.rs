// Embeddings model normalization.
// 翻译自 packages/memory-host-sdk/src/host/embeddings-model-normalize.ts

pub fn normalize_embeddings_model(name: &str) -> String {
    name.trim().to_lowercase()
}