//! System-prompt cache boundary helpers.
//! 翻译自 packages/ai/src/utils/system-prompt-cache-boundary.ts
//!
//! Inserts / detects a marker boundary in the system prompt so that
//! downstream caching layers can break on stable prefixes.

/// Stable marker used by all providers as the system-prompt cache break.
pub const SYSTEM_PROMPT_CACHE_BOUNDARY: &str = "<!--SYSTEM_CACHE_BOUNDARY-->";

/// Returns the system prompt with the cache boundary inserted.
pub fn with_cache_boundary(system_prompt: &str) -> String {
    if system_prompt.contains(SYSTEM_PROMPT_CACHE_BOUNDARY) {
        return system_prompt.to_string();
    }
    let mut out = String::with_capacity(system_prompt.len() + SYSTEM_PROMPT_CACHE_BOUNDARY.len() + 1);
    out.push_str(system_prompt);
    out.push('\n');
    out.push_str(SYSTEM_PROMPT_CACHE_BOUNDARY);
    out
}

/// Returns the position of the cache boundary in `system_prompt`, if any.
pub fn find_cache_boundary(system_prompt: &str) -> Option<usize> {
    system_prompt.find(SYSTEM_PROMPT_CACHE_BOUNDARY)
}