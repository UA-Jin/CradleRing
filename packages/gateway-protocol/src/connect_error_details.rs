// Shared gateway connect-error detail helpers.
// 翻译自 packages/gateway-protocol/src/connect-error-details.ts
//
// These details cross client/server boundaries, so readers normalize untrusted
// payloads before using them in reconnect decisions or user-facing messages.

use serde::{Deserialize, Serialize};
use serde_json::Value;

fn normalize_optional_string(value: &Value) -> Option<String> {
    let s = value.as_str()?;
    let trimmed = s.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

fn normalize_array_backed_trimmed_string_list(value: &Value) -> Option<Vec<String>> {
    let arr = value.as_array()?;
    let values: Vec<String> = arr
        .iter()
        .filter_map(|entry| normalize_optional_string(entry))
        .collect();
    if values.is_empty() {
        None
    } else {
        Some(values)
    }
}

/// Structured connect-error codes carried in gateway error `details.code`.
pub mod connect_error_detail_codes {
    pub const AUTH_REQUIRED: &str = "AUTH_REQUIRED";
    pub const AUTH_UNAUTHORIZED: &str = "AUTH_UNAUTHORIZED";
    pub const AUTH_TOKEN_MISSING: &str = "AUTH_TOKEN_MISSING";
    pub const AUTH_TOKEN_MISMATCH: &str = "AUTH_TOKEN_MISMATCH";
    pub const AUTH_TOKEN_NOT_CONFIGURED: &str = "AUTH_TOKEN_NOT_CONFIGURED";
    pub const AUTH_PASSWORD_MISSING: &str = "AUTH_PASSWORD_MISSING";
    pub const AUTH_PASSWORD_MISMATCH: &str = "AUTH_PASSWORD_MISMATCH";
    pub const AUTH_PASSWORD_NOT_CONFIGURED: &str = "AUTH_PASSWORD_NOT_CONFIGURED";
    pub const AUTH_BOOTSTRAP_TOKEN_INVALID: &str = "AUTH_BOOTSTRAP_TOKEN_INVALID";
    pub const AUTH_DEVICE_TOKEN_MISMATCH: &str = "AUTH_DEVICE_TOKEN_MISMATCH";
    pub const AUTH_SCOPE_MISMATCH: &str = "AUTH_SCOPE_MISMATCH";
    pub const AUTH_RATE_LIMITED: &str = "AUTH_RATE_LIMITED";
    pub const AUTH_TAILSCALE_IDENTITY_MISSING: &str = "AUTH_TAILSCALE_IDENTITY_MISSING";
    pub const AUTH_TAILSCALE_PROXY_MISSING: &str = "AUTH_TAILSCALE_PROXY_MISSING";
    pub const AUTH_TAILSCALE_WHOIS_FAILED: &str = "AUTH_TAILSCALE_WHOIS_FAILED";
    pub const AUTH_TAILSCALE_IDENTITY_MISMATCH: &str = "AUTH_TAILSCALE_IDENTITY_MISMATCH";
    pub const CONTROL_UI_ORIGIN_NOT_ALLOWED: &str = "CONTROL_UI_ORIGIN_NOT_ALLOWED";
    pub const PROTOCOL_MISMATCH: &str = "PROTOCOL_MISMATCH";
    pub const CONTROL_UI_DEVICE_IDENTITY_REQUIRED: &str = "CONTROL_UI_DEVICE_IDENTITY_REQUIRED";
    pub const DEVICE_IDENTITY_REQUIRED: &str = "DEVICE_IDENTITY_REQUIRED";
    pub const DEVICE_AUTH_INVALID: &str = "DEVICE_AUTH_INVALID";
    pub const DEVICE_AUTH_DEVICE_ID_MISMATCH: &str = "DEVICE_AUTH_DEVICE_ID_MISMATCH";
    pub const DEVICE_AUTH_SIGNATURE_EXPIRED: &str = "DEVICE_AUTH_SIGNATURE_EXPIRED";
    pub const DEVICE_AUTH_NONCE_REQUIRED: &str = "DEVICE_AUTH_NONCE_REQUIRED";
    pub const DEVICE_AUTH_NONCE_MISMATCH: &str = "DEVICE_AUTH_NONCE_MISMATCH";
    pub const DEVICE_AUTH_SIGNATURE_INVALID: &str = "DEVICE_AUTH_SIGNATURE_INVALID";
    pub const DEVICE_AUTH_PUBLIC_KEY_INVALID: &str = "DEVICE_AUTH_PUBLIC_KEY_INVALID";
    pub const PAIRING_REQUIRED: &str = "PAIRING_REQUIRED";
    pub const CLIENT_VERSION_MISMATCH: &str = "CLIENT_VERSION_MISMATCH";
}

pub type ConnectErrorDetailCode = &'static str;

/// Pairing-specific reasons clients can display and use for reconnect policy.
pub mod connect_pairing_required_reasons {
    pub const NOT_PAIRED: &str = "not-paired";
    pub const ROLE_UPGRADE: &str = "role-upgrade";
    pub const SCOPE_UPGRADE: &str = "scope-upgrade";
    pub const METADATA_UPGRADE: &str = "metadata-upgrade";
}

pub type ConnectPairingRequiredReason = &'static str;

/// Suggested client-side recovery action for structured connect errors.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ConnectRecoveryNextStep {
    RetryWithDeviceToken,
    UpdateAuthConfiguration,
    UpdateAuthCredentials,
    WaitThenRetry,
    ReviewAuthConfiguration,
}

