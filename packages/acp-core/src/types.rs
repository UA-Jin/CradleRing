// ACP Core type module defines shared TypeScript contracts.
// 翻译自 packages/acp-core/src/types.ts

use crate::normalize_text;
use normalization_core::string_coerce;
use serde_json::Value;

const ACP_PROVENANCE_MODE_VALUES: &[&str] = &["off", "meta", "meta+receipt"];

pub type SessionId = String;

pub type AcpProvenanceMode = String;

/// Normalizes ACP provenance mode from string input.
pub fn normalize_acp_provenance_mode(value: Option<&str>) -> Option<AcpProvenanceMode> {
    let wrapped = match value {
        Some(s) => Value::String(s.to_string()),
        None => Value::Null,
    };
    let normalized = string_coerce::normalize_optional_lowercase_string(&wrapped);
    match normalized {
        Some(n) if ACP_PROVENANCE_MODE_VALUES.contains(&n.as_str()) => Some(n),
        _ => None,
    }
}

/// AbortController shim mirroring the Node.js Web AbortController API.
#[derive(Debug, Clone, Default)]
pub struct AbortController {
    pub signal: AbortSignal,
}

impl AbortController {
    pub fn new() -> Self {
        Self {
            signal: AbortSignal { aborted: false },
        }
    }

    pub fn abort(&mut self) {
        self.signal.aborted = true;
    }
}

#[derive(Debug, Clone, Default)]
pub struct AbortSignal {
    pub aborted: bool,
}

#[derive(Debug, Clone, Default)]
pub struct AcpSession {
    pub session_id: SessionId,
    pub session_key: String,
    pub ledger_session_id: Option<String>,
    pub cwd: String,
    pub created_at: i64,
    pub last_touched_at: i64,
    pub abort_controller: Option<AbortController>,
    pub active_run_id: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct SessionCreateRateLimit {
    pub max_requests: Option<u32>,
    pub window_ms: Option<u64>,
}

#[derive(Debug, Clone, Default)]
pub struct AcpServerOptions {
    pub gateway_url: Option<String>,
    pub gateway_token: Option<String>,
    pub gateway_password: Option<String>,
    pub default_session_key: Option<String>,
    pub default_session_label: Option<String>,
    pub require_existing_session: Option<bool>,
    pub reset_session: Option<bool>,
    pub prefix_cwd: Option<bool>,
    pub provenance_mode: Option<AcpProvenanceMode>,
    pub session_create_rate_limit: Option<SessionCreateRateLimit>,
    pub verbose: Option<bool>,
}

pub type SessionAcpIdentitySource = String;
pub type SessionAcpIdentityState = String;

#[derive(Debug, Clone, Default)]
pub struct SessionAcpIdentity {
    /// Pending identities may expose provisional ids; resolved identities are safe for resume output.
    pub state: SessionAcpIdentityState,
    pub acpx_record_id: Option<String>,
    pub acpx_session_id: Option<String>,
    pub agent_session_id: Option<String>,
    /// Runtime lifecycle point that last supplied the identity fields.
    pub source: SessionAcpIdentitySource,
    pub last_updated_at: i64,
}

#[derive(Debug, Clone, Default)]
pub struct AcpSessionRuntimeOptions {
    /// ACP runtime mode set via session/set_mode (for example: "plan", "normal", "auto").
    pub runtime_mode: Option<String>,
    /// ACP runtime config option: model id.
    pub model: Option<String>,
    /// ACP runtime config option: thinking/reasoning effort.
    pub thinking: Option<String>,
    /// Working directory override for ACP session turns.
    pub cwd: Option<String>,
    /// ACP runtime config option: permission profile id.
    pub permission_profile: Option<String>,
    /// ACP runtime config option: per-turn timeout in seconds.
    pub timeout_seconds: Option<i64>,
    /// Backend-specific option bag mapped through session/set_config_option.
    pub backend_extras: Option<std::collections::HashMap<String, String>>,
}

#[derive(Debug, Clone, Default)]
pub enum AcpSessionMode {
    #[default]
    Persistent,
    Oneshot,
}

#[derive(Debug, Clone, Default)]
pub enum AcpSessionState {
    #[default]
    Idle,
    Running,
    Error,
}

#[derive(Debug, Clone, Default)]
pub struct SessionAcpMeta {
    pub backend: String,
    pub agent: String,
    pub runtime_session_name: String,
    /// Canonical backend/agent ids used for resume hints and thread/status details.
    pub identity: Option<SessionAcpIdentity>,
    pub mode: AcpSessionMode,
    pub runtime_options: Option<AcpSessionRuntimeOptions>,
    pub cwd: Option<String>,
    pub state: AcpSessionState,
    pub last_activity_at: i64,
    pub last_error: Option<String>,
}

// keep normalize_text re-export reference available for the crate root
#[allow(dead_code)]
fn _force_use() -> Option<String> {
    normalize_text::normalize_text(&Value::Null)
}