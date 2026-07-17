// Media Core module implements media source url behavior.
// 翻译自 packages/media-core/src/media-source-url.ts

use regex::Regex;
use std::sync::OnceLock;

fn http_url_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"^https?://").unwrap())
}

fn mxc_url_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"^mxc://").unwrap())
}

fn buffer_url_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"^buffer://").unwrap())
}

/** Returns true for remote media URLs that should stay URL-backed instead of local-file-backed. */
pub fn is_pass_through_remote_media_source(value: Option<&str>) -> bool {
    let normalized = value.map(|s| s.trim()).unwrap_or("");
    !normalized.is_empty()
        && (http_url_re().is_match(normalized)
            || mxc_url_re().is_match(normalized)
            || buffer_url_re().is_match(normalized))
}