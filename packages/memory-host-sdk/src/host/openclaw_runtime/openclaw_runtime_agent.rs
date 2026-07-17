// CradleRing runtime agent helpers.
// 翻译自 packages/memory-host-sdk/src/host/openclaw-runtime-agent.ts

use serde_json::Value;

pub const SILENT_REPLY_TOKEN: &str = "NO_REPLY";
pub const DEFAULT_AGENT_COMPACTION_RESERVE_TOKENS_FLOOR: i64 = 8_000;

pub type AnyAgentTool = Box<dyn std::fmt::Debug + Send + Sync>;

pub fn resolve_cron_style_now() -> i64 {
    chrono::Utc::now().timestamp_millis()
}

pub fn resolve_default_agent_id() -> String {
    "default".to_string()
}

pub fn resolve_session_agent_id(_session_key: &str) -> String {
    resolve_default_agent_id()
}

pub fn parse_agent_session_key(key: &str) -> Option<(String, String)> {
    let parts: Vec<&str> = key.splitn(2, ':').collect();
    if parts.len() == 2 {
        Some((parts[0].to_string(), parts[1].to_string()))
    } else {
        None
    }
}

pub fn resolve_memory_search_config() -> Value {
    Value::Null
}

pub fn as_tool_params_record(value: &Value) -> std::collections::HashMap<String, Value> {
    match value.as_object() {
        Some(map) => map.iter().map(|(k, v)| (k.clone(), v.clone())).collect(),
        None => std::collections::HashMap::new(),
    }
}

pub fn json_result(value: Value) -> Value {
    value
}

pub fn read_number_param(value: &Value, key: &str) -> Option<f64> {
    value.get(key).and_then(|v| v.as_f64())
}

pub fn read_string_param(value: &Value, key: &str) -> Option<String> {
    value.get(key).and_then(|v| v.as_str()).map(|s| s.to_string())
}