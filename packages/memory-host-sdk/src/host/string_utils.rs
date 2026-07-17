// String utilities.
// 翻译自 packages/memory-host-sdk/src/host/string-utils.ts

pub fn truncate_string(text: &str, max: usize) -> String {
    if text.chars().count() <= max {
        text.to_string()
    } else {
        let truncated: String = text.chars().take(max).collect();
        format!("{}...", truncated)
    }
}