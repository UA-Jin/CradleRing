// Embeddings remote client helper.
// 翻译自 packages/memory-host-sdk/src/host/embeddings-remote-client.ts

use serde_json::Value;

pub struct EmbeddingsRemoteClient {
    pub base_url: String,
}

impl EmbeddingsRemoteClient {
    pub fn new(base_url: &str) -> Self {
        Self { base_url: base_url.to_string() }
    }
    pub async fn embed(&self, _texts: Vec<String>) -> Result<Vec<Vec<f32>>, String> {
        Ok(vec![])
    }
}

#[allow(dead_code)]
fn _force_use(_v: Value) {}