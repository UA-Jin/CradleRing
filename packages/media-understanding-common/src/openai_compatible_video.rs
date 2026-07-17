// OpenAI-compatible video request/response helpers.
// 翻译自 packages/media-understanding-common/src/openai-compatible-video.ts

use base64::Engine;
use serde_json::{json, Value};

/** Minimal OpenAI-compatible video response payload shape. */
#[derive(Debug, Clone, Default)]
pub struct OpenAiCompatibleVideoPayload {
    pub choices: Option<Vec<OpenAiCompatibleVideoChoice>>,
}

#[derive(Debug, Clone, Default)]
pub struct OpenAiCompatibleVideoChoice {
    pub message: Option<OpenAiCompatibleVideoMessage>,
}

#[derive(Debug, Clone, Default)]
pub struct OpenAiCompatibleVideoMessage {
    pub content: Option<OpenAiCompatibleVideoContent>,
    pub reasoning_content: Option<String>,
}

#[derive(Debug, Clone)]
pub enum OpenAiCompatibleVideoContent {
    Text(String),
    Parts(Vec<OpenAiCompatibleVideoPart>),
}

#[derive(Debug, Clone, Default)]
pub struct OpenAiCompatibleVideoPart {
    pub text: Option<String>,
}

/** Trim optional strings, falling back when empty. */
pub fn resolve_media_understanding_string(value: Option<&str>, fallback: &str) -> String {
    let trimmed = value.map(|s| s.trim());
    match trimmed {
        Some(t) if !t.is_empty() => t.to_string(),
        _ => fallback.to_string(),
    }
}

/** Coerce text from OpenAI-compatible content or reasoning fields. */
pub fn coerce_openai_compatible_video_text(
    payload: &OpenAiCompatibleVideoPayload,
) -> Option<String> {
    let message = payload.choices.as_ref()?.first()?.message.as_ref()?;
    if let Some(content) = &message.content {
        match content {
            OpenAiCompatibleVideoContent::Text(s) => {
                let t = s.trim();
                if !t.is_empty() {
                    return Some(t.to_string());
                }
            }
            OpenAiCompatibleVideoContent::Parts(parts) => {
                let text = parts
                    .iter()
                    .map(|p| p.text.as_deref().map(|t| t.trim()).unwrap_or(""))
                    .filter(|t| !t.is_empty())
                    .collect::<Vec<_>>()
                    .join("\n");
                if !text.is_empty() {
                    return Some(text);
                }
            }
        }
    }
    if let Some(reasoning) = &message.reasoning_content {
        let t = reasoning.trim();
        if !t.is_empty() {
            return Some(t.to_string());
        }
    }
    None
}

pub struct BuildOpenAiCompatibleVideoRequestBodyParams<'a> {
    pub model: &'a str,
    pub prompt: &'a str,
    pub mime: &'a str,
    pub buffer: &'a [u8],
}

/** Build an OpenAI-compatible request body with an inline data URL video. */
pub fn build_openai_compatible_video_request_body(
    params: BuildOpenAiCompatibleVideoRequestBodyParams,
) -> Value {
    let encoded = base64::engine::general_purpose::STANDARD.encode(params.buffer);
    json!({
        "model": params.model,
        "messages": [
            {
                "role": "user",
                "content": [
                    { "type": "text", "text": params.prompt },
                    {
                        "type": "video_url",
                        "video_url": {
                            "url": format!("data:{};base64,{}", params.mime, encoded),
                        }
                    }
                ]
            }
        ]
    })
}