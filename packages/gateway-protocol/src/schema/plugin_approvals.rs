// Gateway Protocol schema: plugin_approvals.
// 翻译自 packages/gateway-protocol/src/schema/plugin-approvals.ts
//
// Plugin approval schemas.
//
// These payloads cross from plugin/tool execution into reviewer-facing UI, so
// title, description, decision set, and timeout limits are part of the public
// gateway contract.
//
// TS 用 TypeBox 定义 schema（运行时验证 + 类型）。
// Rust 用 serde struct + 验证函数实现等价的序列化/反序列化语义。

use serde::{Deserialize, Serialize};

use super::primitives::NonEmptyString;

// ---------- Module-private bounds ----------

/// Maximum approval timeout (10 minutes), mirroring `MAX_PLUGIN_APPROVAL_TIMEOUT_MS`.
/// 对齐 TS: `MAX_PLUGIN_APPROVAL_TIMEOUT_MS = 600_000`.
pub const MAX_PLUGIN_APPROVAL_TIMEOUT_MS: i64 = 600_000;
pub const PLUGIN_APPROVAL_TIMEOUT_MIN_MS: i64 = 1;

/// Title length bound (UTF-8 chars).
/// 对齐 TS: `PLUGIN_APPROVAL_TITLE_MAX_LENGTH = 80`.
pub const PLUGIN_APPROVAL_TITLE_MAX_LENGTH: usize = 80;
pub const PLUGIN_APPROVAL_TITLE_MIN_LENGTH: usize = 1;

/// Description length bound (UTF-8 chars).
/// 对齐 TS: `PLUGIN_APPROVAL_DESCRIPTION_MAX_LENGTH = 512`.
pub const PLUGIN_APPROVAL_DESCRIPTION_MAX_LENGTH: usize = 512;
pub const PLUGIN_APPROVAL_DESCRIPTION_MIN_LENGTH: usize = 1;

/// Allowed-decision list bounds.
/// 对齐 TS: `Type.Array(..., { minItems: 1, maxItems: 3 })`.
pub const PLUGIN_APPROVAL_ALLOWED_DECISIONS_MIN_ITEMS: usize = 1;
pub const PLUGIN_APPROVAL_ALLOWED_DECISIONS_MAX_ITEMS: usize = 3;

// ---------- Closed enums ----------

/// Severity levels reported alongside plugin approval requests.
/// 对齐 TS: `Type.String({ enum: ["info", "warning", "critical"] })`.
pub mod plugin_approval_severities {
    pub const INFO: &str = "info";
    pub const WARNING: &str = "warning";
    pub const CRITICAL: &str = "critical";

    pub fn all() -> &'static [&'static str] {
        &[INFO, WARNING, CRITICAL]
    }

    pub fn from_str(s: &str) -> Option<&'static str> {
        match s {
            "info" => Some(INFO),
            "warning" => Some(WARNING),
            "critical" => Some(CRITICAL),
            _ => None,
        }
    }
}

pub fn is_valid_plugin_approval_severity(s: &str) -> bool {
    plugin_approval_severities::from_str(s).is_some()
}

/// Severity levels reported alongside plugin approval requests.
/// 对齐 TS: `Type.String({ enum: ["info", "warning", "critical"] })`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PluginApprovalSeverity {
    #[serde(rename = "info")]
    Info,
    #[serde(rename = "warning")]
    Warning,
    #[serde(rename = "critical")]
    Critical,
}

impl PluginApprovalSeverity {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Info => "info",
            Self::Warning => "warning",
            Self::Critical => "critical",
        }
    }
}

/// Reviewer decisions accepted for plugin approvals.
/// 对齐 TS: `Type.String({ enum: ["allow-once", "allow-always", "deny"] })`.
pub mod plugin_approval_decisions {
    pub const ALLOW_ONCE: &str = "allow-once";
    pub const ALLOW_ALWAYS: &str = "allow-always";
    pub const DENY: &str = "deny";

    pub fn all() -> &'static [&'static str] {
        &[ALLOW_ONCE, ALLOW_ALWAYS, DENY]
    }

    pub fn from_str(s: &str) -> Option<&'static str> {
        match s {
            "allow-once" => Some(ALLOW_ONCE),
            "allow-always" => Some(ALLOW_ALWAYS),
            "deny" => Some(DENY),
            _ => None,
        }
    }
}

pub fn is_valid_plugin_approval_decision(s: &str) -> bool {
    plugin_approval_decisions::from_str(s).is_some()
}

/// Reviewer decisions accepted for plugin approvals.
/// 对齐 TS: `Type.String({ enum: ["allow-once", "allow-always", "deny"] })`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PluginApprovalDecision {
    #[serde(rename = "allow-once")]
    AllowOnce,
    #[serde(rename = "allow-always")]
    AllowAlways,
    #[serde(rename = "deny")]
    Deny,
}

