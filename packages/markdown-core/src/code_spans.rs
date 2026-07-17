// Markdown Core module implements code spans behavior.
// 翻译自 packages/markdown-core/src/code-spans.ts
use crate::fences::{scan_fence_spans, FenceScanState, FenceSpan};
use std::sync::Arc;

/// Incremental inline-code scanner state carried across chunk boundaries.
#[derive(Debug, Clone, Default)]
pub struct InlineCodeState {
    /// Whether the current scan is inside an unterminated inline code span.
    pub open: bool,
    /// Backtick run length required to close the current inline code span.
    pub ticks: usize,
}

/// Creates the carry-forward state used when scanning inline code across chunks.
pub fn create_inline_code_state() -> InlineCodeState {
    InlineCodeState {
        open: false,
        ticks: 0,
    }
}

#[derive(Debug, Clone)]
pub struct InlineCodeSpansResult {
    pub spans: Vec<(usize, usize)>,
    pub state: InlineCodeState,
}

#[derive(Clone)]
pub struct CodeSpanIndex {
    /// Inline-code state to carry into the next streamed chunk.
    pub inline_state: InlineCodeState,
    /// Fenced-code state to carry into the next streamed chunk.
    pub fence_state: FenceScanState,
    /// True when an offset is inside fenced code or inline code.
    pub is_inside: Arc<dyn Fn(usize) -> bool + Send + Sync>,
}

/// Builds a lookup for fenced and inline code spans while preserving scanner state.
pub fn build_code_span_index(
    text: &str,
    inline_state: Option<&InlineCodeState>,
    fence_state: Option<&FenceScanState>,
) -> CodeSpanIndex {
    let (fence_spans, next_fence_state) = scan_fence_spans(text, fence_state);
    let start_state = match inline_state {
        Some(s) => InlineCodeState {
            open: s.open,
            ticks: s.ticks,
        },
        None => create_inline_code_state(),
    };
    let (inline_spans, next_inline_state) =
        parse_inline_code_spans(text, &fence_spans, &start_state);

    let fence_spans_clone = fence_spans.clone();
    let inline_spans_clone = inline_spans.clone();
    CodeSpanIndex {
        inline_state: next_inline_state,
        fence_state: next_fence_state,
        is_inside: Arc::new(move |index: usize| {
            is_inside_fence_span(index, &fence_spans_clone)
                || is_inside_inline_span(index, &inline_spans_clone)
        }),
    }
}

impl std::fmt::Debug for CodeSpanIndex {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CodeSpanIndex")
            .field("inline_state", &self.inline_state)
            .field("fence_state", &self.fence_state)
            .finish()
    }
}

pub fn parse_inline_code_spans(
    text: &str,
    fence_spans: &[FenceSpan],
    initial_state: &InlineCodeState,
) -> (Vec<(usize, usize)>, InlineCodeState) {
    let mut spans: Vec<(usize, usize)> = Vec::new();
    let mut open = initial_state.open;
    let mut ticks = initial_state.ticks;
    let mut open_start: usize = if open { 0 } else { usize::MAX };

    let bytes = text.as_bytes();
    let len = bytes.len();
    let mut i = 0usize;
    while i < len {
        if let Some(fence) = find_fence_span_at_inclusive(fence_spans, i) {
            i = fence.end;
            continue;
        }

        if bytes[i] != b'`' {
            i += 1;
            continue;
        }

        let run_start = i;
        let mut run_length = 0usize;
        while i < len && bytes[i] == b'`' {
            run_length += 1;
            i += 1;
        }

        if !open {
            open = true;
            ticks = run_length;
            open_start = run_start;
            continue;
        }

        if run_length == ticks {
            spans.push((open_start, i));
            open = false;
            ticks = 0;
            open_start = usize::MAX;
        }
    }

    if open {
        spans.push((open_start, len));
    }

    (
        spans,
        InlineCodeState { open, ticks },
    )
}

fn find_fence_span_at_inclusive(spans: &[FenceSpan], index: usize) -> Option<&FenceSpan> {
    spans.iter().find(|s| index >= s.start && index < s.end)
}

fn is_inside_fence_span(index: usize, spans: &[FenceSpan]) -> bool {
    spans.iter().any(|s| index >= s.start && index < s.end)
}

fn is_inside_inline_span(index: usize, spans: &[(usize, usize)]) -> bool {
    spans.iter().any(|(s, e)| index >= *s && index < *e)
}