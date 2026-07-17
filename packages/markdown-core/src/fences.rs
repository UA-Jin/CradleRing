// Markdown Core module implements fences behavior.
// 翻译自 packages/markdown-core/src/fences.ts
use once_cell::sync::Lazy;
use regex::Regex;

/// Markdown fenced-code block span with the opener data needed to reopen it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FenceSpan {
    pub start: usize,
    pub end: usize,
    pub open_line: String,
    pub marker: String,
    pub indent: String,
}

#[derive(Debug, Clone, Default)]
pub struct OpenFence {
    pub marker_char: String,
    pub marker_len: usize,
    pub open_line: String,
    pub marker: String,
    pub indent: String,
}

/// Streaming fence scanner state carried across partial markdown chunks.
#[derive(Debug, Clone, Default)]
pub struct FenceScanState {
    pub at_line_start: bool,
    pub open: Option<OpenFence>,
}

static FENCE_LINE_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^( {0,3})(`{3,}|~{3,})(.*)$").unwrap());

static SPACES_TABS_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"^[ \t]*$").unwrap());

/// Scans fenced-code spans incrementally so chunking can carry an open fence forward.
pub fn scan_fence_spans(
    buffer: &str,
    state: Option<&FenceScanState>,
) -> (Vec<FenceSpan>, FenceScanState) {
    let mut spans: Vec<FenceSpan> = Vec::new();
    let starts_at_line_start = state.map(|s| s.at_line_start).unwrap_or(true);
    let mut open: Option<(usize, OpenFence)> = state
        .as_ref()
        .and_then(|s| s.open.as_ref().map(|o| (0usize, o.clone())));

    // Mirror offset position in bytes through the buffer.
    let bytes = buffer.as_bytes();
    let len = buffer.len();
    let mut offset = 0usize;
    while offset <= len {
        // Find next newline (byte index).
        let mut nl = len;
        for i in offset..len {
            if bytes[i] == b'\n' {
                nl = i;
                break;
            }
        }
        let line_end = nl;
        let line_raw = &buffer[offset..line_end];
        let line = line_raw.strip_suffix('\r').unwrap_or(line_raw);

        if let Some(caps) = FENCE_LINE_RE.captures(line) {
            if offset > 0 || starts_at_line_start {
                let indent = caps.get(1).map(|m| m.as_str()).unwrap_or("");
                let marker = caps.get(2).map(|m| m.as_str()).unwrap_or("");
                let trailing = caps.get(3).map(|m| m.as_str()).unwrap_or("");
                let marker_char: String = marker.chars().next().map(|c| c.to_string()).unwrap_or_default();
                let marker_len = marker.len();

                if open.is_none() {
                    open = Some((
                        offset,
                        OpenFence {
                            marker_char: marker_char.clone(),
                            marker_len,
                            open_line: line.to_string(),
                            marker: marker.to_string(),
                            indent: indent.to_string(),
                        },
                    ));
                } else {
                    let cur = open.as_ref().unwrap();
                    let matches_marker = cur.1.marker_char == marker_char
                        && marker_len >= cur.1.marker_len
                        && SPACES_TABS_RE.is_match(trailing);
                    if matches_marker {
                        let cur = open.take().unwrap();
                        spans.push(FenceSpan {
                            start: cur.0,
                            end: line_end,
                            open_line: cur.1.open_line,
                            marker: cur.1.marker,
                            indent: cur.1.indent,
                        });
                    }
                }
            }
        }

        if nl == len {
            break;
        }
        offset = nl + 1;
    }

    if let Some((start, o)) = open.take() {
        spans.push(FenceSpan {
            start,
            end: len,
            open_line: o.open_line,
            marker: o.marker,
            indent: o.indent,
        });
    }

    let at_line_start = if len == 0 {
        starts_at_line_start
    } else {
        buffer.ends_with('\n')
    };
    let next_state = FenceScanState {
        at_line_start,
        open: open.map(|(_, o)| o),
    };
    (spans, next_state)
}

/// Parses all fenced-code spans in a complete markdown buffer.
pub fn parse_fence_spans(buffer: &str) -> Vec<FenceSpan> {
    scan_fence_spans(buffer, None).0
}

/// Looks up the fence containing an offset; spans must be sorted by start offset.
pub fn find_fence_span_at(spans: &[FenceSpan], index: usize) -> Option<&FenceSpan> {
    let mut low = 0isize;
    let mut high = spans.len() as isize - 1;

    while low <= high {
        let mid = (low + high) / 2;
        let span = spans.get(mid as usize)?;
        if index <= span.start {
            high = mid - 1;
            continue;
        }
        if index >= span.end {
            low = mid + 1;
            continue;
        }
        return Some(span);
    }
    None
}

/// True when a chunk boundary would not split a fenced-code block.
pub fn is_safe_fence_break(spans: &[FenceSpan], index: usize) -> bool {
    find_fence_span_at(spans, index).is_none()
}