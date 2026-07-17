// Terminal Core module implements prompt style behavior.
// 翻译自 packages/terminal-core/src/prompt-style.ts

use crate::theme::{is_rich, theme};

/// Style a prompt message when rich terminal output is active.
pub fn style_prompt_message(message: &str) -> String {
    if is_rich() {
        let t = theme();
        t.accent.hex(message)
    } else {
        message.to_string()
    }
}

/// Style a prompt title when rich terminal output is active.
pub fn style_prompt_title(title: Option<&str>) -> Option<String> {
    match title {
        None => None,
        Some(t) if t.is_empty() => Some(t.to_string()),
        Some(t) => {
            if is_rich() {
                let theme = theme();
                Some(theme.heading.hex(t))
            } else {
                Some(t.to_string())
            }
        }
    }
}

/// Style a prompt hint when rich terminal output is active.
pub fn style_prompt_hint(hint: Option<&str>) -> Option<String> {
    match hint {
        None => None,
        Some(h) if h.is_empty() => Some(h.to_string()),
        Some(h) => {
            if is_rich() {
                let theme = theme();
                Some(theme.muted.hex(h))
            } else {
                Some(h.to_string())
            }
        }
    }
}
