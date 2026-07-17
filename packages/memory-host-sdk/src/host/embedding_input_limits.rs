// Embedding input limits helper.
// 翻译自 packages/memory-host-sdk/src/host/embedding-input-limits.ts

pub const MAX_INPUT_TOKENS: i64 = 8_000;

pub fn truncate_input_tokens(text: &str, max_tokens: i64) -> String {
    let max_chars = (max_tokens as usize) * 4;
    if text.len() <= max_chars {
        text.to_string()
    } else {
        text[..max_chars].to_string()
    }
}