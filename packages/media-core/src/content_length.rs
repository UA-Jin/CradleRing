// 翻译自 packages/media-core/src/content-length.ts

use regex::Regex;

/** Parses a Content-Length header as a safe integer or rejects malformed values. */
pub fn parse_media_content_length(raw: Option<&str>) -> Result<Option<i64>, String> {
    let raw = match raw {
        Some(r) => r,
        None => return Ok(None),
    };
    let values: Vec<&str> = raw.split(',').map(|v| v.trim()).collect();
    let value = values.first().copied().unwrap_or("");
    // Repeated lengths affect framing, so their trimmed decimal bytes must match.
    // Numeric comparison would wrongly accept ambiguous values such as "05, 5".
    let digit_re = Regex::new(r"^\d+$").unwrap();
    if !digit_re.is_match(value) || values.iter().any(|candidate| *candidate != value) {
        return Err(format!("invalid content-length header: {}", raw));
    }
    let size: i64 = value.parse().map_err(|_| format!("invalid content-length header: {}", raw))?;
    if !(-9007199254740991i64..=9007199254740991i64).contains(&size) {
        return Err(format!("invalid content-length header: {}", raw));
    }
    Ok(Some(size))
}