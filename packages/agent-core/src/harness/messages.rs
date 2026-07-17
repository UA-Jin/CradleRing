// Harness messages.
// 翻译自 packages/agent-core/src/harness/messages.ts

use serde_json::Value;

/// Returns the transcript timestamp for the given message.
pub fn message_timestamp(message: &Value) -> Option<i64> {
    message
        .get("timestamp")
        .and_then(|t| t.as_i64())
}

/// Stable entry id resolver used by branch/compaction helpers.
pub fn entry_id(message: &Value) -> Option<String> {
    message
        .get("id")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
}