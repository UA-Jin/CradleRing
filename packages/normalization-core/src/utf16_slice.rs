// Surrogate-safe UTF-16 string slicing helpers.
// 翻译自 packages/normalization-core/src/utf16-slice.ts
//
// Rust 字符串是 UTF-8，但 JS 字符串是 UTF-16。为了 1:1 对齐行为，
// 这里按 UTF-16 码元（code unit）来切片。

fn is_high_surrogate(code_unit: u16) -> bool {
    code_unit >= 0xd800 && code_unit <= 0xdbff
}

fn is_low_surrogate(code_unit: u16) -> bool {
    code_unit >= 0xdc00 && code_unit <= 0xdfff
}

/// 将 Rust 字符串转为 UTF-16 码元序列（对标 JS string.charCodeAt）
fn utf16_codes(s: &str) -> Vec<u16> {
    s.encode_utf16().collect()
}

/// 将 UTF-16 码元序列转回 String
fn from_utf16_codes(codes: &[u16]) -> String {
    String::from_utf16_lossy(codes)
}

/// Slices a UTF-16 string without returning dangling surrogate halves at either edge.
pub fn slice_utf16_safe(input: &str, start: i64, end: Option<i64>) -> String {
    let codes = utf16_codes(input);
    let len = codes.len() as i64;

    let mut from = if start < 0 {
        std::cmp::max(len + start, 0)
    } else {
        std::cmp::min(start, len)
    };
    let mut to = match end {
        None => len,
        Some(e) if e < 0 => std::cmp::max(len + e, 0),
        Some(e) => std::cmp::min(e, len),
    };

    if to <= from {
        return String::new();
    }

    if from > 0 && from < len {
        let code_unit = codes[from as usize];
        if is_low_surrogate(code_unit) && is_high_surrogate(codes[(from - 1) as usize]) {
            from += 1;
        }
    }

    if to > 0 && to < len {
        let code_unit = codes[(to - 1) as usize];
        if is_high_surrogate(code_unit) && to < len && is_low_surrogate(codes[to as usize]) {
            to -= 1;
        }
    }

    if to <= from {
        return String::new();
    }
    from_utf16_codes(&codes[from as usize..to as usize])
}

/// Truncates a UTF-16 string without cutting a surrogate pair in half.
pub fn truncate_utf16_safe(input: &str, max_len: i64) -> String {
    let limit = std::cmp::max(0, max_len);
    let codes = utf16_codes(input);
    if (codes.len() as i64) <= limit {
        return input.to_string();
    }
    slice_utf16_safe(input, 0, Some(limit))
}
