// Tool Call Repair module implements promote behavior.
// 1:1 port of openclaw-main/packages/tool-call-repair/src/promote.ts
// openclaw -> cradle-ring renames applied. Logic preserved line-by-line.

use std::collections::{HashMap, HashSet};

use crate::payload::{parse_standalone_plain_text_tool_call_blocks, PlainTextToolCallBlock};
use serde_json::{Map, Value as JsonValue};

/// Resolves model-emitted tool names to the exact names allowed by the provider request.
pub type ToolCallRepairNameResolver =
    fn(raw_name: &str, allowed_tool_names: &HashSet<String>) -> Option<String>;

/// Builds a provider-native tool-call block from a repaired plain-text payload.
pub type PromotedPlainTextToolCallBlockFactory =
    fn(block: &PlainTextToolCallBlock, resolved_name: &str) -> Map<String, JsonValue>;

/// Controls when standalone assistant text may be rewritten as tool-call content.
pub struct PlainTextToolCallPromotionOptions {
    pub allowed_stop_reasons: Option<HashSet<JsonValue>>,
    pub allowed_tool_names: HashSet<String>,
    pub create_tool_call_block: PromotedPlainTextToolCallBlockFactory,
    pub is_retainable_non_text_block: Option<fn(block: &Map<String, JsonValue>) -> bool>,
    pub message: JsonValue,
    pub require_assistant_role: Option<bool>,
    pub resolve_tool_name: Option<ToolCallRepairNameResolver>,
}

#[derive(Debug, Clone)]
pub struct PlainTextToolCallMessageProjection {
    pub message: Map<String, JsonValue>,
    pub source_to_projected_content_index: HashMap<usize, usize>,
}

fn random_call_id() -> String {
    // Mirror crypto.randomUUID().replace(/-/g, "").slice(0, 24)
    // Use a deterministic pseudo-UUID for the 1:1 port; the JS relied on the runtime
    // crypto. We use a simple counter + time seed to mimic the same shape.
    use std::sync::atomic::{AtomicU64, Ordering};
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let n = COUNTER.fetch_add(1, Ordering::SeqCst);
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(0);
    let combined = now.wrapping_mul(0x9E3779B97F4A7C15).wrapping_add(n);
    let hex = format!("{:024x}", combined & 0xFFFFFFFFFFFF);
    format!("call_{}", &hex[..hex.len().min(24)])
}

/// Builds the shared assistant-message shape for a repaired text tool call.
pub fn create_promoted_plain_text_tool_call_block(
    block: &PlainTextToolCallBlock,
    name: &str,
) -> Map<String, JsonValue> {
    let mut map = Map::new();
    map.insert("type".to_string(), JsonValue::String("toolCall".to_string()));
    map.insert("id".to_string(), JsonValue::String(random_call_id()));
    map.insert("name".to_string(), JsonValue::String(name.to_string()));
    let args = JsonValue::Object(block.arguments.clone());
    map.insert("arguments".to_string(), args.clone());
    map.insert(
        "partialArgs".to_string(),
        JsonValue::String(serde_json::to_string(&args).unwrap_or_else(|_| "{}".to_string())),
    );
    map
}

fn as_record(value: &JsonValue) -> Option<Map<String, JsonValue>> {
    match value {
        JsonValue::Object(map) => Some(map.clone()),
        _ => None,
    }
}

fn as_record_ref(value: &JsonValue) -> Option<&Map<String, JsonValue>> {
    match value {
        JsonValue::Object(map) => Some(map),
        _ => None,
    }
}

