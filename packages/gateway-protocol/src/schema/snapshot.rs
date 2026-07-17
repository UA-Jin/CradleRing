// Gateway Protocol schema: snapshot.
// 翻译自 packages/gateway-protocol/src/schema/snapshot.ts
//
// TS 用 TypeBox 定义 schema（运行时验证 + 类型）。
// Rust 用 serde struct + 验证函数实现等价的序列化/反序列化语义。
//
// Gateway state snapshot schemas.
// Snapshots are sent during hello and later event streams; they summarize node
// presence, health, session defaults, and version counters for clients.

use serde::{Deserialize, Serialize};
use serde_json::Value;

// ---------- 基础验证原语 (对齐 TypeBox: NonEmptyString / Integer{min:0}) ----------

/// 对齐 TS: `NonEmptyString = Type.String({ minLength: 1 })`
fn is_non_empty_string(s: &str) -> bool {
    !s.trim().is_empty()
}

fn validate_non_empty_string(s: &str) -> Result<(), String> {
    if is_non_empty_string(s) {
        Ok(())
    } else {
        Err(format!("expected non-empty string, got {:?}", s))
    }
}

fn validate_optional_non_empty_string(value: Option<&str>) -> Result<(), String> {
    if let Some(s) = value {
        validate_non_empty_string(s)?;
    }
    Ok(())
}

fn validate_non_empty_string_list(values: &[String]) -> Result<(), String> {
    for (i, v) in values.iter().enumerate() {
        validate_non_empty_string(v)
            .map_err(|e| format!("index {}: {}", i, e))?;
    }
    Ok(())
}

fn validate_optional_non_empty_string_list(value: Option<&[String]>) -> Result<(), String> {
    if let Some(arr) = value {
        validate_non_empty_string_list(arr)?;
    }
    Ok(())
}

/// 对齐 TS: `Type.Integer({ minimum: 0 })`
fn validate_non_negative_integer(n: i64) -> Result<(), String> {
    if n >= 0 {
        Ok(())
    } else {
        Err(format!("expected integer >= 0, got {}", n))
    }
}

fn validate_optional_non_negative_integer(value: Option<i64>) -> Result<(), String> {
    if let Some(n) = value {
        validate_non_negative_integer(n)?;
    }
    Ok(())
}

// ---------- PresenceEntry ----------

/// One gateway-visible presence record for a node/client/runtime.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PresenceEntry {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub host: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ip: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub platform: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub device_family: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model_identifier: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mode: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_input_seconds: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,
    /// 对齐 TS: `text: Type.Optional(Type.String())` —— 普通可选字符串，不强制非空。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    pub ts: i64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub device_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub roles: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scopes: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub instance_id: Option<String>,
}

impl PresenceEntry {
    pub fn validate(&self) -> Result<(), String> {
        validate_optional_non_empty_string(self.host.as_deref())?;
        validate_optional_non_empty_string(self.ip.as_deref())?;
        validate_optional_non_empty_string(self.version.as_deref())?;
        validate_optional_non_empty_string(self.platform.as_deref())?;
        validate_optional_non_empty_string(self.device_family.as_deref())?;
        validate_optional_non_empty_string(self.model_identifier.as_deref())?;
        validate_optional_non_empty_string(self.mode.as_deref())?;
        validate_optional_non_negative_integer(self.last_input_seconds)?;
        validate_optional_non_empty_string(self.reason.as_deref())?;
        validate_optional_non_empty_string_list(self.tags.as_deref())?;
        validate_optional_non_empty_string_list(self.roles.as_deref())?;
        validate_optional_non_empty_string_list(self.scopes.as_deref())?;
        validate_optional_non_empty_string(self.device_id.as_deref())?;
        validate_optional_non_empty_string(self.instance_id.as_deref())?;
        validate_non_negative_integer(self.ts)?;
        Ok(())
    }
}

// ---------- HealthSnapshot ----------

/// Health snapshot is intentionally opaque because providers contribute nested shapes.
/// 对齐 TS: `HealthSnapshotSchema = Type.Any()` —— 任意 JSON 值。
pub type HealthSnapshot = Value;

// ---------- SessionDefaults ----------

