//! OpenAI Responses API stream-compat helpers.
//! 翻译自 packages/ai/src/providers/openai-responses-stream-compat.ts

pub const OPENAI_RESPONSES_OUTPUT_TEXT_CONTENT_PART_TYPE: &str = "output_text";
pub const AZURE_RESPONSES_TEXT_CONTENT_PART_TYPE: &str = "text";
pub const OPENAI_RESPONSES_OUTPUT_TEXT_DELTA_EVENT_TYPE: &str = "response.output_text.delta";
pub const AZURE_RESPONSES_TEXT_DELTA_EVENT_TYPE: &str = "response.text.delta";

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(untagged)]
pub enum AzureResponsesTextContentPart {
    OutputText { r#type: String, text: String },
    Text { r#type: String, text: String },
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AzureResponsesTextDeltaEvent {
    pub r#type: String,
    pub delta: String,
}

pub fn is_responses_text_content_part_type(type_: &str) -> bool {
    type_ == OPENAI_RESPONSES_OUTPUT_TEXT_CONTENT_PART_TYPE
        || type_ == AZURE_RESPONSES_TEXT_CONTENT_PART_TYPE
}

pub fn is_responses_text_delta_event_type(type_: &str) -> bool {
    type_ == OPENAI_RESPONSES_OUTPUT_TEXT_DELTA_EVENT_TYPE
        || type_ == AZURE_RESPONSES_TEXT_DELTA_EVENT_TYPE
}

pub fn is_azure_responses_text_delta_event_type(type_: &str) -> bool {
    type_ == AZURE_RESPONSES_TEXT_DELTA_EVENT_TYPE
}

pub fn is_azure_responses_text_delta_event(event: &serde_json::Value) -> bool {
    let Some(obj) = event.as_object() else {
        return false;
    };
    let type_matches = obj
        .get("type")
        .and_then(|v| v.as_str())
        .map(is_azure_responses_text_delta_event_type)
        .unwrap_or(false);
    let delta_is_string = obj
        .get("delta")
        .map(|v| v.is_string())
        .unwrap_or(false);
    type_matches && delta_is_string
}

#[derive(Debug, Clone)]
pub enum ResponsesMessageSnapshotCollapse {
    Extend { text: String },
    Keep,
}

/// Some openai-responses providers re-emit the assistant message as cumulative
/// snapshot items. A same-phase strict extension replaces the prior text block,
/// or the visible reply repeats once per snapshot.
pub fn resolve_responses_message_snapshot_collapse(
    prior: Option<&SnapshotItem>,
    next_text: &str,
    next_phase: Option<&str>,
) -> ResponsesMessageSnapshotCollapse {
    let Some(prior) = prior else {
        return ResponsesMessageSnapshotCollapse::Keep;
    };
    if prior.text.is_empty() || next_text.is_empty() || prior.phase.as_deref() != next_phase {
        return ResponsesMessageSnapshotCollapse::Keep;
    }
    if next_text.len() > prior.text.len() && next_text.starts_with(&prior.text) {
        return ResponsesMessageSnapshotCollapse::Extend {
            text: next_text.to_string(),
        };
    }
    ResponsesMessageSnapshotCollapse::Keep
}

/// One prior snapshot entry for the collapse helper.
#[derive(Debug, Clone, Default)]
pub struct SnapshotItem {
    pub text: String,
    pub phase: Option<String>,
}