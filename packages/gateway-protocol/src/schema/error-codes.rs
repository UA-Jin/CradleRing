// Gateway Protocol schema module defines protocol validation shapes.
// 翻译自 packages/gateway-protocol/src/schema/error-codes.ts

use crate::frames::ErrorShape;

/// Gateway JSON-RPC style error codes shared by clients and server handlers.
pub mod error_codes {
    /// Client has not completed account/device linking for this gateway.
    pub const NOT_LINKED: &str = "NOT_LINKED";
    /// Device exists but still needs an explicit pairing approval.
    pub const NOT_PAIRED: &str = "NOT_PAIRED";
    /// Agent turn exceeded the gateway wait window.
    pub const AGENT_TIMEOUT: &str = "AGENT_TIMEOUT";
    /// Request payload failed protocol validation or method preconditions.
    pub const INVALID_REQUEST: &str = "INVALID_REQUEST";
    /// Approval resolution referenced a missing or expired approval request.
    pub const APPROVAL_NOT_FOUND: &str = "APPROVAL_NOT_FOUND";
    /// Gateway service or required backend is temporarily unavailable.
    pub const UNAVAILABLE: &str = "UNAVAILABLE";
}

/// Closed set of canonical gateway error code strings.
pub type ErrorCode = &'static str;

/// Builds the canonical gateway error payload while preserving optional retry metadata.
pub fn error_shape(
    code: ErrorCode,
    message: &str,
    details: Option<serde_json::Value>,
    retryable: Option<bool>,
    retry_after_ms: Option<i64>,
) -> ErrorShape {
    ErrorShape {
        code: code.to_string(),
        message: message.to_string(),
        details,
        retryable,
        retry_after_ms,
    }
}