// 翻译自 packages/media-core/src/base64.ts

/** Estimates decoded bytes without allocating a cleaned copy of the base64 payload. */
pub fn estimate_base64_decoded_bytes(base64: &str) -> usize {
    // Avoid `trim()`/`replace()` here: they allocate a second (potentially huge) string.
    // We only need a conservative decoded-size estimate to enforce budgets before Buffer.from(..., "base64").
    let mut effective_len: usize = 0;
    for c in base64.chars() {
        let code = c as u32;
        // Treat ASCII control + space as whitespace; base64 decoders commonly ignore these.
        if code <= 0x20 {
            continue;
        }
        effective_len += 1;
    }

    if effective_len == 0 {
        return 0;
    }

    let mut padding: usize = 0;
    // Find last non-whitespace char(s) to detect '=' padding without allocating/copying.
    let chars: Vec<char> = base64.chars().collect();
    let mut end: i64 = (chars.len() as i64) - 1;
    while end >= 0 && (chars[end as usize] as u32) <= 0x20 {
        end -= 1;
    }
    if end >= 0 && chars[end as usize] == '=' {
        padding = 1;
        end -= 1;
        while end >= 0 && (chars[end as usize] as u32) <= 0x20 {
            end -= 1;
        }
        if end >= 0 && chars[end as usize] == '=' {
            padding = 2;
        }
    }

    let estimated = (effective_len * 3) / 4;
    estimated.saturating_sub(padding)
}

const CANONICALIZE_BASE64_CHUNK_SIZE: usize = 8192;

fn is_base64_data_char(code: u32) -> bool {
    (code >= 0x41 && code <= 0x5a)
        || (code >= 0x61 && code <= 0x7a)
        || (code >= 0x30 && code <= 0x39)
        || code == 0x2b
        || code == 0x2f
}

/**
 * Normalizes and validates a base64 string, returning canonical no-whitespace
 * base64 only when the input has valid alphabet, padding, and length.
 */
pub fn canonicalize_base64(base64: &str) -> Option<String> {
    let mut chunks: Vec<String> = Vec::new();
    let mut current = String::new();
    let mut cleaned_length: usize = 0;
    let mut padding: usize = 0;
    let mut saw_padding = false;

    let append = |char_str: &str, current: &mut String, cleaned_length: &mut usize, chunks: &mut Vec<String>| {
        current.push_str(char_str);
        *cleaned_length += 1;
        if current.len() >= CANONICALIZE_BASE64_CHUNK_SIZE {
            chunks.push(std::mem::take(current));
        }
    };

    let chars: Vec<char> = base64.chars().collect();
    for i in 0..chars.len() {
        let code = chars[i] as u32;
        if code <= 0x20 {
            continue;
        }
        if code == 0x3d {
            padding += 1;
            if padding > 2 {
                return None;
            }
            saw_padding = true;
            append("=", &mut current, &mut cleaned_length, &mut chunks);
            continue;
        }
        if saw_padding || !is_base64_data_char(code) {
            return None;
        }
        let c = chars[i].to_string();
        append(&c, &mut current, &mut cleaned_length, &mut chunks);
    }
    if cleaned_length == 0 {
        return None;
    }
    let remainder = cleaned_length % 4;
    if remainder != 0 {
        if saw_padding || remainder == 1 {
            return None;
        }
        let pad_str: String = std::iter::repeat('=').take(4 - remainder).collect();
        current.push_str(&pad_str);
    }
    if !current.is_empty() {
        chunks.push(current);
    }
    Some(chunks.join(""))
}