//! Overflow helpers classify provider overflow errors and retryable responses.
//! 翻译自 packages/ai/src/utils/overflow.ts

use once_cell::sync::Lazy;
use regex::Regex;

use llm_core::types::AssistantMessage;

static CONFIGURED_CONTEXT_SIZE_OVERFLOW_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"prompt has [\d,]+ tokens?, but the configured context size is [\d,]+ tokens?")
        .unwrap()
});

/// Detects DS4-style raw token-count context overflow errors.
pub fn is_configured_context_size_overflow_error(error_message: &str) -> bool {
    CONFIGURED_CONTEXT_SIZE_OVERFLOW_RE.is_match(error_message)
}

/// Patterns that detect context-overflow errors from different providers.
static OVERFLOW_PATTERNS: Lazy<Vec<Regex>> = Lazy::new(|| {
    vec![
        Regex::new(r"prompt is too long").unwrap(),
        Regex::new(r"request_too_large").unwrap(),
        Regex::new(r"input is too long for requested model").unwrap(),
        Regex::new(r"exceeds the context window").unwrap(),
        Regex::new(r"exceeds (?:the )?(?:model'?s )?maximum context length of [\d,]+ tokens?").unwrap(),
        Regex::new(r"input token count.*exceeds the maximum").unwrap(),
        Regex::new(r"maximum prompt length is \d+").unwrap(),
        Regex::new(r"reduce the length of the messages").unwrap(),
        Regex::new(r"maximum context length is \d+ tokens").unwrap(),
        Regex::new(r"input \(\d+ tokens\) is longer than the model'?s context length \(\d+ tokens\)").unwrap(),
        Regex::new(r"exceeds the limit of \d+").unwrap(),
        Regex::new(r"exceeds the available context size").unwrap(),
        Regex::new(r"greater than the context length").unwrap(),
        Regex::new(r"context window exceeds limit").unwrap(),
        Regex::new(r"exceeded model token limit").unwrap(),
        Regex::new(r"too large for model with \d+ maximum context length").unwrap(),
        Regex::new(r"prompt has [\d,]+ tokens?, but the configured context size is [\d,]+ tokens?").unwrap(),
        Regex::new(r"model_context_window_exceeded").unwrap(),
        Regex::new(r"prompt too long; exceeded (?:max )?context length").unwrap(),
        Regex::new(r"context[_ ]length[_ ]exceeded").unwrap(),
        Regex::new(r"too many tokens").unwrap(),
        Regex::new(r"token limit exceeded").unwrap(),
        Regex::new(r"^4(?:00|13)\s*(?:status code)?\s*\(no body\)").unwrap(),
    ]
});

/// Patterns that indicate non-overflow errors (rate limiting, server errors, etc.).
static NON_OVERFLOW_PATTERNS: Lazy<Vec<Regex>> = Lazy::new(|| {
    vec![
        Regex::new(r"^(Throttling error|Service unavailable):").unwrap(),
        Regex::new(r"rate limit").unwrap(),
        Regex::new(r"too many requests").unwrap(),
    ]
});

fn resolve_context_input_tokens(message: &AssistantMessage) -> Option<i64> {
    if let Some(ctx) = &message.usage.context_usage {
        match ctx {
            llm_core::types::ContextUsage::Available { prompt_tokens, .. } => {
                return Some(*prompt_tokens);
            }
            llm_core::types::ContextUsage::Unavailable => return None,
        }
    }
    Some(message.usage.input + message.usage.cache_read)
}

/// Check if an assistant message represents a context overflow error.
pub fn is_context_overflow(message: &AssistantMessage, context_window: Option<i64>) -> bool {
    if message.stop_reason == "error" {
        if let Some(error_message) = &message.error_message {
            let is_non_overflow = NON_OVERFLOW_PATTERNS.iter().any(|p| p.is_match(error_message));
            if !is_non_overflow
                && OVERFLOW_PATTERNS.iter().any(|p| p.is_match(error_message))
            {
                return true;
            }
        }
    }

    if let Some(cw) = context_window {
        if message.stop_reason == "stop" {
            if let Some(input_tokens) = resolve_context_input_tokens(message) {
                if input_tokens > cw {
                    return true;
                }
            }
        }

        if message.stop_reason == "length" && message.usage.output == 0 {
            if let Some(input_tokens) = resolve_context_input_tokens(message) {
                if input_tokens as f64 >= (cw as f64) * 0.99 {
                    return true;
                }
            }
        }
    }
    false
}