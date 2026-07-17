// Gateway Protocol: lightweight frame guards.
// 翻译自 packages/gateway-protocol/src/frame-guards.ts
//
// Lightweight runtime guards that validate dispatch-critical envelope fields
// without compiling the full schemas or rejecting additive payload fields.

use serde_json::Value;

pub use crate::frames::{EventFrame, ResponseFrame};

// Re-export canonical frame types for downstream consumers.
pub use crate::frames::{ConnectParams, ErrorShape, GatewayFrame, HelloOk, RequestFrame};

fn is_record(value: &Value) -> bool {
    value.is_object()
}

fn is_non_empty_string_opt(value: Option<&Value>) -> bool {
    value.map(is_non_empty_string).unwrap_or(false)
}

fn is_non_empty_string(value: &Value) -> bool {
    value.as_str().map(|s| !s.is_empty()).unwrap_or(false)
}

fn is_non_negative_integer(value: &Value) -> bool {
    value.as_i64().map(|n| n >= 0).unwrap_or(false)
}

fn is_gateway_error_shape(value: &Value) -> bool {
    if !is_record(value) {
        return false;
    }
    let obj = match value.as_object() {
        Some(o) => o,
        None => return false,
    };
    if !is_non_empty_string_opt(obj.get("code"))
        || !is_non_empty_string_opt(obj.get("message"))
    {
        return false;
    }
    if let Some(retryable) = obj.get("retryable") {
        if !retryable.is_boolean() {
            return false;
        }
    }
    if let Some(retry_after_ms) = obj.get("retryAfterMs") {
        if !is_non_negative_integer(retry_after_ms) {
            return false;
        }
    }
    true
}

/// Lightweight guard for an `event` frame envelope. Validates only the
/// dispatch-critical fields (type/event/seq) and lets payload be permissive.
/// 对齐 TS: `isGatewayEventFrame(value: unknown): value is EventFrame`.
pub fn is_gateway_event_frame(value: &Value) -> bool {
    if !is_record(value) {
        return false;
    }
    let obj = match value.as_object() {
        Some(o) => o,
        None => return false,
    };
    let frame_type = obj.get("type");
    if frame_type.and_then(Value::as_str) != Some("event") {
        return false;
    }
    if !is_non_empty_string_opt(obj.get("event")) {
        return false;
    }
    if let Some(seq) = obj.get("seq") {
        if !is_non_negative_integer(seq) {
            return false;
        }
    }
    true
}

/// Lightweight guard for a `res` frame envelope. Validates only the
/// dispatch-critical fields (type/id/ok/error) and lets payload be permissive.
/// 对齐 TS: `isGatewayResponseFrame(value: unknown): value is ResponseFrame`.
pub fn is_gateway_response_frame(value: &Value) -> bool {
    if !is_record(value) {
        return false;
    }
    let obj = match value.as_object() {
        Some(o) => o,
        None => return false,
    };
    let frame_type = obj.get("type");
    if frame_type.and_then(Value::as_str) != Some("res") {
        return false;
    }
    if !is_non_empty_string_opt(obj.get("id")) {
        return false;
    }
    if !obj.get("ok").map(Value::is_boolean).unwrap_or(false) {
        return false;
    }
    if let Some(err) = obj.get("error") {
        if !is_gateway_error_shape(err) {
            return false;
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn event_frame_requires_event_field() {
        assert!(is_gateway_event_frame(&json!({"type": "event", "event": "tick"})));
        assert!(!is_gateway_event_frame(&json!({"type": "event"})));
        assert!(!is_gateway_event_frame(&json!({"type": "req", "event": "tick"})));
    }

    #[test]
    fn event_frame_seq_must_be_non_negative_integer() {
        assert!(is_gateway_event_frame(
            &json!({"type": "event", "event": "tick", "seq": 0})
        ));
        assert!(is_gateway_event_frame(
            &json!({"type": "event", "event": "tick", "seq": 12})
        ));
        assert!(!is_gateway_event_frame(
            &json!({"type": "event", "event": "tick", "seq": -1})
        ));
        assert!(!is_gateway_event_frame(
            &json!({"type": "event", "event": "tick", "seq": "1"})
        ));
    }

    #[test]
    fn response_frame_requires_id_and_ok() {
        assert!(is_gateway_response_frame(
            &json!({"type": "res", "id": "r-1", "ok": true})
        ));
        assert!(!is_gateway_response_frame(
            &json!({"type": "res", "ok": true})
        ));
        assert!(!is_gateway_response_frame(
            &json!({"type": "res", "id": "r-1"})
        ));
    }

    #[test]
    fn response_frame_error_must_be_well_formed() {
        assert!(is_gateway_response_frame(&json!({
            "type": "res",
            "id": "r-1",
            "ok": false,
            "error": {"code": "INVALID", "message": "bad"}
        })));
        assert!(!is_gateway_response_frame(&json!({
            "type": "res",
            "id": "r-1",
            "ok": false,
            "error": {"code": "INVALID"}
        })));
    }
}