impl ConnectRecoveryNextStep {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::RetryWithDeviceToken => "retry_with_device_token",
            Self::UpdateAuthConfiguration => "update_auth_configuration",
            Self::UpdateAuthCredentials => "update_auth_credentials",
            Self::WaitThenRetry => "wait_then_retry",
            Self::ReviewAuthConfiguration => "review_auth_configuration",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "retry_with_device_token" => Some(Self::RetryWithDeviceToken),
            "update_auth_configuration" => Some(Self::UpdateAuthConfiguration),
            "update_auth_credentials" => Some(Self::UpdateAuthCredentials),
            "wait_then_retry" => Some(Self::WaitThenRetry),
            "review_auth_configuration" => Some(Self::ReviewAuthConfiguration),
            _ => None,
        }
    }
}

/// Optional retry guidance extracted from gateway connect-error details.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectErrorRecoveryAdvice {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub can_retry_with_device_token: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recommended_next_step: Option<String>,
}

/// Full structured details for pairing-required connect failures.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PairingConnectErrorDetails {
    pub code: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remediation_hint: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recommended_next_step: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub retryable: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pause_reconnect: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub device_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub requested_role: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub requested_scopes: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub approved_roles: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub approved_scopes: Option<Vec<String>>,
}

/// Compact pairing-required subset used by reconnect/status surfaces.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectPairingRequiredDetails {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,
}

const PAIRING_CONNECT_REQUEST_ID_PATTERN: &str = r"^[A-Za-z0-9][A-Za-z0-9._:-]{0,127}$";

fn pairing_connect_reason_metadata(reason: &str) -> Option<(&'static str, &'static str, &'static str)> {
    Some(match reason {
        "not-paired" => (
            "device is not approved yet",
            "Approve this device from the pending pairing requests.",
            "Gateway pairing approval required.",
        ),
        "role-upgrade" => (
            "device is asking for a higher role than currently approved",
            "Review the requested role upgrade, then approve the pending request.",
            "Gateway role upgrade approval required.",
        ),
        "scope-upgrade" => (
            "device is asking for more scopes than currently approved",
            "Review the requested scopes, then approve the pending upgrade.",
            "Gateway scope upgrade approval required.",
        ),
        "metadata-upgrade" => (
            "device identity changed and must be re-approved",
            "Review the refreshed device details, then approve the pending request.",
            "Gateway device refresh approval required.",
        ),
        _ => return None,
    })
}

fn connect_pairing_required_message_by_reason(reason: &str) -> &'static str {
    match reason {
        "not-paired" => "device pairing required",
        "role-upgrade" => "role upgrade pending approval",
        "scope-upgrade" => "scope upgrade pending approval",
        "metadata-upgrade" => "device metadata change pending approval",
        _ => "device pairing required",
    }
}

