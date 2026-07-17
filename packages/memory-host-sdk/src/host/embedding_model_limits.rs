// Embedding model limits.
// 翻译自 packages/memory-host-sdk/src/host/embedding-model-limits.ts

pub const MAX_EMBEDDING_INPUT_CHARS: i64 = 8_000;

pub fn max_input_chars(_model: &str) -> i64 {
    MAX_EMBEDDING_INPUT_CHARS
}