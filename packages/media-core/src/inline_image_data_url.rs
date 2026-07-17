// Media Core module implements inline image data url behavior.
// 翻译自 packages/media-core/src/inline-image-data-url.ts

use crate::base64::canonicalize_base64;

/** Prefix used to distinguish inline data URLs from remote/local image references. */
pub const INLINE_IMAGE_DATA_URL_PREFIX: &str = "data:";

const HEIC_BRANDS: &[&str] = &[
    "heic", "heix", "hevc", "hevx", "heis", "heim", "hevm", "hevs",
];
const HEIF_BRANDS: &[&str] = &["mif1", "msf1"];
const IMAGE_SIGNATURE_PREFIX_BASE64_CHARS: usize = 128;
const INLINE_IMAGE_DATA_URL_MIMES: &[&str] = &[
    "image/png", "image/jpeg", "image/webp", "image/gif",
];

fn starts_with_data_url(value: &str) -> bool {
    if value.len() < INLINE_IMAGE_DATA_URL_PREFIX.len() {
        return false;
    }
    value[..INLINE_IMAGE_DATA_URL_PREFIX.len()].to_lowercase()
        == INLINE_IMAGE_DATA_URL_PREFIX
}

fn sniff_iso_bmff_image_mime(buffer: &[u8]) -> Option<&'static str> {
    if buffer.len() < 12 || buffer[4..8] != *b"ftyp" {
        return None;
    }
    let mut brands: Vec<&[u8]> = vec![&buffer[8..12]];
    let mut offset = 16;
    while offset + 4 <= buffer.len() {
        brands.push(&buffer[offset..offset + 4]);
        offset += 4;
    }
    if brands.iter().any(|b| HEIC_BRANDS.iter().any(|h| h.as_bytes() == *b)) {
        return Some("image/heic");
    }
    if brands.iter().any(|b| HEIF_BRANDS.iter().any(|h| h.as_bytes() == *b)) {
        return Some("image/heif");
    }
    None
}

fn ascii_eq(buffer: &[u8], expected: &[u8]) -> bool {
    buffer.len() >= expected.len() && &buffer[..expected.len()] == expected
}

fn starts_with(buffer: &[u8], prefix: &[u8]) -> bool {
    buffer.len() >= prefix.len() && &buffer[..prefix.len()] == prefix
}

fn starts_with_byte(buffer: &[u8], b: u8) -> bool {
    buffer.first().copied() == Some(b)
}

fn sniff_inline_image_mime(buffer: &[u8]) -> Option<&'static str> {
    if starts_with(buffer, b"\x89PNG\r\n\x1a\n") {
        return Some("image/png");
    }
    if buffer.len() >= 3 && buffer[0] == 0xff && buffer[1] == 0xd8 && buffer[2] == 0xff {
        return Some("image/jpeg");
    }
    if ascii_eq(buffer, b"RIFF") && buffer.len() >= 12 && &buffer[8..12] == b"WEBP" {
        return Some("image/webp");
    }
    if ascii_eq(buffer, b"GIF87a") || ascii_eq(buffer, b"GIF89a") {
        return Some("image/gif");
    }
    if buffer.len() >= 2 && starts_with_byte(buffer, 0x42) && buffer.get(1).copied() == Some(0x4d) {
        return Some("image/bmp");
    }
    sniff_iso_bmff_image_mime(buffer)
}

fn is_image_mime_type(value: &str) -> bool {
    let v = value.trim().to_lowercase();
    v.starts_with("image/")
}

#[derive(Debug, Clone)]
pub struct SanitizedInlineImageBase64 {
    pub mime_type: String,
    pub base64: String,
}

/** Canonicalizes trusted inline image base64 and rejects malformed or non-image payloads. */
pub fn sanitize_inline_image_base64(
    mime_type: &str,
    base64: &str,
) -> Option<SanitizedInlineImageBase64> {
    if !is_image_mime_type(mime_type) {
        return None;
    }
    let canonical_payload = canonicalize_base64(base64)?;
    // Decode the prefix to sniff the actual mime type.
    let prefix: String = canonical_payload
        .chars()
        .take(IMAGE_SIGNATURE_PREFIX_BASE64_CHARS)
        .collect();
    let decoded = base64_decode(&prefix);
    let sniffed = sniff_inline_image_mime(&decoded)?;
    Some(SanitizedInlineImageBase64 {
        mime_type: sniffed.to_string(),
        base64: canonical_payload,
    })
}

fn base64_decode(input: &str) -> Vec<u8> {
    // Use the `base64` crate via internal helper.
    use base64::Engine;
    base64::engine::general_purpose::STANDARD
        .decode(input)
        .unwrap_or_default()
}

struct ParsedInlineImageDataUrl {
    metadata: Vec<String>,
    payload: String,
}

fn parse_inline_image_data_url(value: &str) -> Option<ParsedInlineImageDataUrl> {
    if !starts_with_data_url(value) {
        return Some(ParsedInlineImageDataUrl {
            metadata: Vec::new(),
            payload: value.to_string(),
        });
    }
    let after_prefix = &value[INLINE_IMAGE_DATA_URL_PREFIX.len()..];
    let comma_index = after_prefix.find(',')?;
    let metadata_str = &after_prefix[..comma_index];
    let payload = after_prefix[comma_index + 1..].to_string();
    let metadata: Vec<String> = metadata_str
        .split(';')
        .map(|part| part.trim().to_string())
        .collect();
    Some(ParsedInlineImageDataUrl {
        metadata,
        payload,
    })
}

fn metadata_allows_image_base64(metadata: &[String]) -> bool {
    let (mime_type, options) = match metadata.split_first() {
        Some(t) => t,
        None => return false,
    };
    is_image_mime_type(mime_type)
        && options
            .iter()
            .any(|part| part.to_lowercase() == "base64")
}

fn sanitize_inline_image_data_url_with_allowed_mimes(
    image_url: &str,
    allowed_mimes: Option<&std::collections::HashSet<String>>,
) -> Option<String> {
    let parsed = parse_inline_image_data_url(image_url)?;
    if parsed.metadata.is_empty() {
        return Some(image_url.to_string());
    }
    if !metadata_allows_image_base64(&parsed.metadata) {
        return None;
    }
    let mime_type = parsed.metadata.first().map(|s| s.as_str()).unwrap_or("");
    let sanitized = sanitize_inline_image_base64(mime_type, &parsed.payload)?;
    if let Some(allowed) = allowed_mimes {
        if !allowed.contains(&sanitized.mime_type) {
            return None;
        }
    }
    // Trust the byte signature over caller-supplied metadata before reinlining.
    Some(format!(
        "data:{};base64,{}",
        sanitized.mime_type, sanitized.base64
    ))
}

/**
 * Canonicalizes trusted inline image data URLs for persistence.
 * Accepts every image signature supported by `sanitize_inline_image_base64`.
 */
pub fn sanitize_inline_image_data_url_for_storage(image_url: &str) -> Option<String> {
    sanitize_inline_image_data_url_with_allowed_mimes(image_url, None)
}

/** Canonicalizes provider-safe inline image data URLs and rejects unsupported formats. */
pub fn sanitize_inline_image_data_url(image_url: &str) -> Option<String> {
    let allowed: std::collections::HashSet<String> = INLINE_IMAGE_DATA_URL_MIMES
        .iter()
        .map(|s| s.to_string())
        .collect();
    sanitize_inline_image_data_url_with_allowed_mimes(image_url, Some(&allowed))
}