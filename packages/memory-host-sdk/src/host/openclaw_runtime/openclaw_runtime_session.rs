// CradleRing runtime session helpers.
// 翻译自 packages/memory-host-sdk/src/host/openclaw-runtime-session.ts

pub const SILENT_REPLY_TOKEN: &str = "NO_REPLY";

pub fn session_key_to_agent_id(_session_key: &str) -> String {
    "default".to_string()
}