impl PluginApprovalDecision {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::AllowOnce => "allow-once",
            Self::AllowAlways => "allow-always",
            Self::Deny => "deny",
        }
    }
}

// ---------- 基础验证原语 ----------

/// 对齐 TS: `NonEmptyString = Type.String({ minLength: 1 })`
fn is_non_empty_string(s: &str) -> bool {
    !s.is_empty()
}

fn validate_non_empty_string(field: &str, value: &str) -> Result<(), String> {
    if is_non_empty_string(value) {
        Ok(())
    } else {
        Err(format!(
            "{}: expected non-empty string, got {:?}",
            field, value
        ))
    }
}

fn validate_optional_non_empty_string(field: &str, value: Option<&str>) -> Result<(), String> {
    if let Some(s) = value {
        validate_non_empty_string(field, s)?;
    }
    Ok(())
}

fn validate_optional_non_empty_string_list(
    field: &str,
    values: Option<&Vec<String>>,
) -> Result<(), String> {
    if let Some(arr) = values {
        for (i, v) in arr.iter().enumerate() {
            validate_non_empty_string(&format!("{}[{}]", field, i), v)?;
        }
    }
    Ok(())
}

/// 对齐 TS: `Type.String({ minLength: 1, maxLength: 80 })` (title).
fn validate_plugin_approval_title(field: &str, value: &str) -> Result<(), String> {
    let len = value.chars().count();
    if len < PLUGIN_APPROVAL_TITLE_MIN_LENGTH || len > PLUGIN_APPROVAL_TITLE_MAX_LENGTH {
        return Err(format!(
            "{}: expected length [{}, {}], got {}",
            field, PLUGIN_APPROVAL_TITLE_MIN_LENGTH, PLUGIN_APPROVAL_TITLE_MAX_LENGTH, len
        ));
    }
    Ok(())
}

/// 对齐 TS: `Type.String({ minLength: 1, maxLength: 512 })` (description).
fn validate_plugin_approval_description(field: &str, value: &str) -> Result<(), String> {
    let len = value.chars().count();
    if len < PLUGIN_APPROVAL_DESCRIPTION_MIN_LENGTH
        || len > PLUGIN_APPROVAL_DESCRIPTION_MAX_LENGTH
    {
        return Err(format!(
            "{}: expected length [{}, {}], got {}",
            field,
            PLUGIN_APPROVAL_DESCRIPTION_MIN_LENGTH,
            PLUGIN_APPROVAL_DESCRIPTION_MAX_LENGTH,
            len
        ));
    }
    Ok(())
}

/// 对齐 TS: `Type.Integer({ minimum: 1, maximum: 600_000 })` (timeoutMs).
fn validate_plugin_approval_timeout(field: &str, n: i64) -> Result<(), String> {
    if n < PLUGIN_APPROVAL_TIMEOUT_MIN_MS || n > MAX_PLUGIN_APPROVAL_TIMEOUT_MS {
        return Err(format!(
            "{}: expected integer in [{}, {}], got {}",
            field, PLUGIN_APPROVAL_TIMEOUT_MIN_MS, MAX_PLUGIN_APPROVAL_TIMEOUT_MS, n
        ));
    }
    Ok(())
}

/// 对齐 TS: `Type.Array(..., { minItems: 1, maxItems: 3 })`.
fn validate_plugin_approval_allowed_decisions(values: &[String]) -> Result<(), String> {
    if values.len() < PLUGIN_APPROVAL_ALLOWED_DECISIONS_MIN_ITEMS
        || values.len() > PLUGIN_APPROVAL_ALLOWED_DECISIONS_MAX_ITEMS
    {
        return Err(format!(
            "allowedDecisions: expected length [{}, {}], got {}",
            PLUGIN_APPROVAL_ALLOWED_DECISIONS_MIN_ITEMS,
            PLUGIN_APPROVAL_ALLOWED_DECISIONS_MAX_ITEMS,
            values.len()
        ));
    }
    for (i, v) in values.iter().enumerate() {
        if !is_valid_plugin_approval_decision(v) {
            return Err(format!(
                "allowedDecisions[{}]: expected one of {:?}, got {:?}",
                i,
                plugin_approval_decisions::all(),
                v
            ));
        }
    }
    Ok(())
}

// ---------- TurnSourceThreadIdSchema ----------

/// Either a string id or a numeric id used for the originating thread.
/// 对齐 TS: `Type.Optional(Type.Union([Type.String(), Type.Number()]))`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum TurnSourceThreadIdSchema {
    String(String),
    Number(f64),
}