/// Maps internal auth failure reasons to public connect-error detail codes.
pub fn resolve_auth_connect_error_detail_code(reason: Option<&str>) -> ConnectErrorDetailCode {
    match reason {
        Some("token_missing") => connect_error_detail_codes::AUTH_TOKEN_MISSING,
        Some("token_mismatch") => connect_error_detail_codes::AUTH_TOKEN_MISMATCH,
        Some("token_missing_config") => connect_error_detail_codes::AUTH_TOKEN_NOT_CONFIGURED,
        Some("password_missing") => connect_error_detail_codes::AUTH_PASSWORD_MISSING,
        Some("password_mismatch") => connect_error_detail_codes::AUTH_PASSWORD_MISMATCH,
        Some("password_missing_config") => connect_error_detail_codes::AUTH_PASSWORD_NOT_CONFIGURED,
        Some("bootstrap_token_invalid") => connect_error_detail_codes::AUTH_BOOTSTRAP_TOKEN_INVALID,
        Some("tailscale_user_missing") => connect_error_detail_codes::AUTH_TAILSCALE_IDENTITY_MISSING,
        Some("tailscale_proxy_missing") => connect_error_detail_codes::AUTH_TAILSCALE_PROXY_MISSING,
        Some("tailscale_whois_failed") => connect_error_detail_codes::AUTH_TAILSCALE_WHOIS_FAILED,
        Some("tailscale_user_mismatch") => connect_error_detail_codes::AUTH_TAILSCALE_IDENTITY_MISMATCH,
        Some("rate_limited") => connect_error_detail_codes::AUTH_RATE_LIMITED,
        Some("device_token_mismatch") => connect_error_detail_codes::AUTH_DEVICE_TOKEN_MISMATCH,
        Some("scope_mismatch") => connect_error_detail_codes::AUTH_SCOPE_MISMATCH,
        None => connect_error_detail_codes::AUTH_REQUIRED,
        Some(_) => connect_error_detail_codes::AUTH_UNAUTHORIZED,
    }
}

/// Maps device-auth verifier reasons to public connect-error detail codes.
pub fn resolve_device_auth_connect_error_detail_code(reason: Option<&str>) -> ConnectErrorDetailCode {
    match reason {
        Some("device-id-mismatch") => connect_error_detail_codes::DEVICE_AUTH_DEVICE_ID_MISMATCH,
        Some("device-signature-stale") => connect_error_detail_codes::DEVICE_AUTH_SIGNATURE_EXPIRED,
        Some("device-nonce-missing") => connect_error_detail_codes::DEVICE_AUTH_NONCE_REQUIRED,
        Some("device-nonce-mismatch") => connect_error_detail_codes::DEVICE_AUTH_NONCE_MISMATCH,
        Some("device-signature") => connect_error_detail_codes::DEVICE_AUTH_SIGNATURE_INVALID,
        Some("device-public-key") => connect_error_detail_codes::DEVICE_AUTH_PUBLIC_KEY_INVALID,
        _ => connect_error_detail_codes::DEVICE_AUTH_INVALID,
    }
}

