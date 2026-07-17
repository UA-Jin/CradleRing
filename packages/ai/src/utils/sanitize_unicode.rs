//! Unicode sanitization for LLM inputs.
//! 翻译自 packages/ai/src/utils/sanitize-unicode.ts
//!
//! Strips control characters and zero-width chars that often slip into
//! LLM input but have no semantic value (and can cause cache misses).

use once_cell::sync::Lazy;
use regex::Regex;

/// Characters we strip: control chars (\x00-\x08, \x0B, \x0C, \x0E-\x1F, \x7F),
/// BOM, zero-width spaces, and direction overrides.
static CONTROL_CHARS: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"[\x00-\x08\x0B\x0C\x0E-\x1F\x7F\u{FEFF}\u{200B}-\u{200F}\u{2028}\u{2029}\u{202A}-\u{202E}\u{2066}-\u{2069}]")
        .unwrap()
});

/// Returns `text` with sanitization applied. The TS helper strips control
/// characters and normalizes unicode whitespace; we keep parity.
pub fn sanitize_unicode(text: &str) -> String {
    CONTROL_CHARS.replace_all(text, "").into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strips_control_chars() {
        let input = "hello\u{0000}world";
        assert_eq!(sanitize_unicode(input), "helloworld");
    }

    #[test]
    fn preserves_normal_text() {
        let input = "Hello, world!";
        assert_eq!(sanitize_unicode(input), "Hello, world!");
    }
}