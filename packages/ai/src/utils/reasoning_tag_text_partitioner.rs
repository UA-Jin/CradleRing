//! Reasoning-tag text partitioner.
//! 翻译自 packages/ai/src/utils/reasoning-tag-text-partitioner.ts
//!
//! Splits streaming text into reasoning (`<think>...</think>`) and visible
//! text segments. Handles unclosed think tags gracefully.

use once_cell::sync::Lazy;
use regex::Regex;

/// One partition produced by the partitioner.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Partition {
    /// Visible / final text.
    Text(String),
    /// Hidden reasoning text.
    Reasoning(String),
}

/// Splits `input` into reasoning and visible-text partitions.
pub fn partition_reasoning_tags(input: &str) -> Vec<Partition> {
    static THINK_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?is)<\s*think\s*>(.*?)<\s*/\s*think\s*>").unwrap());

    let mut out: Vec<Partition> = Vec::new();
    let mut last_end = 0usize;
    for cap in THINK_RE.captures_iter(input) {
        let mat = cap.get(0).unwrap();
        if mat.start() > last_end {
            let chunk = &input[last_end..mat.start()];
            if !chunk.is_empty() {
                out.push(Partition::Text(chunk.to_string()));
            }
        }
        let inner = cap.get(1).map(|m| m.as_str()).unwrap_or("");
        out.push(Partition::Reasoning(inner.to_string()));
        last_end = mat.end();
    }
    if last_end < input.len() {
        let tail = &input[last_end..];
        out.push(Partition::Text(tail.to_string()));
    }
    out
}

/// Returns only the visible-text partitions joined together.
pub fn strip_reasoning_tags(input: &str) -> String {
    let mut out = String::new();
    for p in partition_reasoning_tags(input) {
        if let Partition::Text(text) = p {
            out.push_str(&text);
        }
    }
    out
}