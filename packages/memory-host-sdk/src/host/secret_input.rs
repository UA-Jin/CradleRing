// Secret input helper.
// 翻译自 packages/memory-host-sdk/src/host/secret-input.ts

pub fn resolve_secret_input(value: Option<&str>) -> Option<String> {
    value.and_then(|v| {
        let trimmed = v.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed.to_string())
        }
    })
}