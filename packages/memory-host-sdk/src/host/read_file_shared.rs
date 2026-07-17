// Read file shared helpers.
// 翻译自 packages/memory-host-sdk/src/host/read-file-shared.ts

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryReadResult {
    pub text: String,
    pub path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub truncated: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lines: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_from: Option<i64>,
}

pub const DEFAULT_MEMORY_READ_LINES: i64 = 200;
pub const DEFAULT_MEMORY_READ_MAX_CHARS: i64 = 200_000;

pub fn build_memory_read_result(text: String, path: &str, from: Option<i64>, lines: Option<i64>) -> MemoryReadResult {
    MemoryReadResult {
        text,
        path: path.to_string(),
        truncated: None,
        from,
        lines,
        next_from: None,
    }
}

pub fn build_memory_read_result_from_slice(text: String, path: &str, from: i64, lines: i64) -> MemoryReadResult {
    let total_lines = text.lines().count() as i64;
    let truncated = from + lines < total_lines;
    MemoryReadResult {
        text,
        path: path.to_string(),
        truncated: if truncated { Some(true) } else { None },
        from: Some(from),
        lines: Some(lines),
        next_from: if truncated { Some(from + lines) } else { None },
    }
}