// Markdown Core module implements chunk text behavior.
// 翻译自 packages/markdown-core/src/chunk-text.ts

fn resolve_chunk_early_return(text: &str, limit: usize) -> Option<Vec<String>> {
    if text.is_empty() {
        return Some(vec![]);
    }
    if limit == 0 {
        return Some(vec![text.to_string()]);
    }
    if text.chars().count() <= limit {
        return Some(vec![text.to_string()]);
    }
    None
}

fn scan_paren_aware_breakpoints(text: &str) -> (i64, i64) {
    let mut last_newline: i64 = -1;
    let mut last_whitespace: i64 = -1;
    let mut depth: i32 = 0;

    let chars: Vec<char> = text.chars().collect();
    for (i, &c) in chars.iter().enumerate() {
        if c == '(' {
            depth += 1;
            continue;
        }
        if c == ')' && depth > 0 {
            depth -= 1;
            continue;
        }
        if depth != 0 {
            continue;
        }
        if c == '\n' {
            last_newline = i as i64;
        } else if c.is_whitespace() {
            last_whitespace = i as i64;
        }
    }

    (last_newline, last_whitespace)
}

/**
 * Keeps UTF-16 chunk boundaries from separating a supplementary-plane character.
 * A one-unit positive limit still needs to emit an entire surrogate pair.
 *
 * Rust strings are UTF-8, but the JS surrogate-pair logic translates to
 * ensuring we never split a multi-byte UTF-8 character at a code-point
 * boundary that leaves a partial sequence behind.
 */
pub fn avoid_trailing_high_surrogate_break(text: &str, start: usize, end: usize) -> usize {
    let len = text.len();
    if end >= len {
        return end;
    }
    // Check if the byte at `end` is a UTF-8 continuation byte, meaning
    // `end - 1` cuts a multi-byte character.
    if let Some(b) = text.as_bytes().get(end) {
        if (0x80..=0xBF).contains(b) {
            // The byte at `end` is a continuation byte, meaning
            // end-1 splits a codepoint. Step back to before the start of that codepoint.
            let mut safe = end - 1;
            while safe > start && (text.as_bytes()[safe] & 0xC0) == 0x80 {
                safe -= 1;
            }
            if safe > start {
                return safe;
            }
            return end;
        }
    }
    end
}

/**
 * Splits plain text into size-bounded chunks at readable boundaries.
 *
 * Returns the original text as one chunk when the limit is non-positive.
 */
pub fn chunk_text(text: &str, limit: usize) -> Vec<String> {
    if let Some(early) = resolve_chunk_early_return(text, limit) {
        return early;
    }

    let chars: Vec<char> = text.chars().collect();
    let total = chars.len();
    let mut chunks: Vec<String> = Vec::new();
    let mut cursor: usize = 0;
    while cursor < total {
        if total - cursor <= limit {
            chunks.push(chars[cursor..].iter().collect());
            break;
        }
        let window_end = std::cmp::min(total, cursor + limit);
        let window: String = chars[cursor..window_end].iter().collect();
        let (last_newline, last_whitespace) = scan_paren_aware_breakpoints(&window);
        // Prefer block boundaries, then spaces, then a hard size cut when no
        // readable breakpoint exists inside this window.
        let break_offset = if last_newline > 0 { last_newline } else { last_whitespace };
        let raw_end = if break_offset > 0 {
            cursor + break_offset as usize
        } else {
            window_end
        };
        // Convert char-based end back to byte boundary safely.
        let byte_cursor: usize = chars[..cursor].iter().map(|c| c.len_utf8()).sum();
        let byte_raw_end: usize = chars[..raw_end].iter().map(|c| c.len_utf8()).sum();
        let end = avoid_trailing_high_surrogate_break(text, byte_cursor, byte_raw_end);
        let byte_chunks: String = text[byte_cursor..end].to_string();
        chunks.push(byte_chunks);
        // advance cursor to char index of `end`
        let mut new_cursor_byte = end;
        let mut new_cursor = cursor;
        while new_cursor < total {
            let clen = chars[new_cursor].len_utf8();
            if new_cursor_byte <= clen {
                break;
            }
            new_cursor_byte -= clen;
            new_cursor += 1;
        }
        cursor = new_cursor;
        while cursor < total && chars[cursor].is_whitespace() {
            cursor += 1;
        }
    }
    chunks
}