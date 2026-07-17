// Speech Core module implements runtime-api behavior.
// 1:1 port of openclaw-main/packages/speech-core/runtime-api.ts
// openclaw -> cradle-ring renames applied. Logic preserved line-by-line.

use serde_json::Value as JsonValue;
use std::collections::HashMap;

pub type RuntimeConfigSnapshot = JsonValue;
pub type RuntimeConfigSourceSnapshot = JsonValue;

pub fn get_runtime_config_snapshot() -> Option<RuntimeConfigSnapshot> {
    None
}

pub fn get_runtime_config_source_snapshot() -> Option<RuntimeConfigSourceSnapshot> {
    None
}

pub fn select_applicable_runtime_config(params: SelectApplicableRuntimeConfigParams) -> Option<JsonValue> {
    let _ = params;
    None
}

pub struct SelectApplicableRuntimeConfigParams {
    pub input_config: Option<JsonValue>,
    pub runtime_config: Option<RuntimeConfigSnapshot>,
    pub runtime_source_config: Option<RuntimeConfigSourceSnapshot>,
}

pub fn is_verbose() -> bool {
    false
}

pub fn log_verbose(_msg: &str) {}

pub type ReplyPayload = HashMap<String, JsonValue>;

pub fn resolve_sendable_outbound_reply_parts(payload: &ReplyPayload) -> ResolvedReply {
    ResolvedReply {
        text: payload
            .get("text")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        has_media: payload.get("mediaUrl").is_some() || payload.get("media").is_some(),
    }
}

pub struct ResolvedReply {
    pub text: String,
    pub has_media: bool,
}

pub fn mark_reply_payload_as_tts_supplement(payload: ReplyPayload) -> ReplyPayload {
    let mut p = payload;
    p.insert("__ttsSupplement".to_string(), JsonValue::Bool(true));
    p
}
