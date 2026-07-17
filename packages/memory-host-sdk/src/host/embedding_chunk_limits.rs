// Embedding chunk limits helper.
// 翻译自 packages/memory-host-sdk/src/host/embedding-chunk-limits.ts

pub const MAX_CHUNK_CHARS: i64 = 2_000;

pub fn chunk_text_by_limit(text: &str) -> Vec<String> {
    let max_chars = MAX_CHUNK_CHARS as usize;
    text.chars()
        .collect::<Vec<_>>()
        .chunks(max_chars)
        .map(|c| c.iter().collect::<String>())
        .collect()
}