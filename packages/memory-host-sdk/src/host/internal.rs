// Memory host internal helpers.
// 翻译自 packages/memory-host-sdk/src/host/internal.ts

use std::future::Future;

use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryChunk {
    pub path: String,
    pub start_line: i64,
    pub end_line: i64,
    pub text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryFileEntry {
    pub path: String,
    pub size: i64,
    pub modified_at: i64,
}

pub fn build_file_entry(path: &str, size: i64, modified_at: i64) -> MemoryFileEntry {
    MemoryFileEntry {
        path: path.to_string(),
        size,
        modified_at,
    }
}

pub fn build_multimodal_chunk_for_indexing(text: &str) -> MemoryChunk {
    MemoryChunk {
        path: String::new(),
        start_line: 0,
        end_line: text.lines().count() as i64,
        text: text.to_string(),
    }
}

pub fn chunk_markdown(text: &str, max_chars: usize) -> Vec<MemoryChunk> {
    let mut chunks = vec![];
    let mut start = 0;
    let chars: Vec<char> = text.chars().collect();
    while start < chars.len() {
        let end = (start + max_chars).min(chars.len());
        let chunk_text: String = chars[start..end].iter().collect();
        chunks.push(MemoryChunk {
            path: String::new(),
            start_line: 0,
            end_line: chunk_text.lines().count() as i64,
            text: chunk_text,
        });
        start = end;
    }
    chunks
}

pub fn cosine_similarity(a: &[f32], b: &[f32]) -> f64 {
    if a.len() != b.len() {
        return 0.0;
    }
    let mut dot = 0.0;
    let mut na = 0.0;
    let mut nb = 0.0;
    for (x, y) in a.iter().zip(b.iter()) {
        dot += (*x as f64) * (*y as f64);
        na += (*x as f64).powi(2);
        nb += (*y as f64).powi(2);
    }
    if na == 0.0 || nb == 0.0 {
        0.0
    } else {
        dot / (na.sqrt() * nb.sqrt())
    }
}

pub fn ensure_dir(path: &str) -> std::io::Result<()> {
    std::fs::create_dir_all(path)
}

pub fn hash_text(text: &str) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut hasher = DefaultHasher::new();
    text.hash(&mut hasher);
    format!("{:x}", hasher.finish())
}

pub fn list_memory_files(_dir: &str) -> Vec<MemoryFileEntry> {
    vec![]
}

pub fn normalize_extra_memory_paths(paths: Vec<String>) -> Vec<String> {
    paths.into_iter().filter(|p| !p.is_empty()).collect()
}

pub fn parse_embedding(value: &Value) -> Option<Vec<f32>> {
    value
        .as_array()
        .map(|arr| arr.iter().filter_map(|v| v.as_f64().map(|f| f as f32)).collect())
}

pub fn remap_chunk_lines(chunks: Vec<MemoryChunk>) -> Vec<MemoryChunk> {
    chunks
}

pub async fn run_with_concurrency<F>(items: Vec<Value>, limit: usize, op: F) -> Vec<Value>
where
    F: Fn(Value) -> std::pin::Pin<Box<dyn Future<Output = Value> + Send>> + Send + Sync,
{
    let _ = (items, limit, op);
    vec![]
}