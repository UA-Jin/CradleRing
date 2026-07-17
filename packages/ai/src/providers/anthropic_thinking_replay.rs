//! Anthropic thinking-replay helpers.
//! 翻译自 packages/ai/src/providers/anthropic-thinking-replay.ts

/// Marker text used when a replayed assistant reasoning block is omitted.
pub const ANTHROPIC_OMITTED_REASONING_TEXT: &str = "[assistant reasoning omitted]";

/// Find the assistant message index whose tool calls precede the trailing
/// tool-result block. Used to decide whether signed thinking should be
/// preserved across the next request.
pub fn find_active_anthropic_tool_turn_assistant_index(messages: &[serde_json::Value]) -> i64 {
    let mut tool_result_ids: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
    let mut index = messages.len() as i64 - 1;
    while index >= 0 {
        let Some(message) = messages.get(index as usize) else {
            break;
        };
        let role = message.get("role").and_then(|v| v.as_str());
        if role != Some("toolResult") {
            break;
        }
        if let Some(id) = message.get("toolCallId").and_then(|v| v.as_str()) {
            tool_result_ids.insert(id.to_string());
        }
        index -= 1;
    }
    if tool_result_ids.is_empty() {
        return -1;
    }
    let Some(assistant) = messages.get(index as usize) else {
        return -1;
    };
    if assistant.get("role").and_then(|v| v.as_str()) != Some("assistant") {
        return -1;
    }
    let Some(content) = assistant.get("content").and_then(|v| v.as_array()) else {
        return -1;
    };

    let mut tool_call_ids: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
    for block in content {
        let block_type = block.get("type").and_then(|v| v.as_str());
        if !matches!(block_type, Some("toolCall") | Some("tool_use") | Some("function_call")) {
            continue;
        }
        if let Some(id) = block.get("id").and_then(|v| v.as_str()) {
            tool_call_ids.insert(id.to_string());
        }
    }

    if tool_result_ids
        .iter()
        .all(|id| tool_call_ids.contains(id))
    {
        index
    } else {
        -1
    }
}