impl TurnSourceThreadIdSchema {
    pub fn validate(&self) -> Result<(), String> {
        // Both variants accept any value (string / finite number).
        match self {
            TurnSourceThreadIdSchema::String(_) => Ok(()),
            TurnSourceThreadIdSchema::Number(n) => {
                if n.is_finite() {
                    Ok(())
                } else {
                    Err(format!("turnSourceThreadId: expected finite number, got {}", n))
                }
            }
        }
    }
}

// ---------- PluginApprovalRequestParamsSchema ----------

/// Approval request raised by a plugin before a sensitive tool action proceeds.
/// 对齐 TS:
///   `Type.Object({
///      pluginId:                  Type.Optional(NonEmptyString),
///      title:                     Type.String({ minLength: 1, maxLength: 80 }),
///      description:               Type.String({ minLength: 1, maxLength: 512 }),
///      severity:                  Type.Optional(Type.String({ enum: [...] })),
///      toolName:                  Type.Optional(Type.String()),
///      toolCallId:                Type.Optional(Type.String()),
///      allowedDecisions:          Type.Optional(
///        Type.Array(Type.String({ enum: [...] }), { minItems: 1, maxItems: 3 })
///      ),
///      agentId:                   Type.Optional(Type.String()),
///      sessionKey:                Type.Optional(Type.String()),
///      approvalReviewerDeviceIds: Type.Optional(Type.Array(NonEmptyString, ...)),
///      turnSourceChannel:         Type.Optional(Type.String()),
///      turnSourceTo:              Type.Optional(Type.String()),
///      turnSourceAccountId:       Type.Optional(Type.String()),
///      turnSourceThreadId:        Type.Optional(Type.Union([Type.String(), Type.Number()])),
///      timeoutMs:                 Type.Optional(Type.Integer({ minimum: 1, maximum: 600_000 })),
///      twoPhase:                  Type.Optional(Type.Boolean()),
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginApprovalRequestParams {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub plugin_id: Option<NonEmptyString>,
    pub title: String,
    pub description: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub severity: Option<PluginApprovalSeverity>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub allowed_decisions: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub agent_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub session_key: Option<String>,
    /// Trusted approval-runtime metadata naming operator devices that may review
    /// this approval; ordinary Gateway clients may send the field, but the
    /// Gateway only binds it for internal approval-runtime requests.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub approval_reviewer_device_ids: Option<Vec<NonEmptyString>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub turn_source_channel: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub turn_source_to: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub turn_source_account_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub turn_source_thread_id: Option<TurnSourceThreadIdSchema>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub timeout_ms: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub two_phase: Option<bool>,
}

impl PluginApprovalRequestParams {
    pub fn validate(&self) -> Result<(), String> {
        validate_optional_non_empty_string("pluginId", self.plugin_id.as_deref())?;
        validate_plugin_approval_title("title", &self.title)?;
        validate_plugin_approval_description("description", &self.description)?;
        if let Some(allowed) = &self.allowed_decisions {
            validate_plugin_approval_allowed_decisions(allowed)?;
        }
        validate_optional_non_empty_string_list(
            "approvalReviewerDeviceIds",
            self.approval_reviewer_device_ids.as_ref(),
        )?;
        if let Some(n) = self.timeout_ms {
            validate_plugin_approval_timeout("timeoutMs", n)?;
        }
        if let Some(thread_id) = &self.turn_source_thread_id {
            thread_id
                .validate()
                .map_err(|e| format!("turnSourceThreadId: {}", e))?;
        }
        Ok(())
    }
}

// ---------- PluginApprovalResolveParamsSchema ----------

/// Reviewer decision payload resolving one pending plugin approval request.
/// 对齐 TS:
///   `Type.Object({
///      id:       NonEmptyString,
///      decision: NonEmptyString,
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginApprovalResolveParams {
    pub id: NonEmptyString,
    pub decision: NonEmptyString,
}

impl PluginApprovalResolveParams {
    pub fn validate(&self) -> Result<(), String> {
        validate_non_empty_string("id", &self.id)?;
        validate_non_empty_string("decision", &self.decision)?;
        Ok(())
    }
}

// Owner-local wire types derived directly from local schema consts so the
// public plugin-sdk declaration graph never pulls in the ProtocolSchemas registry.
// 对应 TS:
//   export type PluginApprovalRequestParams = Static<typeof PluginApprovalRequestParamsSchema>;
//   export type PluginApprovalResolveParams = Static<typeof PluginApprovalResolveParamsSchema>;
pub type PluginApprovalRequestParamsType = PluginApprovalRequestParams;
pub type PluginApprovalResolveParamsType = PluginApprovalResolveParams;