// Embeddings remote provider helper.
// 翻译自 packages/memory-host-sdk/src/host/embeddings-remote-provider.ts

pub struct EmbeddingsRemoteProvider {
    pub model: String,
}

impl EmbeddingsRemoteProvider {
    pub fn new(model: &str) -> Self {
        Self { model: model.to_string() }
    }
}