// Post JSON helper.
// 翻译自 packages/memory-host-sdk/src/host/post-json.ts

pub async fn post_json(_url: &str, _body: serde_json::Value) -> Result<serde_json::Value, String> {
    Ok(serde_json::Value::Null)
}