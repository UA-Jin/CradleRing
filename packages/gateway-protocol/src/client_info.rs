// Shared gateway client identity contract.
// 翻译自 packages/gateway-protocol/src/client-info.ts
//
// These values cross the WebSocket handshake boundary, so additions must stay
// aligned with protocol schemas and server policy checks.

use serde::{Deserialize, Serialize};

fn normalize_optional_lowercase_string(raw: Option<&str>) -> Option<String> {
    let s = raw?;
    let normalized = s.trim().to_lowercase();
    if normalized.is_empty() {
        None
    } else {
        Some(normalized)
    }
}

/// Canonical client ids accepted in gateway hello/connect payloads.
/// 项目名替换：openclaw → cradle-ring
pub mod gateway_client_ids {
    pub const WEBCHAT_UI: &str = "webchat-ui";
    pub const CONTROL_UI: &str = "cradle-ring-control-ui";
    pub const TUI: &str = "cradle-ring-tui";
    pub const WEBCHAT: &str = "webchat";
    pub const CLI: &str = "cli";
    pub const GATEWAY_CLIENT: &str = "gateway-client";
    pub const MACOS_APP: &str = "cradle-ring-macos";
    pub const IOS_APP: &str = "cradle-ring-ios";
    pub const WATCHOS_APP: &str = "cradle-ring-watchos";
    pub const ANDROID_APP: &str = "cradle-ring-android";
    pub const NODE_HOST: &str = "node-host";
    pub const WORKER: &str = "cradle-ring-worker";
    pub const TEST: &str = "test";
    pub const FINGERPRINT: &str = "fingerprint";
    pub const PROBE: &str = "cradle-ring-probe";

    /// 所有合法 client id
    pub fn all() -> &'static [&'static str] {
        &[
            WEBCHAT_UI, CONTROL_UI, TUI, WEBCHAT, CLI, GATEWAY_CLIENT,
            MACOS_APP, IOS_APP, WATCHOS_APP, ANDROID_APP, NODE_HOST,
            WORKER, TEST, FINGERPRINT, PROBE,
        ]
    }
}

/// GatewayClientId (string on the wire)
pub type GatewayClientId = String;

/// Back-compat naming (internal): these values are IDs, not display names.
pub fn gateway_client_names() -> &'static [&'static str] {
    gateway_client_ids::all()
}
/// Compatibility alias for internal callers that still use "name" terminology.
pub type GatewayClientName = GatewayClientId;

/// Coarse modes let policy group clients without matching every product id.
pub mod gateway_client_modes {
    pub const WEBCHAT: &str = "webchat";
    pub const CLI: &str = "cli";
    pub const UI: &str = "ui";
    pub const BACKEND: &str = "backend";
    pub const NODE: &str = "node";
    pub const WORKER: &str = "worker";
    pub const PROBE: &str = "probe";
    pub const TEST: &str = "test";

    pub fn all() -> &'static [&'static str] {
        &[WEBCHAT, CLI, UI, BACKEND, NODE, WORKER, PROBE, TEST]
    }
}

/// Coarse client category used for gateway policy and diagnostics.
pub type GatewayClientMode = String;

/// Client metadata sent during gateway connection setup.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GatewayClientInfo {
    /// Stable product/client identifier from `GATEWAY_CLIENT_IDS`.
    pub id: GatewayClientId,
    /// Human-readable label for diagnostics; not used for policy decisions.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    /// Client app or package version reported by the connecting process.
    pub version: String,
    /// Runtime platform string, such as `darwin`, `ios`, `android`, or `web`.
    pub platform: String,
    /// Optional device family used by native clients for display and routing hints.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub device_family: Option<String>,
    /// Native hardware/model identifier when available.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model_identifier: Option<String>,
    /// Coarse category from `GATEWAY_CLIENT_MODES` for policy and diagnostics.
    pub mode: GatewayClientMode,
    /// Per-installation or per-process id used to distinguish same-product clients.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instance_id: Option<String>,
}

/// Capability flags a client may advertise during the gateway handshake.
pub mod gateway_client_caps {
    pub const INLINE_WIDGETS: &str = "inline-widgets";
    pub const TASK_SUGGESTIONS: &str = "task-suggestions";
    pub const TOOL_EVENTS: &str = "tool-events";
}

/// Optional capability advertised by clients during gateway handshake.
pub type GatewayClientCap = String;

/// Normalizes untrusted client ids and rejects unknown values.
pub fn normalize_gateway_client_id(raw: Option<&str>) -> Option<GatewayClientId> {
    let normalized = normalize_optional_lowercase_string(raw)?;
    if gateway_client_ids::all().contains(&normalized.as_str()) {
        Some(normalized)
    } else {
        None
    }
}

/// Normalizes legacy client-name fields through the canonical client-id registry.
pub fn normalize_gateway_client_name(raw: Option<&str>) -> Option<GatewayClientName> {
    normalize_gateway_client_id(raw)
}

/// Normalizes untrusted client modes and rejects unknown values.
pub fn normalize_gateway_client_mode(raw: Option<&str>) -> Option<GatewayClientMode> {
    let normalized = normalize_optional_lowercase_string(raw)?;
    if gateway_client_modes::all().contains(&normalized.as_str()) {
        Some(normalized)
    } else {
        None
    }
}

/// Checks a client-advertised capability list without treating missing caps as errors.
pub fn has_gateway_client_cap(caps: Option<&[String]>, cap: &str) -> bool {
    match caps {
        Some(arr) => arr.iter().any(|c| c == cap),
        None => false,
    }
}
