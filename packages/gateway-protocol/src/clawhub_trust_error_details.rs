// Gateway Protocol: ClawHub trust error details.
// 翻译自 packages/gateway-protocol/src/clawhub-trust-error-details.ts
//
// Structured ClawHub trust details carried in gateway error payloads. Provides
// the trust-error codes, the payload shape, and helpers to build/read the
// payload from arbitrary JSON values.

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Closed enumeration of ClawHub trust error codes.
/// 对齐 TS: `ClawHubTrustErrorCodes` const object.
pub mod clawhub_trust_error_codes {
    pub const SECURITY_UNAVAILABLE: &str = "clawhub_security_unavailable";
    pub const RISK_ACKNOWLEDGEMENT_REQUIRED: &str = "clawhub_risk_acknowledgement_required";
    pub const DOWNLOAD_BLOCKED: &str = "clawhub_download_blocked";
}

/// Discriminator type for ClawHub trust error codes.
/// 对齐 TS: `type ClawHubTrustErrorCode = ...`.
pub type ClawHubTrustErrorCode = String;

/// Structured ClawHub trust details carried in gateway error payloads.
/// 对齐 TS: `type ClawHubTrustErrorDetails = { ... }`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ClawHubTrustErrorDetails {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub clawhub_trust_code: Option<ClawHubTrustErrorCode>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub warning: Option<String>,
}

fn normalize_non_empty_string(value: &Value) -> Option<String> {
    let s = value.as_str()?;
    let trimmed = s.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

/// Returns true when `value` matches a closed ClawHub trust error code.
/// 对齐 TS: `isClawHubTrustErrorCode(value: unknown)`.
pub fn is_clawhub_trust_error_code(value: &Value) -> bool {
    matches!(
        value.as_str(),
        Some(
            clawhub_trust_error_codes::SECURITY_UNAVAILABLE
                | clawhub_trust_error_codes::RISK_ACKNOWLEDGEMENT_REQUIRED
                | clawhub_trust_error_codes::DOWNLOAD_BLOCKED,
        )
    )
}

/// Builds the canonical structured details payload, omitting empty fields.
/// 对齐 TS: `buildClawHubTrustErrorDetails(params)`.
pub fn build_clawhub_trust_error_details(
    code: Option<&str>,
    version: Option<&str>,
    warning: Option<&str>,
) -> Option<ClawHubTrustErrorDetails> {
    if code.is_none() && version.is_none() && warning.is_none() {
        return None;
    }
    Some(ClawHubTrustErrorDetails {
        clawhub_trust_code: code.map(|s| s.to_string()),
        version: version.map(|s| s.to_string()),
        warning: warning.map(|s| s.to_string()),
    })
}

/// Reads and normalizes a structured ClawHub trust error details payload from
/// an untrusted JSON value. Returns `None` when the payload has no usable
/// content.
/// 对齐 TS: `readClawHubTrustErrorDetails(details: unknown)`.
pub fn read_clawhub_trust_error_details(details: &Value) -> Option<ClawHubTrustErrorDetails> {
    let obj = details.as_object()?;
    let raw_code = obj.get("clawhubTrustCode").unwrap_or(&Value::Null);
    let raw_version = obj.get("version").unwrap_or(&Value::Null);
    let raw_warning = obj.get("warning").unwrap_or(&Value::Null);

    let code = if is_clawhub_trust_error_code(raw_code) {
        raw_code.as_str().map(|s| s.to_string())
    } else {
        None
    };
    let version = normalize_non_empty_string(raw_version);
    let warning = normalize_non_empty_string(raw_warning);

    if code.is_none() && version.is_none() && warning.is_none() {
        return None;
    }
    Some(ClawHubTrustErrorDetails {
        clawhub_trust_code: code,
        version,
        warning,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn is_clawhub_trust_error_code_recognises_known_values() {
        assert!(is_clawhub_trust_error_code(
            &json!(clawhub_trust_error_codes::SECURITY_UNAVAILABLE)
        ));
        assert!(is_clawhub_trust_error_code(&json!(
            clawhub_trust_error_codes::DOWNLOAD_BLOCKED
        )));
        assert!(!is_clawhub_trust_error_code(&json!("other")));
        assert!(!is_clawhub_trust_error_code(&json!(42)));
    }

    #[test]
    fn build_skips_when_all_empty() {
        assert!(build_clawhub_trust_error_details(None, None, None).is_none());
    }

    #[test]
    fn build_preserves_supplied_fields() {
        let details = build_clawhub_trust_error_details(
            Some(clawhub_trust_error_codes::SECURITY_UNAVAILABLE),
            Some("1.0"),
            Some("warn"),
        )
        .unwrap();
        assert_eq!(
            details.clawhub_trust_code.as_deref(),
            Some(clawhub_trust_error_codes::SECURITY_UNAVAILABLE)
        );
        assert_eq!(details.version.as_deref(), Some("1.0"));
        assert_eq!(details.warning.as_deref(), Some("warn"));
    }

    #[test]
    fn read_returns_none_for_non_object() {
        assert!(read_clawhub_trust_error_details(&json!("nope")).is_none());
        assert!(read_clawhub_trust_error_details(&json!([])).is_none());
        assert!(read_clawhub_trust_error_details(&Value::Null).is_none());
    }

    #[test]
    fn read_normalizes_payload() {
        let payload = json!({
            "clawhubTrustCode": clawhub_trust_error_codes::DOWNLOAD_BLOCKED,
            "version": "  ",
            "warning": "blocked"
        });
        let details = read_clawhub_trust_error_details(&payload).unwrap();
        assert_eq!(
            details.clawhub_trust_code.as_deref(),
            Some(clawhub_trust_error_codes::DOWNLOAD_BLOCKED)
        );
        assert!(details.version.is_none());
        assert_eq!(details.warning.as_deref(), Some("blocked"));
    }
}
