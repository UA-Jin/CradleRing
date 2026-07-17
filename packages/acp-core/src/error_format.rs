// ACP Core helper module supports error format behavior.
// 翻译自 packages/acp-core/src/error-format.ts

use std::sync::Mutex;

use once_cell::sync::Lazy;
use regex::Regex;

const SECRET_PATTERNS: &[&str] = &[
    r#"\b[A-Z0-9_]*(?:KEY|TOKEN|SECRET|PASSWORD|PASSWD|CARD[_-]?NUMBER|CARD[_-]?CVC|CARD[_-]?CVV|CVC|CVV|SECURITY[_-]?CODE|PAYMENT[_-]?CREDENTIAL|SHARED[_-]?PAYMENT[_-]?TOKEN)\b\s*[=:]\s*(["']?)([^\s"'\\]+)\1"#,
    r#"\b[A-Z0-9_]*(?:KEY|TOKEN|SECRET|PASSWORD|PASSWD|CARD[_-]?NUMBER|CARD[_-]?CVC|CARD[_-]?CVV|CVC|CVV|SECURITY[_-]?CODE|PAYMENT[_-]?CREDENTIAL|SHARED[_-]?PAYMENT[_-]?TOKEN)\b\s*[=:]\s*\\+(["'])([^\s"'\\]+)\\+\1"#,
    r#"[?&](?:access[-_]?token|auth[-_]?token|hook[-_]?token|refresh[-_]?token|api[-_]?key|client[-_]?secret|token|key|secret|password|pass|passwd|auth|signature|card[-_]?number|card[-_]?cvc|card[-_]?cvv|cvc|cvv|security[-_]?code|payment[-_]?credential|shared[-_]?payment[-_]?token)=([^&\s"'<>]+)"#,
    r#""(?:apiKey|token|secret|password|passwd|accessToken|refreshToken|cardNumber|card_number|cardCvc|card_cvc|cardCvv|card_cvv|cvc|cvv|securityCode|security_code|paymentCredential|payment_credential|sharedPaymentToken|shared_payment_token)"\s*:\s*"([^"]+)""#,
    r#"(^|[\s,{])["']?(?:api[-_]key|access[-_]token|refresh[-_]token|authToken|auth[-_]?token|clientSecret|client[-_]secret|appSecret|app[-_]secret)["']?\s*[:=]\s*(["'])([^"'\r\n]+)\2"#,
    r#"(^|[\s,{])["']?(?:authorization|proxy-authorization|cookie|set-cookie|x-api-key|x-auth-token)["']?\s*[:=]\s*(["'])([^"'\r\n]+)\2"#,
    r#"--(?:api[-_]?key|hook[-_]?token|token|secret|password|passwd|card[-_]?number|card[-_]?cvc|card[-_]?cvv|cvc|cvv|security[-_]?code|payment[-_]?credential|shared[-_]?payment[-_]?token)\s+(["']?)([^\s"']+)\1"#,
    r#"Authorization\s*[:=]\s*Bearer\s+([A-Za-z0-9._\-+=]+)"#,
    r#"Authorization\s*[:=]\s*Basic\s+([A-Za-z0-9+/=]+)"#,
    r#"(?:X-CradleRing-Token|x-pomerium-jwt-assertion|X-Api-Key|X-Auth-Token)\s*[:=]\s*([^\s"',;]+)"#,
    r#"\bBearer\s+([A-Za-z0-9._\-+=]{18,})\b"#,
    r#"(^|[\s,;])(?:access_token|refresh_token|auth[-_]?token|api[-_]?key|client[-_]?secret|app[-_]?secret|token|secret|password|passwd|card[-_]?number|card[-_]?cvc|card[-_]?cvv|cvc|cvv|security[-_]?code|payment[-_]?credential|shared[-_]?payment[-_]?token)=([^\s&#]+)"#,
    r#"-----BEGIN [A-Z ]*PRIVATE KEY-----[\s\S]+?-----END [A-Z ]*PRIVATE KEY-----"#,
    r#"\b(sk-[A-Za-z0-9_-]{8,})\b"#,
    r#"(ghp_[A-Za-z0-9]{20,})"#,
    r#"(github_pat_[A-Za-z0-9_]{20,})"#,
    r#"(xox[baprs]-[A-Za-z0-9-]{10,})"#,
    r#"(xapp-[A-Za-z0-9-]{10,})"#,
    r#"(gsk_[A-Za-z0-9_-]{10,})"#,
    r#"(AIza[0-9A-Za-z\-_]{20,})"#,
    r#"(ya29\.[0-9A-Za-z_\-./+=]{10,})"#,
    r#"(1\/\/0[0-9A-Za-z_\-./+=]{10,})"#,
    r#"(eyJ[A-Za-z0-9_-]{10,}\.[A-Za-z0-9_-]{10,}\.[A-Za-z0-9_-]{10,})"#,
    r#"(pplx-[A-Za-z0-9_-]{10,})"#,
    r#"(npm_[A-Za-z0-9]{10,})"#,
    r#"(AKID[A-Za-z0-9]{10,})"#,
    r#"(LTAI[A-Za-z0-9]{10,})"#,
    r#"(hf_[A-Za-z0-9]{10,})"#,
    r#"(r8_[A-Za-z0-9_-]{10,})"#,
    r#"\bbot(\d{6,}:[A-Za-z0-9_-]{20,})\b"#,
    r#"\b(\d{6,}:[A-Za-z0-9_-]{20,})\b"#,
];

