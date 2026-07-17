//! GitHub Copilot header helpers.
//! 翻译自 packages/ai/src/providers/github-copilot-headers.ts

use std::collections::BTreeMap;

use llm_core::types::{Message, UserMessageContent};

/// Copilot expects `X-Initiator` to indicate user vs agent initiator.
fn infer_copilot_initiator(messages: &[Message]) -> &'static str {
    match messages.last() {
        Some(Message::User(_)) => "user",
        Some(_) => "agent",
        None => "user",
    }
}

/// Returns true when any message contains an image input.
pub fn has_copilot_vision_input(messages: &[Message]) -> bool {
    for msg in messages {
        match msg {
            Message::User(user) => {
                if let UserMessageContent::Parts(parts) = &user.content {
                    if parts.iter().any(|p| matches!(p, llm_core::types::UserMessagePart::Image(_))) {
                        return true;
                    }
                }
            }
            Message::ToolResult(tool) => {
                if tool.content.iter().any(|p| matches!(p, llm_core::types::ToolResultPart::Image(_))) {
                    return true;
                }
            }
            _ => {}
        }
    }
    false
}

/// Builds the dynamic Copilot headers for a request.
pub fn build_copilot_dynamic_headers(params: CopilotHeadersParams) -> BTreeMap<String, String> {
    let mut headers: BTreeMap<String, String> = BTreeMap::new();
    headers.insert("X-Initiator".to_string(), infer_copilot_initiator(&params.messages).to_string());
    headers.insert("Openai-Intent".to_string(), "conversation-edits".to_string());

    if params.has_images {
        headers.insert("Copilot-Vision-Request".to_string(), "true".to_string());
    }
    headers
}

/// Parameters for `build_copilot_dynamic_headers`.
pub struct CopilotHeadersParams {
    pub messages: Vec<Message>,
    pub has_images: bool,
}