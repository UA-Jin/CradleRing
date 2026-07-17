// Response snippet helper.
// 翻译自 packages/memory-host-sdk/src/host/response-snippet.ts

pub fn build_response_snippet(text: &str, max_chars: usize) -> String {
    if text.chars().count() <= max_chars {
        text.to_string()
    } else {
        let truncated: String = text.chars().take(max_chars).collect();
        format!("{}...", truncated)
    }
}