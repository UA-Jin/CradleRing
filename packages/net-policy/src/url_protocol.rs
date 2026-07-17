// Network Policy module implements url protocol behavior.
// 翻译自 packages/net-policy/src/url-protocol.ts

const HTTP_URL_PREFIX_RE: &str = r"^https?://";

/// Returns true when the value starts with `http://` or `https://` (case-insensitive).
pub fn has_http_url_prefix(value: &str) -> bool {
    case_insensitive_prefix_match(value, "http://")
        || case_insensitive_prefix_match(value, "https://")
}

fn case_insensitive_prefix_match(value: &str, prefix: &str) -> bool {
    if value.len() < prefix.len() {
        return false;
    }
    value[..prefix.len()].eq_ignore_ascii_case(prefix)
}

fn parse_url(value: &str) -> Option<url::Url> {
    url::Url::parse(value).ok()
}

/// Returns true when the value is an HTTP or HTTPS URL.
pub fn is_http_url(value: &str) -> bool {
    match parse_url(value) {
        Some(url) => url.scheme() == "http" || url.scheme() == "https",
        None => false,
    }
}

/// Returns true when the value is an HTTPS URL.
pub fn is_https_url(value: &str) -> bool {
    parse_url(value)
        .map(|url| url.scheme() == "https")
        .unwrap_or(false)
}

/// Returns true when the value is a WebSocket URL (ws:// or wss://).
pub fn is_web_socket_url(value: &str) -> bool {
    match parse_url(value) {
        Some(url) => url.scheme() == "ws" || url.scheme() == "wss",
        None => false,
    }
}

// Required to keep the constant referenced (matches TS module export).
#[allow(dead_code)]
const _HTTP_URL_PREFIX_RE: &str = HTTP_URL_PREFIX_RE;