/// Emits the complete provider-neutral lifecycle for promoted tool-call blocks.
pub fn create_promoted_plain_text_tool_call_events(
    message: &Map<String, JsonValue>,
) -> Vec<Map<String, JsonValue>> {
    let content = match message.get("content") {
        Some(JsonValue::Array(arr)) => arr.clone(),
        _ => Vec::new(),
    };
    let mut out = Vec::new();
    for (content_index, block) in content.iter().enumerate() {
        let tool_call = match as_record_ref(block) {
            Some(map) => map,
            None => continue,
        };
        if tool_call.get("type").and_then(|v| v.as_str()) != Some("toolCall") {
            continue;
        }
        let mut start = Map::new();
        start.insert("type".to_string(), JsonValue::String("toolcall_start".to_string()));
        start.insert("contentIndex".to_string(), JsonValue::Number(content_index.into()));
        start.insert("partial".to_string(), JsonValue::Object(message.clone()));
        out.push(start);

        let mut delta = Map::new();
        delta.insert("type".to_string(), JsonValue::String("toolcall_delta".to_string()));
        delta.insert("contentIndex".to_string(), JsonValue::Number(content_index.into()));
        let partial_args = tool_call
            .get("partialArgs")
            .and_then(|v| v.as_str())
            .unwrap_or("{}")
            .to_string();
        delta.insert("delta".to_string(), JsonValue::String(partial_args));
        delta.insert("partial".to_string(), JsonValue::Object(message.clone()));
        out.push(delta);

        let mut end = Map::new();
        end.insert("type".to_string(), JsonValue::String("toolcall_end".to_string()));
        end.insert("contentIndex".to_string(), JsonValue::Number(content_index.into()));
        end.insert("toolCall".to_string(), JsonValue::Object(tool_call.clone()));
        end.insert("partial".to_string(), JsonValue::Object(message.clone()));
        out.push(end);
    }
    out
}

fn resolve_exact_tool_name(raw_name: &str, allowed_tool_names: &HashSet<String>) -> Option<String> {
    if allowed_tool_names.contains(raw_name) {
        Some(raw_name.to_string())
    } else {
        None
    }
}

fn create_promoted_tool_call_blocks(
    text: &str,
    options: &PlainTextToolCallPromotionOptions,
    line_break_offsets: Option<&HashSet<usize>>,
) -> Option<Vec<Map<String, JsonValue>>> {
    let structural_breaks_owned: Option<crate::grammar::StructuralLineBreakOptions> =
        line_break_offsets.map(|offsets| crate::grammar::StructuralLineBreakOptions {
            line_break_offsets: offsets.clone(),
            used_line_break_offsets: None,
        });
    let parsed_blocks = parse_standalone_plain_text_tool_call_blocks(
        text,
        None,
        structural_breaks_owned.as_ref(),
    )?;
    let resolve_tool_name = options.resolve_tool_name.unwrap_or(resolve_exact_tool_name);
    let mut tool_calls: Vec<Map<String, JsonValue>> = Vec::new();
    for block in &parsed_blocks {
        let resolved_name = match resolve_tool_name(&block.name, &options.allowed_tool_names) {
            Some(n) => n,
            None => return None,
        };
        let tool_call = (options.create_tool_call_block)(block, &resolved_name);
        tool_calls.push(tool_call);
    }
    Some(tool_calls)
}

fn create_promoted_tool_call_blocks_from_text_parts(
    text_parts: &[String],
    options: &PlainTextToolCallPromotionOptions,
) -> Option<Vec<Map<String, JsonValue>>> {
    let text: String = text_parts.join("");
    if text.trim().is_empty() {
        return Some(Vec::new());
    }
    let mut offset: usize = 0;
    let mut line_break_offsets: HashSet<usize> = HashSet::new();
    if !text_parts.is_empty() {
        for part in text_parts.iter().take(text_parts.len().saturating_sub(1)) {
            offset += part.len();
            line_break_offsets.insert(offset);
        }
    }
    if line_break_offsets.contains(&text.len()) {
        line_break_offsets.remove(&text.len());
    }
    create_promoted_tool_call_blocks(&text, options, Some(&line_break_offsets))
}

