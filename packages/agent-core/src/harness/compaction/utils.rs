// Harness compaction utilities.
// 翻译自 packages/agent-core/src/harness/compaction/utils.ts

pub fn truncate_for_summary(text: &str, max_chars: usize) -> String {
    if text.chars().count() <= max_chars {
        text.to_string()
    } else {
        let truncated: String = text.chars().take(max_chars).collect();
        truncated
    }
}