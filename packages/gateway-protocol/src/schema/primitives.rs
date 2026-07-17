//! Gateway Protocol schema: primitives.
//! 翻译自 packages/gateway-protocol/src/schema/primitives.ts
//!
//! Shared schema primitives reused by gateway protocol request/result schemas.
//!
//! Keep these schemas small and transport-oriented; feature-specific validation
//! belongs in the owning schema module or runtime handler.
//!
//! TS 用 TypeBox 定义 schema（运行时验证 + 类型）。
//! Rust 用 serde struct + 验证函数实现等价语义。

use serde::{Deserialize, Serialize};

use crate::client_info::{gateway_client_ids, gateway_client_modes};

// ---------------------------------------------------------------------------
// Constants imported from secret-ref-contract.ts.
// Inlined here as `&str` regex sources to mirror the original TypeScript
// grammar exactly while keeping the primitives schema self-contained.
// (对应的 crate-level `secret_ref_contract` 模块尚未翻译——这里先内联保留。)
// ---------------------------------------------------------------------------

/// Canonical id for file secret providers that expose exactly one value.
pub const SINGLE_VALUE_FILE_REF_ID: &str = "value";

/// Source for the shared env/file/exec secret provider alias grammar.
pub const SECRET_PROVIDER_ALIAS_PATTERN: &str = r"^[a-z][a-z0-9_-]{0,63}$";

/// JSON-schema fragment that rejects absolute file secret ref ids.
pub const FILE_SECRET_REF_ID_ABSOLUTE_JSON_SCHEMA_PATTERN: &str = r"^/";

/// JSON-schema fragment that rejects invalid JSON-pointer escape sequences.
pub const FILE_SECRET_REF_ID_INVALID_ESCAPE_JSON_SCHEMA_PATTERN: &str = r"~(?:[^01]|$)";

/// JSON-schema pattern for exec secret ref ids, excluding dot-path traversal.
pub const EXEC_SECRET_REF_ID_JSON_SCHEMA_PATTERN: &str =
    r"^(?!.*(?:^|/)\.{1,2}(?:/|$))[A-Za-z0-9][A-Za-z0-9._:/#-]{0,255}$";

// ---------------------------------------------------------------------------
// Module-private schema constants
// ---------------------------------------------------------------------------

/// Env secret ref id grammar: `[A-Z][A-Z0-9_]{0,127}`.
const ENV_SECRET_REF_ID_RE_SOURCE: &str = r"^[A-Z][A-Z0-9_]{0,127}$";

/// Closed enumeration of allowed input provenance kinds.
const INPUT_PROVENANCE_KIND_VALUES: &[&str] = &[
    "external_user",
    "inter_session",
    "internal_system",
];

/// Maximum stable session label length.
const SESSION_LABEL_MAX_LENGTH: usize = 512;

// ---------------------------------------------------------------------------
// String primitives
// ---------------------------------------------------------------------------

/// Non-empty string primitive for protocol fields that reject blank values.
/// TS equivalent: `Type.String({ minLength: 1 })`.
pub type NonEmptyString = String;

/// Maximum stable session key length accepted by chat-send protocol requests.
pub const CHAT_SEND_SESSION_KEY_MAX_LENGTH: usize = 512;

/// Chat-send session key string primitive with bounded length.
/// TS equivalent: `Type.String({ minLength: 1, maxLength: 512 })`.
pub type ChatSendSessionKeyString = String;

/// Human-readable session label primitive with bounded display length.
/// TS equivalent: `Type.String({ minLength: 1, maxLength: 512 })`.
pub type SessionLabelString = String;

/// Returns true when `s` is a non-empty protocol string (TS `minLength: 1`).
pub fn is_non_empty_string(s: &str) -> bool {
    !s.is_empty()
}

/// Returns true when `s` fits the chat-send session key bounds.
pub fn is_valid_chat_send_session_key(s: &str) -> bool {
    !s.is_empty() && s.chars().count() <= CHAT_SEND_SESSION_KEY_MAX_LENGTH
}

/// Returns true when `s` fits the session label bounds.
pub fn is_valid_session_label(s: &str) -> bool {
    !s.is_empty() && s.chars().count() <= SESSION_LABEL_MAX_LENGTH
}

// ---------------------------------------------------------------------------
// InputProvenanceSchema
// ---------------------------------------------------------------------------

/// Returns true when `kind` is one of the closed enum values.
pub fn is_valid_input_provenance_kind(kind: &str) -> bool {
    INPUT_PROVENANCE_KIND_VALUES.contains(&kind)
}

/// Provenance marker for content copied from another user/session/system source.
/// TS equivalent: `Type.Object({ kind: String({enum: [...]}), ... }, {additionalProperties: false})`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InputProvenanceSchema {
    pub kind: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub origin_session_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_session_key: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_channel: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_tool: Option<String>,
}