/// Default session routing keys included in initial gateway snapshots.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionDefaults {
    pub default_agent_id: String,
    pub main_key: String,
    pub main_session_key: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scope: Option<String>,
}

impl SessionDefaults {
    pub fn validate(&self) -> Result<(), String> {
        validate_non_empty_string(&self.default_agent_id)?;
        validate_non_empty_string(&self.main_key)?;
        validate_non_empty_string(&self.main_session_key)?;
        validate_optional_non_empty_string(self.scope.as_deref())?;
        Ok(())
    }
}

// ---------- StateVersion ----------

/// Monotonic version counters for snapshot subtrees.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateVersion {
    pub presence: i64,
    pub health: i64,
}

impl StateVersion {
    pub fn validate(&self) -> Result<(), String> {
        validate_non_negative_integer(self.presence)?;
        validate_non_negative_integer(self.health)?;
        Ok(())
    }
}

// ---------- AuthMode (union of literals) ----------

/// Authentication mode advertised in gateway snapshots.
/// 对齐 TS: `Type.Union([Type.Literal("none"), Type.Literal("token"),
///                       Type.Literal("password"), Type.Literal("trusted-proxy")])`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AuthMode {
    #[serde(rename = "none")]
    None,
    #[serde(rename = "token")]
    Token,
    #[serde(rename = "password")]
    Password,
    #[serde(rename = "trusted-proxy")]
    TrustedProxy,
}

impl AuthMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Token => "token",
            Self::Password => "password",
            Self::TrustedProxy => "trusted-proxy",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "none" => Some(Self::None),
            "token" => Some(Self::Token),
            "password" => Some(Self::Password),
            "trusted-proxy" => Some(Self::TrustedProxy),
            _ => None,
        }
    }

    pub fn all() -> &'static [AuthMode] {
        &[Self::None, Self::Token, Self::Password, Self::TrustedProxy]
    }
}

// ---------- UpdateAvailable ----------

/// Update metadata describing a newer available gateway version.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateAvailable {
    pub current_version: String,
    pub latest_version: String,
    pub channel: String,
}

impl UpdateAvailable {
    pub fn validate(&self) -> Result<(), String> {
        validate_non_empty_string(&self.current_version)?;
        validate_non_empty_string(&self.latest_version)?;
        validate_non_empty_string(&self.channel)?;
        Ok(())
    }
}

// ---------- Snapshot ----------

/// Initial and incremental gateway state snapshot payload.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Snapshot {
    pub presence: Vec<PresenceEntry>,
    pub health: HealthSnapshot,
    pub state_version: StateVersion,
    pub uptime_ms: i64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub config_path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub state_dir: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub session_defaults: Option<SessionDefaults>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub auth_mode: Option<AuthMode>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub update_available: Option<UpdateAvailable>,
}

impl Snapshot {
    pub fn validate(&self) -> Result<(), String> {
        for (i, entry) in self.presence.iter().enumerate() {
            entry
                .validate()
                .map_err(|e| format!("presence[{}]: {}", i, e))?;
        }
        validate_non_negative_integer(self.uptime_ms)?;
        validate_optional_non_empty_string(self.config_path.as_deref())?;
        validate_optional_non_empty_string(self.state_dir.as_deref())?;
        if let Some(sd) = &self.session_defaults {
            sd.validate().map_err(|e| format!("sessionDefaults: {}", e))?;
        }
        if let Some(ua) = &self.update_available {
            ua.validate().map_err(|e| format!("updateAvailable: {}", e))?;
        }
        self.state_version
            .validate()
            .map_err(|e| format!("stateVersion: {}", e))?;
        Ok(())
    }
}

// Wire types derive directly from local schema consts so public d.ts graphs never
// pull in the ProtocolSchemas registry.
// 对应 TS:
//   export type Snapshot = Static<typeof SnapshotSchema>;
//   export type PresenceEntry = Static<typeof PresenceEntrySchema>;
//   export type StateVersion = Static<typeof StateVersionSchema>;
pub type SnapshotType = Snapshot;
pub type PresenceEntryType = PresenceEntry;
pub type StateVersionType = StateVersion;