static SECRET_PATTERN_REGEXES: Lazy<Vec<Regex>> = Lazy::new(|| {
    SECRET_PATTERNS
        .iter()
        .filter_map(|p| {
            let opts = if p.contains("(?i)") {
                ""
            } else {
                "(?i)"
            };
            Regex::new(&format!("{}{}", opts, p)).ok()
        })
        .collect()
});

static CONFIGURED_REDACTOR: Mutex<Option<Box<dyn Fn(&str) -> String + Send + Sync>>> =
    Mutex::new(None);

/// Installs a host-provided redactor used before ACP fallback secret-pattern redaction.
pub fn configure_acp_error_redactor<F>(redactor: Option<F>)
where
    F: Fn(&str) -> String + Send + Sync + 'static,
{
    let mut guard = CONFIGURED_REDACTOR.lock().unwrap();
    *guard = redactor.map(|f| Box::new(f) as Box<dyn Fn(&str) -> String + Send + Sync>);
}

/// Redacts common provider, GitHub, HTTP, payment, bot, and private-key secrets from error text.
pub fn redact_sensitive_text(value: &str) -> String {
    let mut redacted: String = {
        let guard = CONFIGURED_REDACTOR.lock().unwrap();
        match guard.as_ref() {
            Some(f) => f(value),
            None => value.to_string(),
        }
    };
    for pattern in SECRET_PATTERN_REGEXES.iter() {
        let mut result = String::new();
        let mut last_end = 0;
        for caps in pattern.captures_iter(&redacted) {
            let m = caps.get(0).unwrap();
            result.push_str(&redacted[last_end..m.start()]);
            let matched = m.as_str();
            if matched.contains("PRIVATE KEY-----") {
                result.push_str("[REDACTED_PRIVATE_KEY]");
            } else {
                // Find the last non-empty capture group as the secret token.
                let mut token: Option<&str> = None;
                for i in 1..caps.len() {
                    if let Some(g) = caps.get(i) {
                        let s = g.as_str();
                        if !s.is_empty() {
                            token = Some(s);
                        }
                    }
                }
                if let Some(tok) = token {
                    if let Some(idx) = matched.rfind(tok) {
                        result.push_str(&matched[..idx]);
                        result.push_str("[REDACTED]");
                        result.push_str(&matched[idx + tok.len()..]);
                    } else {
                        result.push_str("[REDACTED]");
                    }
                } else {
                    result.push_str("[REDACTED]");
                }
            }
            last_end = m.end();
        }
        result.push_str(&redacted[last_end..]);
        redacted = result;
    }
    redacted
}

/// Render a non-Error `cause` value without leaking `[object Object]` or throwing
/// while formatting nested ACP runtime failures.
pub fn stringify_non_error_cause(value: &serde_json::Value) -> String {
    use serde_json::Value;
    match value {
        Value::Null => "null".to_string(),
        Value::String(s) => s.clone(),
        Value::Number(n) => n.to_string(),
        Value::Bool(b) => b.to_string(),
        _ => match serde_json::to_string(value) {
            Ok(s) => s,
            Err(_) => format!("{:?}", value),
        },
    }
}

/// Convenience overload accepting `&Value` and rendering any unknown cause.
pub fn stringify_non_error_cause_unknown(value: &dyn std::fmt::Debug) -> String {
    format!("{:?}", value)
}