// Terminal Core module implements note behavior.
// 翻译自 packages/terminal-core/src/note.ts

use std::io::IsTerminal;

use once_cell::sync::Lazy;
use regex::Regex;

use crate::ansi::{split_graphemes, visible_width};
use crate::prompt_style::style_prompt_title;
use crate::string::normalize_lowercase_string_or_empty;

const MIN_NOTE_COLUMNS: usize = 80;

static URL_PREFIX_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^(https?://|file://)").unwrap());
static WINDOWS_DRIVE_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^[a-zA-Z]:[\\/]").unwrap());
static FILE_LIKE_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^[a-zA-Z0-9._-]+$").unwrap());

thread_local! {
    static SUPPRESS_NOTES: std::cell::Cell<bool> = std::cell::Cell::new(false);
}

fn is_suppressed_by_env(value: Option<&str>) -> bool {
    let Some(v) = value else {
        return false;
    };
    let normalized = normalize_lowercase_string_or_empty(v);
    if normalized.is_empty() {
        return false;
    }
    normalized != "0" && normalized != "false" && normalized != "off"
}

fn split_long_word(word: &str, max_len: usize) -> Vec<String> {
    if max_len == 0 {
        return vec![word.to_string()];
    }
    let mut parts: Vec<String> = Vec::new();
    let mut current = String::new();
    let mut current_width: usize = 0;
    for grapheme in split_graphemes(word) {
        let width = visible_width(&grapheme);
        if !current.is_empty() && current_width + width > max_len {
            parts.push(current.clone());
            current.clear();
            current_width = 0;
        }
        current.push_str(&grapheme);
        current_width += width;
    }
    if !current.is_empty() {
        parts.push(current);
    }
    if parts.is_empty() {
        vec![word.to_string()]
    } else {
        parts
    }
}

fn is_copy_sensitive_token(word: &str) -> bool {
    if word.is_empty() {
        return false;
    }
    if URL_PREFIX_RE.is_match(word) {
        return true;
    }
    if word.starts_with('/')
        || word.starts_with("~/")
        || word.starts_with("./")
        || word.starts_with("../")
    {
        return true;
    }
    if WINDOWS_DRIVE_RE.is_match(word) || word.starts_with("\\\\") {
        return true;
    }
    if word.contains('/') || word.contains('\\') {
        return true;
    }
    // Preserve common file-like tokens.
    word.contains('_') && FILE_LIKE_RE.is_match(word)
}

struct WrapParams<'a> {
    word: &'a str,
    available: usize,
    first_prefix: String,
    continuation_prefix: String,
    lines: &'a mut Vec<String>,
}

fn push_wrapped_word_segments(params: WrapParams) {
    let parts = split_long_word(params.word, params.available);
    let mut iter = parts.into_iter();
    let first = iter.next().unwrap_or_default();
    params.lines.push(format!("{}{}", params.first_prefix, first));
    for part in iter {
        params.lines.push(format!("{}{}", params.continuation_prefix, part));
    }
}

fn wrap_line_text(line: &str, max_width: usize) -> Vec<String> {
    if line.trim().is_empty() {
        return vec![line.to_string()];
    }
    let indent_re = Regex::new(r"^(\s*)([-*\u{2022}]\s+)?(.*)$").unwrap();
    let captures = indent_re.captures(line);
    let indent = captures
        .as_ref()
        .and_then(|c| c.get(1).map(|m| m.as_str().to_string()))
        .unwrap_or_default();
    let bullet = captures
        .as_ref()
        .and_then(|c| c.get(2).map(|m| m.as_str().to_string()))
        .unwrap_or_default();
    let content = captures
        .as_ref()
        .and_then(|c| c.get(3).map(|m| m.as_str().to_string()))
        .unwrap_or_default();
    let first_prefix = format!("{}{}", indent, bullet);
    let next_prefix = format!(
 "{}{}",
        indent,
        if bullet.is_empty() {
            String::new()
        } else {
            " ".repeat(bullet.len())
        }
    );
    let first_width = std::cmp::max(10, max_width.saturating_sub(visible_width(&first_prefix)));
    let next_width = std::cmp::max(10, max_width.saturating_sub(visible_width(&next_prefix)));

    let words: Vec<&str> = content.split_whitespace().collect();
    let has_words = !words.is_empty();
    let mut lines: Vec<String> = Vec::new();
    let mut current = String::new();
    let mut prefix = first_prefix.clone();
    let mut available = first_width;

    for word in &words {
        let w = word.to_string();
        if current.is_empty() {
            if visible_width(&w) > available {
                if is_copy_sensitive_token(&w) {
                    current = w;
                    continue;
                }
                push_wrapped_word_segments(WrapParams {
                    word: &w,
                    available,
                    first_prefix: prefix.clone(),
                    continuation_prefix: next_prefix.clone(),
                    lines: &mut lines,
                });
                prefix = next_prefix.clone();
                available = next_width;
                continue;
            }
            current = w;
            continue;
        }
        let candidate = format!("{} {}", current, w);
        if visible_width(&candidate) <= available {
            current = candidate;
            continue;
        }
        lines.push(format!("{}{}", prefix, current));
        prefix = next_prefix.clone();
        available = next_width;

        if visible_width(&w) > available {
            if is_copy_sensitive_token(&w) {
                current = w;
                continue;
            }
            push_wrapped_word_segments(WrapParams {
                word: &w,
                available,
                first_prefix: prefix.clone(),
                continuation_prefix: prefix.clone(),
                lines: &mut lines,
            });
            current.clear();
            continue;
        }
        current = w;
    }

    if !current.is_empty() || !has_words {
        lines.push(format!("{}{}", prefix, current));
    }

    lines
}

