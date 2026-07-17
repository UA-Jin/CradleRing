//! OpenAI stop-reason mapping.
//! 翻译自 packages/ai/src/providers/openai-stop-reason.ts

use llm_core::types::StopReason;

#[derive(Debug, Clone)]
pub struct OpenAIStopReasonResult {
    pub stop_reason: StopReason,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct MapOpenAIStopReasonOptions {
    pub allow_singular_tool_call: Option<bool>,
}

/// Map a raw OpenAI finish_reason into a normalized `StopReason` + optional error message.
pub fn map_openai_stop_reason(
    reason: Option<&str>,
    options: Option<MapOpenAIStopReasonOptions>,
) -> OpenAIStopReasonResult {
    let Some(reason) = reason else {
        return OpenAIStopReasonResult {
            stop_reason: "stop".to_string(),
            error_message: None,
        };
    };

    match reason {
        "stop" | "end" => OpenAIStopReasonResult {
            stop_reason: "stop".to_string(),
            error_message: None,
        },
        "length" => OpenAIStopReasonResult {
            stop_reason: "length".to_string(),
            error_message: None,
        },
        "function_call" | "tool_calls" => OpenAIStopReasonResult {
            stop_reason: "toolUse".to_string(),
            error_message: None,
        },
        "tool_call" => {
            if options.and_then(|o| o.allow_singular_tool_call).unwrap_or(false) {
                OpenAIStopReasonResult {
                    stop_reason: "toolUse".to_string(),
                    error_message: None,
                }
            } else {
                OpenAIStopReasonResult {
                    stop_reason: "error".to_string(),
                    error_message: Some(format!("Provider finish_reason: {}", reason)),
                }
            }
        }
        "content_filter" => OpenAIStopReasonResult {
            stop_reason: "error".to_string(),
            error_message: Some("Provider finish_reason: content_filter".to_string()),
        },
        "network_error" => OpenAIStopReasonResult {
            stop_reason: "error".to_string(),
            error_message: Some("Provider finish_reason: network_error".to_string()),
        },
        other => OpenAIStopReasonResult {
            stop_reason: "error".to_string(),
            error_message: Some(format!("Provider finish_reason: {}", other)),
        },
    }
}