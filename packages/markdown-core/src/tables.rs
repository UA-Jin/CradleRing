// Markdown Core module implements tables behavior.
// 翻译自 packages/markdown-core/src/tables.ts
use crate::ir::{markdown_to_ir_with_meta, MarkdownIR, MarkdownLinkSpan, MarkdownStyle};
use crate::render::{
    render_markdown_with_markers, RenderLink, RenderOptions, RenderStyleMap, RenderStyleMarker,
    RenderStyleOpen,
};
use crate::types::MarkdownTableMode;

fn markdown_style_markers() -> RenderStyleMap {
    let mut m = RenderStyleMap::default();
    m.bold = Some(RenderStyleMarker {
        open: RenderStyleOpen::Static("**".to_string()),
        close: "**".to_string(),
    });
    m.italic = Some(RenderStyleMarker {
        open: RenderStyleOpen::Static("_".to_string()),
        close: "_".to_string(),
    });
    m.strikethrough = Some(RenderStyleMarker {
        open: RenderStyleOpen::Static("~~".to_string()),
        close: "~~".to_string(),
    });
    m.code = Some(RenderStyleMarker {
        open: RenderStyleOpen::Static("`".to_string()),
        close: "`".to_string(),
    });
    m.code_block = Some(RenderStyleMarker {
        open: RenderStyleOpen::Static("```\n".to_string()),
        close: "```".to_string(),
    });
    m
}

/// Converts markdown tables into the configured plaintext/code rendering mode.
pub fn convert_markdown_tables(markdown: &str, mode: MarkdownTableMode) -> String {
    if markdown.is_empty() || mode == MarkdownTableMode::Off {
        return markdown.to_string();
    }
    let effective_mode = if mode == MarkdownTableMode::Block {
        MarkdownTableMode::Code
    } else {
        mode
    };

    let result = markdown_to_ir_with_meta(
        markdown,
        crate::ir::MarkdownParseOptions {
            linkify: Some(false),
            enable_spoilers: None,
            heading_style: Some(crate::ir::HeadingStyle::None),
            blockquote_prefix: Some(String::new()),
            autolink: Some(false),
            table_mode: Some(effective_mode),
        },
    );

    if !result.has_tables {
        return markdown.to_string();
    }

    render_markdown_with_markers(
        &result.ir,
        &RenderOptions {
            style_markers: markdown_style_markers(),
            escape_text: |t: &str| t.to_string(),
            build_link: Some(|link: &MarkdownLinkSpan, text: &str| -> Option<RenderLink> {
                let href = link.href.trim();
                if href.is_empty() {
                    return None;
                }
                let label = &text[link.start..link.end];
                if label.is_empty() {
                    return None;
                }
                Some(RenderLink {
                    start: link.start,
                    end: link.end,
                    open: "[".to_string(),
                    close: format!("]({})", href),
                })
            }),
        },
    )
}

#[allow(dead_code)]
fn style_enum_marker_check(s: MarkdownStyle) -> &'static str {
    s.as_str()
}

#[allow(dead_code)]
fn ir_default_check() -> MarkdownIR {
    MarkdownIR::default()
}