fn coerce_note_message(message: &dyn std::any::Any) -> String {
    if let Some(s) = message.downcast_ref::<String>() {
        return s.clone();
    }
    if let Some(s) = message.downcast_ref::<&str>() {
        return s.to_string();
    }
    if let Some(&n) = message.downcast_ref::<i64>() {
        return n.to_string();
    }
    if let Some(&n) = message.downcast_ref::<u64>() {
        return n.to_string();
    }
    if let Some(&b) = message.downcast_ref::<bool>() {
        return b.to_string();
    }
    String::new()
}

pub fn wrap_note_message(
    message: &dyn std::any::Any,
    max_width: Option<usize>,
    columns: Option<usize>,
) -> String {
    let text = coerce_note_message(message);
    let cols = columns.unwrap_or_else(|| resolve_note_columns(None));
    let mwidth = max_width.unwrap_or_else(|| std::cmp::max(40, std::cmp::min(88, cols - 10)));
    let lines: Vec<String> = text
        .split('\n')
        .flat_map(|line| wrap_line_text(line, mwidth).into_iter())
        .collect();
    lines.join("\n")
}

pub fn resolve_note_columns(columns: Option<usize>) -> usize {
    match columns {
        Some(c) if c >= MIN_NOTE_COLUMNS => c,
        _ => MIN_NOTE_COLUMNS,
    }
}

pub fn resolve_note_output_columns(message: &str, columns: usize) -> usize {
    let widest = message
        .split('\n')
        .map(|line| visible_width(line))
        .max()
        .unwrap_or(0);
    std::cmp::max(columns, widest + 6)
}

fn stdout_columns() -> Option<usize> {
    // Best-effort terminal size detection: prefer the standard env if set,
    // otherwise fall back to a hard-coded default. Avoid pulling in
    // `terminal_size` here so this crate remains dependency-light.
    std::env::var("COLUMNS")
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
}

pub fn note<M: std::any::Any>(message: M, title: Option<&str>) {
    let suppressed = SUPPRESS_NOTES.with(|cell| cell.get())
        || is_suppressed_by_env(std::env::var("OPENCLAW_SUPPRESS_NOTES").ok().as_deref());
    if suppressed {
        return;
    }
    let cols = resolve_note_columns(stdout_columns());
    let wrapped = wrap_note_message(&message, None, Some(cols));
    let _ = cols;
    let _ = wrapped;
    let _ = style_prompt_title(title);
    // The TS implementation invokes `clackNote(...)` with a custom Output
    // wrapping stdout. In CradleRing we emit directly to stdout to keep this
    // crate UI-runtime-agnostic.
    if std::io::stdout().is_terminal() {
        print!("{}", wrapped);
    }
}

pub fn with_suppressed_notes<T>(callback: impl FnOnce() -> T) -> T {
    let prev = SUPPRESS_NOTES.with(|cell| cell.get());
    SUPPRESS_NOTES.with(|cell| cell.set(true));
    let result = callback();
    SUPPRESS_NOTES.with(|cell| cell.set(prev));
    result
}
