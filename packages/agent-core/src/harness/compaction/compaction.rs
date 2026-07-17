// Harness compaction helpers.
// 翻译自 packages/agent-core/src/harness/compaction/compaction.ts

use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompactionSettings {
    pub max_context_tokens: i64,
    pub keep_recent_messages: i64,
}

pub const DEFAULT_COMPACTION_SETTINGS: CompactionSettings = CompactionSettings {
    max_context_tokens: 100_000,
    keep_recent_messages: 20,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextUsageEstimate {
    pub tokens: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompactionPreparation {
    pub summary: String,
    pub tokens_before: i64,
    pub tokens_after: Option<i64>,
    pub first_kept_entry_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompactionDetails {
    pub reason: String,
    pub tokens_before: i64,
    pub tokens_after: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompactionResult {
    pub summary: String,
    pub details: CompactionDetails,
}

/// Approximates token count from message text.
pub fn estimate_tokens(messages: &[Value]) -> i64 {
    let mut total = 0i64;
    for m in messages {
        if let Some(text) = m.get("content").and_then(|c| c.as_str()) {
            total += (text.len() as i64) / 4;
        }
    }
    total
}

pub fn estimate_context_tokens(messages: &[Value]) -> i64 {
    estimate_tokens(messages)
}

pub fn calculate_context_tokens(messages: &[Value]) -> i64 {
    estimate_tokens(messages)
}

pub fn should_compact(estimate: &ContextUsageEstimate, settings: &CompactionSettings) -> bool {
    estimate.tokens >= settings.max_context_tokens
}

pub fn find_turn_start_index(messages: &[Value]) -> usize {
    for (i, m) in messages.iter().enumerate() {
        if m.get("role").and_then(|r| r.as_str()) == Some("user") {
            return i;
        }
    }
    0
}

pub fn find_cut_point(messages: &[Value], keep_recent: i64) -> usize {
    if messages.len() as i64 <= keep_recent {
        0
    } else {
        (messages.len() as i64 - keep_recent) as usize
    }
}

pub fn serialize_conversation(messages: &[Value]) -> String {
    serde_json::to_string(messages).unwrap_or_default()
}

pub fn prepare_compaction(messages: &[Value], settings: &CompactionSettings) -> CompactionPreparation {
    let tokens_before = estimate_tokens(messages);
    CompactionPreparation {
        summary: String::new(),
        tokens_before,
        tokens_after: Some(tokens_before / 2),
        first_kept_entry_id: messages.get(find_cut_point(messages, settings.keep_recent_messages))
            .and_then(|m| m.get("id").and_then(|v| v.as_str()))
            .map(|s| s.to_string()),
    }
}

pub async fn compact(
    messages: Vec<Value>,
    _settings: CompactionSettings,
) -> CompactionResult {
    let tokens_before = estimate_tokens(&messages);
    CompactionResult {
        summary: String::new(),
        details: CompactionDetails {
            reason: "manual".to_string(),
            tokens_before,
            tokens_after: Some(tokens_before / 2),
        },
    }
}

pub async fn generate_summary(text: &str) -> String {
    text.chars().take(200).collect()
}

pub fn get_last_assistant_usage(messages: &[Value]) -> Option<Value> {
    for m in messages.iter().rev() {
        if m.get("role").and_then(|r| r.as_str()) == Some("assistant") {
            if let Some(usage) = m.get("usage") {
                return Some(usage.clone());
            }
        }
    }
    None
}