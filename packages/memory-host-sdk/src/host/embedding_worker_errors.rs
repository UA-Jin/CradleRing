// Embedding worker errors.
// 翻译自 packages/memory-host-sdk/src/host/embedding-worker-errors.ts

#[derive(Debug, Clone)]
pub struct EmbeddingWorkerError {
    pub message: String,
}

impl std::fmt::Display for EmbeddingWorkerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for EmbeddingWorkerError {}