/// Reads a non-empty detail code from an untrusted error details payload.
pub fn read_connect_error_detail_code(details: &Value) -> Option<String> {
    if !details.is_object() || details.is_array() {
        return None;
    }
    let code = details.get("code")?;
    let s = code.as_str()?;
    let trimmed = s.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

/// Extracts normalized retry advice from untrusted connect-error details.
pub fn read_connect_error_recovery_advice(details: &Value) -> ConnectErrorRecoveryAdvice {
    if !details.is_object() || details.is_array() {
        return ConnectErrorRecoveryAdvice::default();
    }
    let can_retry = details
        .get("canRetryWithDeviceToken")
        .and_then(|v| v.as_bool());
    let next_step = details
        .get("recommendedNextStep")
        .and_then(|v| v.as_str())
        .and_then(|s| ConnectRecoveryNextStep::from_str(s.trim()))
        .map(|s| s.as_str().to_string());
    ConnectErrorRecoveryAdvice {
        can_retry_with_device_token: can_retry,
        recommended_next_step: next_step,
    }
}

fn normalize_pairing_connect_reason(value: &Value) -> Option<String> {
    let normalized = normalize_optional_string(value)?;
    match normalized.as_str() {
        "not-paired" | "role-upgrade" | "scope-upgrade" | "metadata-upgrade" => Some(normalized),
        _ => None,
    }
}

/// Normalizes pairing request ids before echoing them in close reasons or UI text.
pub fn normalize_pairing_connect_request_id(value: &Value) -> Option<String> {
    let normalized = normalize_optional_string(value)?;
    let re = regex::Regex::new(PAIRING_CONNECT_REQUEST_ID_PATTERN).unwrap();
    if re.is_match(&normalized) {
        Some(normalized)
    } else {
        None
    }
}

fn normalize_string_array(value: &Value) -> Option<Vec<String>> {
    normalize_array_backed_trimmed_string_list(value)
}

fn create_pairing_connect_error_details(
    reason: Option<String>,
    request_id: Option<String>,
    remediation_hint: Option<String>,
    recommended_next_step: Option<String>,
    retryable: Option<bool>,
    pause_reconnect: Option<bool>,
    device_id: Option<String>,
    requested_role: Option<String>,
    requested_scopes: Option<Vec<String>>,
    approved_roles: Option<Vec<String>>,
    approved_scopes: Option<Vec<String>>,
) -> PairingConnectErrorDetails {
    PairingConnectErrorDetails {
        code: connect_error_detail_codes::PAIRING_REQUIRED.to_string(),
        reason,
        request_id,
        remediation_hint,
        recommended_next_step,
        retryable,
        pause_reconnect,
        device_id,
        requested_role,
        requested_scopes,
        approved_roles,
        approved_scopes,
    }
}

/// Human-readable requirement summary for a pairing-required reason.
pub fn describe_pairing_connect_requirement(reason: Option<&str>) -> String {
    match reason {
        Some(r) => pairing_connect_reason_metadata(r)
            .map(|(req, _, _)| req.to_string())
            .unwrap_or_else(|| "device approval is required".to_string()),
        None => "device approval is required".to_string(),
    }
}

/// Builds the gateway close/error message for a pairing-required connect failure.
pub fn build_pairing_connect_error_message(reason: Option<&str>) -> String {
    match reason {
        Some(r) => format!("pairing required: {}", describe_pairing_connect_requirement(Some(r))),
        None => "pairing required".to_string(),
    }
}

fn build_pairing_connect_remediation_hint(reason: Option<&str>) -> String {
    match reason {
        Some(r) => pairing_connect_reason_metadata(r)
            .map(|(_, hint, _)| hint.to_string())
            .unwrap_or_else(|| "Approve the pending device request before retrying.".to_string()),
        None => "Approve the pending device request before retrying.".to_string(),
    }
}

/// Short user-facing recovery title for pairing-required connect failures.
pub fn build_pairing_connect_recovery_title(reason: Option<&str>) -> String {
    match reason {
        Some(r) => pairing_connect_reason_metadata(r)
            .map(|(_, _, title)| title.to_string())
            .unwrap_or_else(|| "Gateway pairing approval required.".to_string()),
        None => "Gateway pairing approval required.".to_string(),
    }
}

/// Builds sanitized structured details for a pairing-required connect failure.
#[allow(clippy::too_many_arguments)]
pub fn build_pairing_connect_error_details(
    reason: Option<&str>,
    request_id: Option<&Value>,
    remediation_hint: Option<&Value>,
    recommended_next_step: Option<String>,
    retryable: Option<bool>,
    pause_reconnect: Option<bool>,
    device_id: Option<&Value>,
    requested_role: Option<&Value>,
    requested_scopes: Option<&Value>,
    approved_roles: Option<&Value>,
    approved_scopes: Option<&Value>,
) -> PairingConnectErrorDetails {
    let request_id = request_id.and_then(normalize_pairing_connect_request_id);
    let remediation_hint = remediation_hint
        .and_then(normalize_optional_string)
        .or_else(|| Some(build_pairing_connect_remediation_hint(reason)));
    let device_id = device_id.and_then(normalize_optional_string);
    let requested_role = requested_role.and_then(normalize_optional_string);
    let requested_scopes = requested_scopes.and_then(normalize_string_array);
    let approved_roles = approved_roles.and_then(normalize_string_array);
    let approved_scopes = approved_scopes.and_then(normalize_string_array);
    create_pairing_connect_error_details(
        reason.map(String::from),
        request_id,
        remediation_hint,
        recommended_next_step,
        retryable,
        pause_reconnect,
        device_id,
        requested_role,
        requested_scopes,
        approved_roles,
        approved_scopes,
    )
}

/// Builds a sanitized close reason string for WebSocket pairing rejections.
pub fn build_pairing_connect_close_reason(reason: Option<&str>, request_id: Option<&Value>) -> String {
    let request_id = request_id.and_then(normalize_pairing_connect_request_id);
    let message = build_pairing_connect_error_message(reason);
    match request_id {
        Some(id) => format!("{} (requestId: {})", message, id),
        None => message,
    }
}

/// Reads and backfills pairing-required details from an untrusted details object.
pub fn read_pairing_connect_error_details(details: &Value) -> Option<PairingConnectErrorDetails> {
    if read_connect_error_detail_code(details).as_deref() != Some(connect_error_detail_codes::PAIRING_REQUIRED) {
        return None;
    }
    if !details.is_object() || details.is_array() {
        return None;
    }
    let reason = details.get("reason").map(normalize_pairing_connect_reason).flatten();
    let request_id = details.get("requestId").and_then(normalize_pairing_connect_request_id);
    let remediation_hint = details
        .get("remediationHint")
        .and_then(normalize_optional_string)
        .or_else(|| Some(build_pairing_connect_remediation_hint(reason.as_deref())));
    let recommended_next_step = details
        .get("recommendedNextStep")
        .and_then(|v| v.as_str())
        .and_then(|s| ConnectRecoveryNextStep::from_str(s.trim()))
        .map(|s| s.as_str().to_string());
    let device_id = details.get("deviceId").and_then(normalize_optional_string);
    let requested_role = details.get("requestedRole").and_then(normalize_optional_string);
    let requested_scopes = details.get("requestedScopes").and_then(normalize_string_array);
    let approved_roles = details.get("approvedRoles").and_then(normalize_string_array);
    let approved_scopes = details.get("approvedScopes").and_then(normalize_string_array);
    let retryable = details.get("retryable").and_then(|v| v.as_bool());
    let pause_reconnect = details.get("pauseReconnect").and_then(|v| v.as_bool());

    Some(create_pairing_connect_error_details(
        reason,
        request_id,
        remediation_hint,
        recommended_next_step,
        retryable,
        pause_reconnect,
        device_id,
        requested_role,
        requested_scopes,
        approved_roles,
        approved_scopes,
    ))
}

/// Parses legacy/string-only pairing-required messages into structured details.
pub fn read_connect_pairing_required_message(message: Option<&Value>) -> Option<ConnectPairingRequiredDetails> {
    let normalized_message = normalize_optional_string(message?)?;
    let normalized = normalized_message.trim().to_lowercase();

    let mut reason: Option<String> = None;
    for candidate in &["not-paired", "role-upgrade", "scope-upgrade", "metadata-upgrade"] {
        let prefix = connect_pairing_required_message_by_reason(candidate);
        if normalized.contains(prefix) {
            reason = Some(candidate.to_string());
            break;
        }
    }
    if reason.is_none() && normalized.contains("pairing required") {
        reason = Some("not-paired".to_string());
    }
    let reason = reason?;

    let request_id_re = regex::Regex::new(r"(?i)\(requestId:\s*([^\s)]+)\)").unwrap();
    let request_id = request_id_re
        .captures(&normalized_message)
        .and_then(|c| c.get(1))
        .map(|m| m.as_str().to_string())
        .and_then(|s| {
            let v = Value::String(s);
            normalize_pairing_connect_request_id(&v)
        });

    Some(ConnectPairingRequiredDetails {
        reason: Some(reason),
        request_id,
    })
}

/// Formats pairing-required details into the canonical user-facing message.
pub fn format_connect_pairing_required_message(details: &Value) -> String {
    let pairing = read_pairing_connect_error_details(details);
    let reason = pairing
        .as_ref()
        .and_then(|p| p.reason.as_deref())
        .unwrap_or("not-paired");
    let base = connect_pairing_required_message_by_reason(reason);
    match pairing.and_then(|p| p.request_id) {
        Some(id) => format!("{} (requestId: {})", base, id),
        None => base.to_string(),
    }
}

/// Formats connect errors using structured details before falling back to raw messages.
pub fn format_connect_error_message(message: Option<&Value>, details: Option<&Value>) -> String {
    if let Some(d) = details {
        if read_connect_error_detail_code(d).as_deref() == Some(connect_error_detail_codes::PAIRING_REQUIRED) {
            return format_connect_pairing_required_message(d);
        }
        if read_connect_error_detail_code(d).as_deref() == Some(connect_error_detail_codes::PROTOCOL_MISMATCH) {
            return format_protocol_mismatch_message(message, d);
        }
    }
    message
        .and_then(normalize_optional_string)
        .unwrap_or_else(|| "gateway request failed".to_string())
}

fn format_protocol_mismatch_message(message: Option<&Value>, details: &Value) -> String {
    fn normalize_protocol_number(v: &Value) -> Option<i64> {
        let n = v.as_i64()?;
        if n > 0 {
            Some(n)
        } else {
            None
        }
    }
    let client_min = details.get("clientMinProtocol").and_then(normalize_protocol_number);
    let client_max = details.get("clientMaxProtocol").and_then(normalize_protocol_number);
    let expected = details.get("expectedProtocol").and_then(normalize_protocol_number);
    let probe_min = details.get("minimumProbeProtocol").and_then(normalize_protocol_number);

    let mut parts: Vec<String> = vec![];
    if let (Some(min), Some(max)) = (client_min, client_max) {
        if min == max {
            parts.push(format!("Control UI v{}", min));
        } else {
            parts.push(format!("Control UI v{}-v{}", min, max));
        }
    }
    if let Some(e) = expected {
        parts.push(format!("Gateway v{}", e));
    }
    if let Some(p) = probe_min {
        parts.push(format!("probe min v{}", p));
    }
    let normalized = message
        .and_then(normalize_optional_string)
        .unwrap_or_else(|| "protocol mismatch".to_string());
    if parts.is_empty() {
        normalized
    } else {
        format!("{}: {}", normalized, parts.join(", "))
    }
}
