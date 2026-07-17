// Gateway Protocol schema: frames.
// 翻译自 packages/gateway-protocol/src/schema/frames.ts
//
// TS 用 TypeBox 定义 schema（运行时验证 + 类型）。
// Rust 用 serde struct 实现等价的序列化/反序列化语义。

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Gateway server capabilities
pub mod gateway_server_caps {
    pub const CHAT_SEND_ROUTING_CONTRACT: &str = "chat-send-routing-contract";
    pub const CRESTODIAN_SETUP_MODEL_REF: &str = "crestodian-setup-model-ref";
}

/// Periodic server heartbeat event payload.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TickEvent {
    pub ts: i64,
}

/// Server shutdown notice event payload.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ShutdownEvent {
    pub reason: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub restart_expected_ms: Option<i64>,
}

/// Device identity sent during connect handshake.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeviceIdentity {
    pub id: String,
    pub public_key: String,
    pub signature: String,
    pub signed_at: i64,
    pub nonce: String,
}

/// Auth credentials sent during connect handshake.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ConnectAuth {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bootstrap_token: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub device_token: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub approval_runtime_token: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "agentRuntimeIdentityToken")]
    pub agent_runtime_identity_token: Option<String>,
}

/// Initial client hello/connect payload sent before the gateway accepts frames.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectParams {
    pub min_protocol: i64,
    pub max_protocol: i64,
    pub client: crate::client_info::GatewayClientInfo,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub caps: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub commands: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub permissions: Option<HashMap<String, bool>>,
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "pathEnv")]
    pub path_env: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scopes: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub device: Option<DeviceIdentity>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub auth: Option<ConnectAuth>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub locale: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "userAgent")]
    pub user_agent: Option<String>,
}

/// Server info in hello-ok response.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ServerInfo {
    pub version: String,
    pub conn_id: String,
}

/// Feature discovery in hello-ok response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Features {
    pub methods: Vec<String>,
    pub events: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub capabilities: Option<Vec<String>>,
}

/// Control UI tab descriptor (additive plugin-declared tabs).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ControlUiTab {
    pub plugin_id: String,
    pub id: String,
    pub label: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub group: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order: Option<f64>,
}

/// Bounded device token issued during bootstrap.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeviceTokenEntry {
    pub device_token: String,
    pub role: String,
    pub scopes: Vec<String>,
    pub issued_at_ms: i64,
}

/// Auth info in hello-ok response.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HelloAuth {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub device_token: Option<String>,
    pub role: String,
    pub scopes: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub issued_at_ms: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub device_tokens: Option<Vec<DeviceTokenEntry>>,
}

/// Gateway policy limits.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Policy {
    pub max_payload: i64,
    pub max_buffered_bytes: i64,
    pub tick_interval_ms: i64,
}

/// Successful gateway hello response with negotiated protocol and initial state.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HelloOk {
    #[serde(rename = "type")]
    pub ok_type: String, // "hello-ok"
    pub protocol: i64,
    pub server: ServerInfo,
    pub features: Features,
    pub snapshot: serde_json::Value, // SnapshotSchema is complex; keep as Value for now
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub control_ui_tabs: Option<Vec<ControlUiTab>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub plugin_surface_urls: Option<HashMap<String, String>>,
    pub auth: HelloAuth,
    pub policy: Policy,
}

/// Standard structured error shape used in response frames and connect failures.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorShape {
    pub code: String,
    pub message: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub retryable: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "retryAfterMs")]
    pub retry_after_ms: Option<i64>,
}

/// Client request frame envelope; `method` selects the payload validator.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestFrame {
    #[serde(rename = "type")]
    pub frame_type: String, // "req"
    pub id: String,
    pub method: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub params: Option<serde_json::Value>,
}

/// Server response frame envelope paired with a prior request id.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseFrame {
    #[serde(rename = "type")]
    pub frame_type: String, // "res"
    pub id: String,
    pub ok: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub payload: Option<serde_json::Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<ErrorShape>,
}

/// Server event frame envelope; `event` selects the payload validator.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventFrame {
    #[serde(rename = "type")]
    pub frame_type: String, // "event"
    pub event: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub payload: Option<serde_json::Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub seq: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "stateVersion")]
    pub state_version: Option<i64>,
}

/// Discriminated union of all top-level frames.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum GatewayFrame {
    #[serde(rename = "req")]
    Request(RequestFrame),
    #[serde(rename = "res")]
    Response(ResponseFrame),
    #[serde(rename = "event")]
    Event(EventFrame),
}
