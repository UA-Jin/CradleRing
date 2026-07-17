//! OpenAI prompt-cache key helpers.
//! 翻译自 packages/ai/src/providers/openai-prompt-cache.ts

/// Maximum prompt cache key length accepted by OpenAI-compatible request metadata.
pub const OPENAI_PROMPT_CACHE_KEY_MAX_LENGTH: usize = 64;

/// Truncates a prompt cache key by Unicode code point count.
pub fn clamp_openai_prompt_cache_key(key: Option<&str>) -> Option<String> {
    let key = key?;
    let chars: Vec<char> = key.chars().collect();
    if chars.len() <= OPENAI_PROMPT_CACHE_KEY_MAX_LENGTH {
        return Some(key.to_string());
    }
    Some(chars[..OPENAI_PROMPT_CACHE_KEY_MAX_LENGTH].iter().collect())
}