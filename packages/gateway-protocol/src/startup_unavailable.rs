// Gateway Protocol: startup-unavailable error helpers.
// 翻译自 packages/gateway-protocol/src/startup-unavailable.ts
//
// Structured error reason used while gateway startup sidecars are still
// initializing. The constants and helpers here are used by both the gateway
// (to emit the structured error) and clients (to detect the retryable cause).

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Structured error reason for startup-unavailable errors.
pub const GATEWAY_STARTUP_UNAVAILABLE_REASON: &str = "startup-sidecars";

/// Internal close cause that distinguishes startup retry closes from generic
/// disconnects.
pub const GATEWAY_STARTUP_PENDING_CLOSE_CAUSE: &str = "startup-sidecars-pending";

/// WebSocket close code for temporary gateway unavailability.
pub const GATEWAY_STARTUP_CLOSE_CODE: i64 = 1013;

/// Human-readable WebSocket close reason for temporary gateway startup
/// unavailability.
pub const GATEWAY_STARTUP_CLOSE_REASON: &str = "gateway starting";

/// Default retry-after hint sent with startup-unavailable handshake errors.
pub const GATEWAY_STARTUP_RETRY_AFTER_MS: i64 = 500;

/// Lower bound for the bounded retry-after delay helper.
pub const GATEWAY_STARTUP_RETRY_MIN_MS: i64 = 100;

/// Upper bound for the bounded retry-after delay helper.
pub const GATEWAY_STARTUP_RETRY_MAX_MS: i64 = 2_000;

/// Details payload attached to retryable startup-unavailable gateway errors.
/// 对齐 TS: `type GatewayStartupUnavailableDetails = { reason: ... }`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GatewayStartupUnavailableDetails {
    pub reason: &'static str,
}

/// Builds the canonical startup-unavailable details payload.
/// 对齐 TS: `gatewayStartupUnavailableDetails()`.
pub fn gateway_startup_unavailable_details() -> GatewayStartupUnavailableDetails {
    GatewayStartupUnavailableDetails {
        reason: GATEWAY_STARTUP_UNAVAILABLE_REASON,
    }
}

fn is_gateway_startup_unavailable_details(details: &Value) -> bool {
    details
        .as_object()
        .and_then(|o| o.get("reason"))
        .and_then(Value::as_str)
        == Some(GATEWAY_STARTUP_UNAVAILABLE_REASON)
}

/// Detects the structured retryable error emitted while startup sidecars are
/// pending. Accepts both the standard gateway error shape (`code`) and the
/// alternate `gatewayCode` field used in some clients.
/// 对齐 TS: `isRetryableGatewayStartupUnavailableError(error: unknown)`.
pub fn is_retryable_gateway_startup_unavailable_error(error: &Value) -> bool {
    if !error.is_object() {
        return false;
    }
    let obj = match error.as_object() {
        Some(o) => o,
        None => return false,
    };
    let code = obj
        .get("gatewayCode")
        .and_then(Value::as_str)
        .or_else(|| obj.get("code").and_then(Value::as_str));
    let retryable = obj.get("retryable").and_then(Value::as_bool);
    let details = obj.get("details").unwrap_or(&Value::Null);
    code == Some("UNAVAILABLE")
        && retryable == Some(true)
        && is_gateway_startup_unavailable_details(details)
}

/// Resolves a bounded retry-after delay from a startup-unavailable error.
/// Returns `None` when `error` is not a retryable startup-unavailable error.
/// 对齐 TS: `resolveGatewayStartupRetryAfterMs(error: unknown): number | null`.
pub fn resolve_gateway_startup_retry_after_ms(error: &Value) -> Option<i64> {
    if !is_retryable_gateway_startup_unavailable_error(error) {
        return None;
    }
    let obj = error.as_object()?;
    let raw = obj
        .get("retryAfterMs")
        .and_then(Value::as_f64)
        .filter(|n| n.is_finite())
        .map(|n| n.floor() as i64)
        .unwrap_or(GATEWAY_STARTUP_RETRY_AFTER_MS);
    Some(raw
        .max(GATEWAY_STARTUP_RETRY_MIN_MS)
        .min(GATEWAY_STARTUP_RETRY_MAX_MS))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn detects_retryable_startup_error() {
        let err = json!({
            "code": "UNAVAILABLE",
            "retryable": true,
            "details": {"reason": GATEWAY_STARTUP_UNAVAILABLE_REASON}
        });
        assert!(is_retryable_gateway_startup_unavailable_error(&err));
    }

    #[test]
    fn rejects_non_retryable() {
        let err = json!({
            "code": "UNAVAILABLE",
            "retryable": false,
            "details": {"reason": GATEWAY_STARTUP_UNAVAILABLE_REASON}
        });
        assert!(!is_retryable_gateway_startup_unavailable_error(&err));
    }

    #[test]
    fn rejects_wrong_details() {
        let err = json!({
            "code": "UNAVAILABLE",
            "retryable": true,
            "details": {"reason": "other"}
        });
        assert!(!is_retryable_gateway_startup_unavailable_error(&err));
    }

    #[test]
    fn resolves_bounded_retry_after() {
        let err = json!({
            "code": "UNAVAILABLE",
            "retryable": true,
            "retryAfterMs": 5000,
            "details": {"reason": GATEWAY_STARTUP_UNAVAILABLE_REASON}
        });
        assert_eq!(
            resolve_gateway_startup_retry_after_ms(&err),
            Some(GATEWAY_STARTUP_RETRY_MAX_MS)
        );
    }

    #[test]
    fn resolves_default_retry_after_when_missing() {
        let err = json!({
            "code": "UNAVAILABLE",
            "retryable": true,
            "details": {"reason": GATEWAY_STARTUP_UNAVAILABLE_REASON}
        });
        assert_eq!(
            resolve_gateway_startup_retry_after_ms(&err),
            Some(GATEWAY_STARTUP_RETRY_AFTER_MS)
        );
    }
}
