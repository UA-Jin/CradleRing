// ACP Core module implements error text behavior.
// 翻译自 packages/acp-core/src/runtime/error-text.ts

use crate::runtime::errors::{to_acp_runtime_error, AcpRuntimeError, AcpRuntimeErrorCode, ToAcpParams};

fn resolve_acp_runtime_error_next_step(error: &AcpRuntimeError) -> Option<String> {
    match error.code.as_str() {
        "ACP_BACKEND_MISSING" | "ACP_BACKEND_UNAVAILABLE" => {
            Some("Run `/acp doctor`, install/enable the backend plugin, then retry.".to_string())
        }
        "ACP_DISPATCH_DISABLED" => {
            Some("Enable `acp.dispatch.enabled=true` to allow thread-message ACP turns.".to_string())
        }
        "ACP_SESSION_INIT_FAILED" => Some(
            "If this session is stale, recreate it with `/acp spawn` and rebind the thread."
                .to_string(),
        ),
        "ACP_INVALID_RUNTIME_OPTION" => {
            Some("Use `/acp status` to inspect options and pass valid values.".to_string())
        }
        "ACP_BACKEND_UNSUPPORTED_CONTROL" => Some(
            "This backend does not support that control; use a supported command.".to_string(),
        ),
        "ACP_TURN_FAILED" => {
            Some("Retry, or use `/acp cancel` and send the message again.".to_string())
        }
        _ => None,
    }
}

/// Formats ACP runtime errors with the operator next-step hint attached when known.
pub fn format_acp_runtime_error_text(error: &AcpRuntimeError) -> String {
    match resolve_acp_runtime_error_next_step(error) {
        Some(next) => format!("ACP error ({}): {}\nnext: {}", error.code, error.message, next),
        None => format!("ACP error ({}): {}", error.code, error.message),
    }
}

/// Normalizes unknown failures into ACP runtime error text for user-facing surfaces.
pub fn to_acp_runtime_error_text<E: std::fmt::Debug>(
    error: &E,
    fallback_code: AcpRuntimeErrorCode,
    fallback_message: &str,
) -> String {
    let acp_error = to_acp_runtime_error(ToAcpParams {
        error,
        fallback_code: &fallback_code,
        fallback_message,
    });
    format_acp_runtime_error_text(&acp_error)
}