// Terminal Core module implements decorative emoji behavior.
// 翻译自 packages/terminal-core/src/decorative-emoji.ts

use once_cell::sync::Lazy;
use regex::Regex;

use crate::ansi::split_graphemes;

pub use crate::display_string::EnvMap;
#[doc(hidden)]
pub use crate::display_string::EnvMap as _DisplayEnvMap;

#[derive(Default)]
pub struct DecorativeEmojiOptions {
    pub env: Option<EnvMap>,
    pub is_tty: Option<bool>,
    pub platform: Option<String>,
    pub stream_is_tty: Option<bool>,
}

static EMOJI_GRAPHEME_PATTERN: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"[\p{Extended_Pictographic}\p{Regional_Indicator}\u{20e3}]").unwrap()
});

/// Detect terminals with known emoji rendering support.
fn is_known_emoji_terminal(env: &EnvMap) -> bool {
    let term_program = env
        .get("TERM_PROGRAM")
        .map(|s| s.to_lowercase())
        .unwrap_or_default();
    let term = env.get("TERM").map(|s| s.to_lowercase()).unwrap_or_default();
    term_program.contains("iterm")
        || term_program.contains("apple_terminal")
        || term_program.contains("ghostty")
        || term_program.contains("wezterm")
        || term_program.contains("vscode")
        || term.contains("ghostty")
        || term.contains("wezterm")
        || env.contains_key("WT_SESSION")
}

/// Return true when locale variables indicate UTF-8 output support.
fn has_utf8_locale(env: &EnvMap) -> bool {
    let locale_keys = ["LC_ALL", "LC_CTYPE", "LANG"];
    let has_locale = locale_keys
        .iter()
        .filter_map(|k| env.get(*k))
        .any(|v| !v.trim().is_empty());
    if !has_locale {
        return true;
    }
    let any_utf8 = locale_keys
        .iter()
        .filter_map(|k| env.get(*k))
        .any(|v| v.to_lowercase().contains("utf-8") || v.to_lowercase().contains("utf8"));
    any_utf8
}

fn env_lookup(env: &EnvMap, key: &str) -> Option<String> {
    env.get(key).cloned()
}

/// Return true when decorative emoji should be emitted for the target terminal.
pub fn supports_decorative_emoji(options: &DecorativeEmojiOptions) -> bool {
    let env: EnvMap = options
        .env
        .clone()
        .unwrap_or_else(|| std::env::vars().collect());
    let platform = options
        .platform
        .clone()
        .unwrap_or_else(|| std::env::consts::OS.to_string());
    let is_tty = options
        .is_tty
        .or(options.stream_is_tty)
        .unwrap_or_else(|| atty_stdout_is_tty());

    if !is_tty {
        return false;
    }
    let term_lower = env_lookup(&env, "TERM")
        .map(|s| s.to_lowercase())
        .unwrap_or_default();
    if term_lower == "dumb" {
        return false;
    }
    if !has_utf8_locale(&env) {
        return false;
    }
    if is_known_emoji_terminal(&env) {
        return true;
    }
    if platform == "darwin" {
        return true;
    }
    false
}

fn atty_stdout_is_tty() -> bool {
    // Use libc-style isatty via std::io::IsTerminal if available; otherwise false.
    use std::io::IsTerminal;
    std::io::stdout().is_terminal()
}

/// Return the emoji only when decorative emoji output is supported.
pub fn decorative_emoji(emoji: &str, options: &DecorativeEmojiOptions) -> String {
    if supports_decorative_emoji(options) {
        emoji.to_string()
    } else {
        String::new()
    }
}

/// Prefix text with a decorative emoji when supported.
pub fn decorative_prefix(emoji: &str, text: &str, options: &DecorativeEmojiOptions) -> String {
    let prefix = decorative_emoji(emoji, options);
    if prefix.is_empty() {
        text.to_string()
    } else {
        format!("{} {}", prefix, text)
    }
}

/// Strip decorative emoji for terminals that should not receive them.
pub fn strip_decorative_emoji_for_terminal(text: &str, options: &DecorativeEmojiOptions) -> String {
    if supports_decorative_emoji(options) {
        return text.to_string();
    }
    let filtered: String = split_graphemes(text)
        .into_iter()
        .filter(|g| !EMOJI_GRAPHEME_PATTERN.is_match(g))
        .collect::<Vec<_>>()
        .join("");
    // collapse multiple whitespace
    let re_ws = Regex::new(r"\s{2,}").unwrap();
    re_ws.replace_all(&filtered, " ").trim().to_string()
}
