// Secret input utils.
// 翻译自 packages/memory-host-sdk/src/host/secret-input-utils.ts

pub fn mask_secret(value: &str) -> String {
    if value.is_empty() {
        return String::new();
    }
    format!("{}...", &value[..value.len().min(4)])
}