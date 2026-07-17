// Terminal Core module implements links behavior.
// 翻译自 packages/terminal-core/src/links.ts

use crate::terminal_link::{format_terminal_link, FormatTerminalLinkOptions};

fn resolve_docs_root() -> &'static str {
    "https://docs.cradle-ring.ai"
}

pub fn format_docs_link(
    path: Option<&str>,
    label: Option<&str>,
    opts: Option<DocsLinkOptions>,
) -> String {
    let docs_root = resolve_docs_root();
    let trimmed = path.map(|p| p.trim()).unwrap_or("");
    // When a caller has no docsPath, link to the docs root rather than crashing
    // the onboarding/channel-selection flows that pass meta.docsPath through
    // here unguarded.
    let url = if !trimmed.is_empty() {
        let lower = trimmed.to_lowercase();
        if lower.starts_with("http://") || lower.starts_with("https://") {
            trimmed.to_string()
        } else {
            let prefix = if trimmed.starts_with('/') { "" } else { "/" };
            format!("{}{}{}", docs_root, prefix, trimmed)
        }
    } else {
        docs_root.to_string()
    };
    let fallback = opts.as_ref().and_then(|o| o.fallback.clone());
    let format_opts = FormatTerminalLinkOptions {
        fallback,
        force: opts.as_ref().and_then(|o| o.force),
    };
    let display_label = label.unwrap_or(&url);
    format_terminal_link(display_label, &url, Some(format_opts))
}

#[derive(Default, Clone, Debug)]
pub struct DocsLinkOptions {
    pub fallback: Option<String>,
    pub force: Option<bool>,
}
