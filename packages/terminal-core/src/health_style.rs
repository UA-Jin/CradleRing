// Terminal Core module implements health style behavior.
// 翻译自 packages/terminal-core/src/health-style.ts

use crate::string::normalize_lowercase_string_or_empty;
use crate::theme::theme;

/// Highlight known health status prefixes in a "label: detail" line.
pub fn style_health_channel_line(line: &str, rich: bool) -> String {
    if !rich {
        return line.to_string();
    }
    let colon = match line.find(':') {
        Some(c) => c,
        None => return line.to_string(),
    };
    let label = &line[..=colon];
    let detail_with_lead = &line[colon + 1..];
    let detail = detail_with_lead.trim_start();
    let normalized = normalize_lowercase_string_or_empty(detail);
    let t = theme();

    let apply_prefix = |prefix: &str, color: &dyn crate::theme::Colorizer| -> String {
        let end = prefix.len();
        let styled = color.hex(&detail[..end.min(detail.len())]);
        let rest = if end < detail.len() { &detail[end..] } else { "" };
        format!("{} {}{}", label, styled, rest)
    };

    if normalized.starts_with("failed") {
        return apply_prefix("failed", &*t.error);
    }
    if normalized.starts_with("ok") {
        return apply_prefix("ok", &*t.success);
    }
    if normalized.starts_with("linked") {
        return apply_prefix("linked", &*t.success);
    }
    if normalized.starts_with("configured") {
        return apply_prefix("configured", &*t.success);
    }
    if normalized.starts_with("not linked") {
        return apply_prefix("not linked", &*t.warn);
    }
    if normalized.starts_with("not configured") {
        return apply_prefix("not configured", &*t.muted);
    }
    if normalized.starts_with("unknown") {
        return apply_prefix("unknown", &*t.warn);
    }

    line.to_string()
}
