// Network Policy module implements redact sensitive url behavior.
// 翻译自 packages/net-policy/src/redact-sensitive-url.ts

use std::collections::HashSet;

#[derive(Debug, Clone, Default)]
pub struct ConfigUiHintTags {
    pub tags: Option<Vec<String>>,
}

fn normalize_lowercase_string_or_empty(value: &str) -> String {
    value.trim().to_lowercase()
}

/// Config UI hint tag for URL-like values that may embed credentials or tokens.
pub const SENSITIVE_URL_HINT_TAG: &str = "url-secret";

fn sensitive_url_query_param_names() -> HashSet<&'static str> {
    let mut set = HashSet::new();
    for name in [
        "token",
        "key",
        "api_key",
        "apikey",
        "secret",
        "access_token",
        "auth_token",
        "password",
        "pass",
        "passwd",
        "auth",
        "jwt",
        "session",
        "id_token",
        "code",
        "client_secret",
        "app_secret",
        "hook_token",
        "refresh_token",
        "signature",
        "x_amz_signature",
        "x_amz_security_token",
        "private_key",
        "credential",
        "authorization",
    ] {
        set.insert(name);
    }
    set
}

const SENSITIVE_URL_QUERY_PARAM_NAMES: std::sync::LazyLock<HashSet<&'static str>> =
    std::sync::LazyLock::new(sensitive_url_query_param_names);

/// Hangul fillers (category Lo) and other separators that may splice sensitive key names.
fn is_query_separator_byte(b: char) -> bool {
    let code = b as u32;
    if code <= 0x1f || code == 0x7f {
        return true;
    }
    if code == 0x20 {
        return true;
    }
    matches!(
        code,
        0x115F | 0x1160 | 0x3164 | 0xFFA0 | 0x2B // '+'
    )
}

fn strip_query_separators(s: &str) -> String {
    s.chars().filter(|c| !is_query_separator_byte(*c)).collect()
}

/// True for auth-like URL query parameter names that should be redacted.
pub fn is_sensitive_url_query_param_name(name: &str) -> bool {
    let normalized = normalize_url_query_param_name(name);
    SENSITIVE_URL_QUERY_PARAM_NAMES.contains(normalized.as_str())
}

fn normalize_url_query_param_name(name: &str) -> String {
    let stripped = strip_query_separators(name);
    let decoded = percent_decode(&stripped).unwrap_or(stripped);
    let decoded_stripped = strip_query_separators(&decoded);
    let lowered = normalize_lowercase_string_or_empty(&decoded_stripped);
    lowered.replace('-', "_")
}

fn percent_decode(value: &str) -> Option<String> {
    let bytes = value.as_bytes();
    let mut out: Vec<u8> = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        let b = bytes[i];
        if b == b'%' && i + 2 < bytes.len() {
            let hi = hex_value(bytes[i + 1])?;
            let lo = hex_value(bytes[i + 2])?;
            out.push((hi << 4) | lo);
            i += 3;
        } else {
            out.push(b);
            i += 1;
        }
    }
    String::from_utf8(out).ok()
}

fn hex_value(b: u8) -> Option<u8> {
    match b {
        b'0'..=b'9' => Some(b - b'0'),
        b'a'..=b'f' => Some(b - b'a' + 10),
        b'A'..=b'F' => Some(b - b'A' + 10),
        _ => None,
    }
}

/// True for config paths whose URL values may contain credentials or secret query params.
pub fn is_sensitive_url_config_path(path: &str) -> bool {
    if path.ends_with(".baseUrl") || path.ends_with(".httpUrl") {
        return true;
    }
    if path.ends_with(".cdpUrl") {
        return true;
    }
    if path.ends_with(".request.proxy.url") {
        return true;
    }
    mcp_server_url_re().is_match(path)
}

fn mcp_server_url_re() -> &'static regex::Regex {
    use std::sync::OnceLock;
    static RE: OnceLock<regex::Regex> = OnceLock::new();
    RE.get_or_init(|| {
        regex::Regex::new(r"^(?:nodeHost\.)?mcp\.servers\.(?:\*|[^.]+)\.url$").unwrap()
    })
}

/// True when a config UI hint explicitly marks a URL-like value as secret-bearing.
pub fn has_sensitive_url_hint_tag(hint: Option<&ConfigUiHintTags>) -> bool {
    hint.and_then(|h| h.tags.as_ref())
        .map(|tags| tags.iter().any(|t| t == SENSITIVE_URL_HINT_TAG))
        .unwrap_or(false)
}