impl InputProvenanceSchema {
    /// Runtime validation mirroring the TS schema (`kind` enum; all other fields
    /// are optional strings). Returns `Err` on the first failure.
    pub fn validate(&self) -> Result<(), String> {
        if !is_valid_input_provenance_kind(&self.kind) {
            return Err(format!("invalid input provenance kind: {}", self.kind));
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// GatewayClientIdSchema / GatewayClientModeSchema
// ---------------------------------------------------------------------------

/// Closed gateway client id schema aligned with `GATEWAY_CLIENT_IDS`.
/// TS equivalent: `Type.Enum(GATEWAY_CLIENT_IDS)`.
pub type GatewayClientIdSchema = String;

/// Closed gateway client mode schema aligned with `GATEWAY_CLIENT_MODES`.
/// TS equivalent: `Type.Enum(GATEWAY_CLIENT_MODES)`.
pub type GatewayClientModeSchema = String;

/// Returns true when `id` is one of the canonical gateway client ids.
pub fn is_valid_gateway_client_id(id: &str) -> bool {
    gateway_client_ids::all().contains(&id)
}

/// Returns true when `mode` is one of the canonical gateway client modes.
pub fn is_valid_gateway_client_mode(mode: &str) -> bool {
    gateway_client_modes::all().contains(&mode)
}

// ---------------------------------------------------------------------------
// SecretRefSourceSchema
// ---------------------------------------------------------------------------

/// Closed enumeration of secret-ref backing stores.
pub mod secret_ref_sources {
    pub const ENV: &str = "env";
    pub const FILE: &str = "file";
    pub const EXEC: &str = "exec";

    /// All allowed source values.
    pub fn all() -> &'static [&'static str] {
        &[ENV, FILE, EXEC]
    }

    /// Parse a secret ref source string; returns `None` for unknown values.
    pub fn from_str(s: &str) -> Option<&'static str> {
        match s {
            "env" => Some(ENV),
            "file" => Some(FILE),
            "exec" => Some(EXEC),
            _ => None,
        }
    }
}

/// Supported secret reference backing stores for protocol SecretRef payloads.
/// TS equivalent: `Type.Union([Type.Literal("env"), Type.Literal("file"), Type.Literal("exec")])`.
pub type SecretRefSourceSchema = String;

/// Returns true when `source` is a recognized SecretRef backing store.
pub fn is_valid_secret_ref_source(source: &str) -> bool {
    secret_ref_sources::from_str(source).is_some()
}

// ---------------------------------------------------------------------------
// SecretRef (discriminated union over `source`)
// TS: `Type.Union([EnvSecretRefSchema, FileSecretRefSchema, ExecSecretRefSchema])`
// Each variant in TS has `source` (literal), `provider` (alias), `id` (pattern).
// ---------------------------------------------------------------------------

fn regex(pattern: &str) -> regex::Regex {
    regex::Regex::new(pattern).expect("invalid regex pattern compiled into primitives")
}

/// Returns true when `alias` matches the secret provider alias grammar.
pub fn is_valid_secret_provider_alias(alias: &str) -> bool {
    regex(SECRET_PROVIDER_ALIAS_PATTERN).is_match(alias)
}

/// Returns true when `id` matches the env secret ref id grammar.
pub fn is_valid_env_secret_ref_id(id: &str) -> bool {
    regex(ENV_SECRET_REF_ID_RE_SOURCE).is_match(id)
}

/// Returns true when `id` matches the file secret ref id schema
/// (`{ const: SINGLE_VALUE_FILE_REF_ID }` OR
/// `allOf [{ pattern: absolute }, { not: { pattern: invalid_escape } }]`).
pub fn is_valid_file_secret_ref_id(id: &str) -> bool {
    if id == SINGLE_VALUE_FILE_REF_ID {
        return true;
    }
    let absolute = regex(FILE_SECRET_REF_ID_ABSOLUTE_JSON_SCHEMA_PATTERN);
    let invalid_escape = regex(FILE_SECRET_REF_ID_INVALID_ESCAPE_JSON_SCHEMA_PATTERN);
    absolute.is_match(id) && !invalid_escape.is_match(id)
}

/// Returns true when `id` matches the exec secret ref id grammar.
pub fn is_valid_exec_secret_ref_id(id: &str) -> bool {
    regex(EXEC_SECRET_REF_ID_JSON_SCHEMA_PATTERN).is_match(id)
}

/// Structured secret reference accepted by config and channel protocol payloads.
/// TS equivalent: discriminated `Type.Union` keyed by `source`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "source", rename_all = "snake_case")]
pub enum SecretRefSchema {
    Env { provider: String, id: String },
    File { provider: String, id: String },
    Exec { provider: String, id: String },
}

impl SecretRefSchema {
    /// Runtime validation mirroring the TS `SecretRefSchema` constraints.
    pub fn validate(&self) -> Result<(), String> {
        let (provider, _id): (&str, &str) = match self {
            SecretRefSchema::Env { provider, id } => (provider, id),
            SecretRefSchema::File { provider, id } => (provider, id),
            SecretRefSchema::Exec { provider, id } => (provider, id),
        };
        if !is_valid_secret_provider_alias(provider) {
            return Err(format!("invalid secret provider alias: {}", provider));
        }
        match self {
            SecretRefSchema::Env { id, .. } => {
                if !is_valid_env_secret_ref_id(id) {
                    return Err(format!("invalid env secret ref id: {}", id));
                }
            }
            SecretRefSchema::File { id, .. } => {
                if !is_valid_file_secret_ref_id(id) {
                    return Err(format!("invalid file secret ref id: {}", id));
                }
            }
            SecretRefSchema::Exec { id, .. } => {
                if !is_valid_exec_secret_ref_id(id) {
                    return Err(format!("invalid exec secret ref id: {}", id));
                }
            }
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// SecretInputSchema
// TS: `Type.Union([Type.String(), SecretRefSchema])`
// ---------------------------------------------------------------------------

/// Secret input value: either an inline string or a structured SecretRef.
/// TS equivalent: `Type.Union([Type.String(), SecretRefSchema])`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum SecretInputSchema {
    Inline(String),
    Ref(SecretRefSchema),
}