/// Promotes text calls and maps source blocks retained in the projected message.
pub fn project_standalone_plain_text_tool_call_message(
    options: &PlainTextToolCallPromotionOptions,
) -> Option<PlainTextToolCallMessageProjection> {
    let message_record = as_record(&options.message)?;
    if options.allowed_tool_names.is_empty() {
        return None;
    }
    if options.require_assistant_role == Some(true)
        && message_record.get("role").and_then(|v| v.as_str()) != Some("assistant")
    {
        return None;
    }
    if let Some(allowed) = options.allowed_stop_reasons.as_ref() {
        let stop_reason = message_record.get("stopReason");
        let stop_reason_value = stop_reason.cloned().unwrap_or(JsonValue::Null);
        if !allowed.contains(&stop_reason_value) {
            return None;
        }
    }

    let original_content = message_record.get("content").cloned().unwrap_or(JsonValue::Null);
    if let Some(text) = original_content.as_str() {
        let tool_calls = create_promoted_tool_call_blocks(text.trim(), options, None)?;
        let mut new_message = message_record.clone();
        let tool_calls_value: Vec<JsonValue> =
            tool_calls.into_iter().map(JsonValue::Object).collect();
        new_message.insert("content".to_string(), JsonValue::Array(tool_calls_value));
        new_message.insert(
            "stopReason".to_string(),
            JsonValue::String("toolUse".to_string()),
        );
        let source_to_projected_content_index: HashMap<usize, usize> = HashMap::new();
        return Some(PlainTextToolCallMessageProjection {
            message: new_message,
            source_to_projected_content_index,
        });
    }

    let original_array = match original_content {
        JsonValue::Array(arr) => arr,
        _ => return None,
    };

    let mut content: Vec<Map<String, JsonValue>> = Vec::new();
    let mut source_to_projected_content_index: HashMap<usize, usize> = HashMap::new();
    let mut promoted_text_block = false;
    let mut text_parts: Vec<String> = Vec::new();
    let flush_text_parts = |text_parts: &mut Vec<String>,
                            content: &mut Vec<Map<String, JsonValue>>,
                            promoted_text_block: &mut bool,
                            options: &PlainTextToolCallPromotionOptions|
     -> Option<()> {
        let tool_calls = create_promoted_tool_call_blocks_from_text_parts(text_parts, options)?;
        text_parts.clear();
        if tool_calls.is_empty() {
            return Some(());
        }
        for tc in tool_calls {
            content.push(tc);
        }
        *promoted_text_block = true;
        Some(())
    };

    for (source_index, block) in original_array.iter().enumerate() {
        let block_record = match as_record(block) {
            Some(m) => m,
            None => return None,
        };
        if block_record.get("type").and_then(|v| v.as_str()) == Some("text") {
            let text = match block_record.get("text").and_then(|v| v.as_str()) {
                Some(s) => s.to_string(),
                None => return None,
            };
            text_parts.push(text);
            continue;
        }
        if flush_text_parts(
            &mut text_parts,
            &mut content,
            &mut promoted_text_block,
            options,
        )
        .is_none()
        {
            return None;
        }
        if let Some(is_retainable) = options.is_retainable_non_text_block {
            if is_retainable(&block_record) {
                source_to_projected_content_index.insert(source_index, content.len());
                content.push(block_record);
                continue;
            }
        }
        return None;
    }

    if flush_text_parts(
        &mut text_parts,
        &mut content,
        &mut promoted_text_block,
        options,
    )
    .is_none()
    {
        return None;
    }
    if !promoted_text_block {
        return None;
    }

    let mut new_message = message_record.clone();
    let array_content: Vec<JsonValue> = content.into_iter().map(JsonValue::Object).collect();
    new_message.insert("content".to_string(), JsonValue::Array(array_content));
    new_message.insert(
        "stopReason".to_string(),
        JsonValue::String("toolUse".to_string()),
    );
    Some(PlainTextToolCallMessageProjection {
        message: new_message,
        source_to_projected_content_index,
    })
}
