//! Prompt-cache stability helpers.
//! 翻译自 packages/ai/src/utils/prompt-cache-stability.ts

/// Returns a normalized, cache-stable representation of `text` suitable
/// for use as a prompt cache key. Collapses whitespace and trims trailing
/// punctuation that often varies across UI inputs.
pub fn stable_cache_key(text: &str) -> String {
    let trimmed = text.trim();
    let mut out = String::with_capacity(trimmed.len());
    let mut prev_was_space = false;
    for c in trimmed.chars() {
        if c.is_whitespace() {
            if !prev_was_space {
                out.push(' ');
                prev_was_space = true;
            }
        } else {
            out.push(c);
            prev_was_space = false;
        }
    }
    out.trim_end_matches(|c: char| !c.is_alphanumeric()).to_string()
}

/// Compares two text inputs for cache-stable equivalence.
pub fn is_cache_stable(a: &str, b: &str) -> bool {
    stable_cache_key(a) == stable_cache_key(b)
}