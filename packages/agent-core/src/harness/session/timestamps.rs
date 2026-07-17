// Harness session timestamp helpers.
// 翻译自 packages/agent-core/src/harness/session/timestamps.ts

pub fn now_ms() -> i64 {
    chrono::Utc::now().timestamp_millis()
}