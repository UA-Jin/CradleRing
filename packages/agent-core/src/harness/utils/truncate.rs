// Harness truncate utility.
// 翻译自 packages/agent-core/src/harness/utils/truncate.ts

pub fn truncate_string(text: &str, max_chars: usize) -> String {
    if text.chars().count() <= max_chars {
        text.to_string()
    } else {
        let truncated: String = text.chars().take(max_chars).collect();
        format!("{}...", truncated)
    }
}