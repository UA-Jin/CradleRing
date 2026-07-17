// CradleRing runtime config helpers.
// 翻译自 packages/memory-host-sdk/src/host/openclaw-runtime-config.ts

use std::collections::HashMap;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use serde_json::Value;

pub type MemoryCitationsMode = String;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CradleRingConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub memory: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agents: Option<HashMap<String, Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub env: Option<HashMap<String, String>>,
}

pub fn get_runtime_config() -> CradleRingConfig {
    CradleRingConfig::default()
}

/// @deprecated Use get_runtimeConfig(), or pass the already loaded config through the call path.
pub fn load_config() -> CradleRingConfig {
    get_runtime_config()
}

pub fn resolve_state_dir() -> PathBuf {
    std::env::var("CRADLE_RING_STATE_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from(".cradle-ring"))
}

pub fn resolve_session_transcripts_dir_for_agent(agent_id: &str) -> PathBuf {
    resolve_state_dir().join("agents").join(agent_id).join("sessions")
}

pub fn parse_non_negative_byte_size(value: &str) -> Option<i64> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return None;
    }
    let (num, mult) = if let Some(stripped) = trimmed.strip_suffix("GB") {
        (stripped.trim(), 1024_i64.pow(3))
    } else if let Some(stripped) = trimmed.strip_suffix("MB") {
        (stripped.trim(), 1024_i64.pow(2))
    } else if let Some(stripped) = trimmed.strip_suffix("KB") {
        (stripped.trim(), 1024_i64)
    } else if let Some(stripped) = trimmed.strip_suffix("B") {
        (stripped.trim(), 1_i64)
    } else {
        (trimmed, 1_i64)
    };
    num.parse::<i64>().ok().map(|n| n * mult)
}