/// Redacts credentials and sensitive query params from parseable URLs.
pub fn redact_sensitive_url(value: &str) -> String {
    match url::Url::parse(value) {
        Ok(mut parsed) => {
            let mut mutated = false;
            let redacted_path = redact_sensitive_url_path(parsed.path());
            if redacted_path != parsed.path() {
                parsed.set_path(&redacted_path);
                mutated = true;
            }
            let username = parsed.username().to_string();
            let password = parsed.password().unwrap_or("").to_string();
            if !username.is_empty() || !password.is_empty() {
                if !username.is_empty() {
                    parsed.set_username("***").ok();
                }
                if !password.is_empty() {
                    parsed.set_password(Some("***")).ok();
                }
                mutated = true;
            }
            let keys: Vec<String> = parsed
                .query_pairs()
                .map(|(k, _)| k.into_owned())
                .collect();
            for key in keys {
                if is_sensitive_url_query_param_name(&key) {
                    let pairs: Vec<(String, String)> = parsed
                        .query_pairs()
                        .map(|(k, v)| (k.into_owned(), v.into_owned()))
                        .collect();
                    let mut serializer = parsed.query_pairs_mut();
                    serializer.clear();
                    for (k, v) in pairs {
                        if k == key {
                            serializer.append_pair(&k, "***");
                        } else {
                            serializer.append_pair(&k, &v);
                        }
                    }
                    mutated = true;
                }
            }
            if mutated {
                parsed.to_string()
            } else {
                value.to_string()
            }
        }
        Err(_) => value.to_string(),
    }
}

fn redact_sensitive_url_path(value: &str) -> String {
    // Telegram bot token path: /bot<token>/...
    let mut result = String::with_capacity(value.len());
    let chars: Vec<char> = value.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        if i + 4 <= chars.len() && &chars[i..i + 4] == ['/', 'b', 'o', 't'] {
            // Try to match: /bot<digits>[:|%3a|%3A][A-Za-z0-9_-]{20,}(/|$)
            let mut j = i + 4;
            while j < chars.len() && chars[j].is_ascii_digit() {
                j += 1;
            }
            if j > i + 4 && j < chars.len() {
                let after_digits = chars[j];
                if after_digits == ':' {
                    let start_token = j + 1;
                    let end_token = scan_token_chars(&chars, start_token);
                    if end_token - start_token >= 20 && end_token < chars.len() {
                        let next = chars[end_token];
                        if next == '/' || next == '?' || next == '#' || next.is_whitespace() {
                            result.push_str("/bot***");
                            i = end_token;
                            continue;
                        }
                    }
                } else if after_digits == '%' && j + 2 < chars.len() && chars[j + 1] == '3' {
                    let c = chars[j + 2];
                    if c == 'a' || c == 'A' {
                        let start_token = j + 3;
                        let end_token = scan_token_chars(&chars, start_token);
                        if end_token - start_token >= 20 && end_token < chars.len() {
                            let next = chars[end_token];
                            if next == '/' || next == '?' || next == '#' || next.is_whitespace() {
                                result.push_str("/bot***");
                                i = end_token;
                                continue;
                            }
                        }
                    }
                }
            }
        }
        result.push(chars[i]);
        i += 1;
    }
    result
}

fn scan_token_chars(chars: &[char], start: usize) -> usize {
    let mut j = start;
    while j < chars.len() && (chars[j].is_ascii_alphanumeric() || chars[j] == '_' || chars[j] == '-')
    {
        j += 1;
    }
    j
}

/// Redacts sensitive URL-looking substrings even when the full value is not a valid URL.
pub fn redact_sensitive_url_like_string(value: &str) -> String {
    let redacted_url = redact_sensitive_url(value);
    if redacted_url != value {
        return redacted_url;
    }
    let redacted_fallback = redact_userinfo_fallback(value);
    redact_sensitive_url_path(&redacted_fallback)
}

fn redact_userinfo_fallback(value: &str) -> String {
    let mut result = String::with_capacity(value.len());
    let bytes = value.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if i + 2 < bytes.len() && bytes[i] == b'/' && bytes[i + 1] == b'/' {
            // Match userinfo pattern: //([^@/?#\s]+)@
            let mut j = i + 2;
            let userinfo_start = j;
            while j < bytes.len() {
                let c = bytes[j];
                if c == b'@' || c == b'/' || c == b'?' || c == b'#' || c.is_ascii_whitespace() {
                    break;
                }
                j += 1;
            }
            if j < bytes.len() && bytes[j] == b'@' && j > userinfo_start {
                result.push_str("//***:***@");
                i = j + 1;
                continue;
            }
        }
        // Match query/anchor key=value pattern (handles ?key=value and &key=value)
        if (bytes[i] == b'?' || bytes[i] == b'&') && i + 1 < bytes.len() {
            let prefix = bytes[i];
            let key_start = i + 1;
            let mut key_end = key_start;
            while key_end < bytes.len() && bytes[key_end] != b'=' && bytes[key_end] != b'&' {
                key_end += 1;
            }
            if key_end < bytes.len() && bytes[key_end] == b'=' {
                let key_str = &value[key_start..key_end];
                if is_sensitive_url_query_param_name(key_str) {
                    result.push(prefix as char);
                    result.push_str(key_str);
                    result.push_str("=***");
                    i = key_end + 1;
                    // skip value
                    while i < bytes.len() && bytes[i] != b'&' {
                        i += 1;
                    }
                    continue;
                }
            }
        }
        result.push(value[i..].chars().next().unwrap());
        i += value[i..].chars().next().unwrap().len_utf8();
    }
    result
}