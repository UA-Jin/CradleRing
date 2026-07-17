// Gateway Protocol schema: approvals.
// 翻译自 packages/gateway-protocol/src/schema/approvals.ts
//
// Gateway Protocol schema module defines durable cross-surface approval shapes.
//
// TS 用 TypeBox 定义 schema（运行时验证 + 类型）。
// Rust 用 serde struct + 验证函数实现等价的序列化/反序列化语义。

use serde::{Deserialize, Serialize};

use super::approval_id::{is_well_formed_approval_id, APPROVAL_ID_WELL_FORMED_UNICODE_PATTERN};

/// Re-export of the upstream pattern constant for downstream callers
/// (mirrors the TS `APPROVAL_ID_WELL_FORMED_UNICODE_PATTERN` export).
pub const APPROVAL_ID_WELL_FORMED_UNICODE_PATTERN_RE: &str = APPROVAL_ID_WELL_FORMED_UNICODE_PATTERN;
use super::plugin_approvals::PluginApprovalSeverity;
use super::primitives::{is_non_empty_string, NonEmptyString};

// ---------- Module-private bounds ----------

/// Minimum length of an approval title.
const PLUGIN_APPROVAL_TITLE_MIN_LENGTH: usize = 1;
const PLUGIN_APPROVAL_TITLE_MAX_LENGTH: usize = 80;

/// Minimum length of a plugin approval description.
const PLUGIN_APPROVAL_DESCRIPTION_MIN_LENGTH: usize = 1;
const PLUGIN_APPROVAL_DESCRIPTION_MAX_LENGTH: usize = 512;

/// Bounds on `allowedDecisions` arrays.
const APPROVAL_ALLOWED_DECISIONS_MIN_ITEMS: usize = 1;
const APPROVAL_ALLOWED_DECISIONS_MAX_ITEMS: usize = 3;

// ---------- ApprovalIdSchema (private) ----------

/// Approval id schema, gated by the well-formed Unicode pattern.
/// 对齐 TS:
///   `ApprovalIdSchema = Type.String({
///      minLength: 1,
///      pattern: APPROVAL_ID_WELL_FORMED_UNICODE_PATTERN,
///      description: "Exact full approval id encoded safely in deep-link paths."
///   })`.
pub type ApprovalIdSchema = String;

fn validate_approval_id(field: &str, value: &str) -> Result<(), String> {
    if !is_well_formed_approval_id(value) {
        return Err(format!(
            "{}: expected well-formed approval id (non-empty, no \".\"/\"..\", no unpaired surrogates), got {:?}",
            field, value
        ));
    }
    Ok(())
}

// ---------- Closed enums ----------

/// Approval owner used to select the safe presentation payload.
/// 对齐 TS: `Type.Union([Type.Literal("exec"), Type.Literal("plugin")])`.
pub mod approval_kinds {
    pub const EXEC: &str = "exec";
    pub const PLUGIN: &str = "plugin";

    pub fn all() -> &'static [&'static str] {
        &[EXEC, PLUGIN]
    }

    pub fn from_str(s: &str) -> Option<&'static str> {
        match s {
            "exec" => Some(EXEC),
            "plugin" => Some(PLUGIN),
            _ => None,
        }
    }
}

pub fn is_valid_approval_kind(s: &str) -> bool {
    approval_kinds::from_str(s).is_some()
}

/// Approval owner used to select the safe presentation payload.
/// 对齐 TS: `Type.Union([Type.Literal("exec"), Type.Literal("plugin")])`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ApprovalKind {
    #[serde(rename = "exec")]
    Exec,
    #[serde(rename = "plugin")]
    Plugin,
}

impl ApprovalKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Exec => "exec",
            Self::Plugin => "plugin",
        }
    }
}

/// Reviewer decisions accepted by the unified approval resolver.
/// 对齐 TS:
///   `Type.Union([Type.Literal("allow-once"), Type.Literal("allow-always"), Type.Literal("deny")])`.
pub mod approval_decisions {
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

pub fn is_valid_approval_decision(s: &str) -> bool {
    approval_decisions::from_str(s).is_some()
}

/// Reviewer decisions accepted by the unified approval resolver.
/// 对齐 TS:
///   `Type.Union([Type.Literal("allow-once"), Type.Literal("allow-always"), Type.Literal("deny")])`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ApprovalDecision {
    #[serde(rename = "allow-once")]
    AllowOnce,
    #[serde(rename = "allow-always")]
    AllowAlways,
    #[serde(rename = "deny")]
    Deny,
}

impl ApprovalDecision {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::AllowOnce => "allow-once",
            Self::AllowAlways => "allow-always",
            Self::Deny => "deny",
        }
    }
}

/// Reviewer decisions that permit an operation to proceed.
/// 对齐 TS:
///   `Type.Union([Type.Literal("allow-once"), Type.Literal("allow-always")])`.
pub mod approval_allow_decisions {
    pub const ALLOW_ONCE: &str = "allow-once";
    pub const ALLOW_ALWAYS: &str = "allow-always";

    pub fn all() -> &'static [&'static str] {
        &[ALLOW_ONCE, ALLOW_ALWAYS]
    }

    pub fn from_str(s: &str) -> Option<&'static str> {
        match s {
            "allow-once" => Some(ALLOW_ONCE),
            "allow-always" => Some(ALLOW_ALWAYS),
            _ => None,
        }
    }
}

pub fn is_valid_approval_allow_decision(s: &str) -> bool {
    approval_allow_decisions::from_str(s).is_some()
}

/// Reviewer decisions that permit an operation to proceed.
/// 对齐 TS:
///   `Type.Union([Type.Literal("allow-once"), Type.Literal("allow-always")])`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ApprovalAllowDecision {
    #[serde(rename = "allow-once")]
    AllowOnce,
    #[serde(rename = "allow-always")]
    AllowAlways,
}

impl ApprovalAllowDecision {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::AllowOnce => "allow-once",
            Self::AllowAlways => "allow-always",
        }
    }
}

/// Closed reason recorded for a terminal approval transition.
/// 对齐 TS:
///   `Type.Union([
///      Type.Literal("user"),
///      Type.Literal("timeout"),
///      Type.Literal("malformed-verdict"),
///      Type.Literal("no-route"),
///      Type.Literal("run-aborted"),
///      Type.Literal("gateway-restart"),
///      Type.Literal("storage-corrupt"),
///   ])`.
pub mod approval_terminal_reasons {
    pub const USER: &str = "user";
    pub const TIMEOUT: &str = "timeout";
    pub const MALFORMED_VERDICT: &str = "malformed-verdict";
    pub const NO_ROUTE: &str = "no-route";
    pub const RUN_ABORTED: &str = "run-aborted";
    pub const GATEWAY_RESTART: &str = "gateway-restart";
    pub const STORAGE_CORRUPT: &str = "storage-corrupt";

    pub fn all() -> &'static [&'static str] {
        &[
            USER,
            TIMEOUT,
            MALFORMED_VERDICT,
            NO_ROUTE,
            RUN_ABORTED,
            GATEWAY_RESTART,
            STORAGE_CORRUPT,
        ]
    }

    pub fn from_str(s: &str) -> Option<&'static str> {
        match s {
            "user" => Some(USER),
            "timeout" => Some(TIMEOUT),
            "malformed-verdict" => Some(MALFORMED_VERDICT),
            "no-route" => Some(NO_ROUTE),
            "run-aborted" => Some(RUN_ABORTED),
            "gateway-restart" => Some(GATEWAY_RESTART),
            "storage-corrupt" => Some(STORAGE_CORRUPT),
            _ => None,
        }
    }
}

pub fn is_valid_approval_terminal_reason(s: &str) -> bool {
    approval_terminal_reasons::from_str(s).is_some()
}

/// Closed reason recorded for a terminal approval transition.
/// 对齐 TS: (see `approval_terminal_reasons` for the literal list).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ApprovalTerminalReason {
    #[serde(rename = "user")]
    User,
    #[serde(rename = "timeout")]
    Timeout,
    #[serde(rename = "malformed-verdict")]
    MalformedVerdict,
    #[serde(rename = "no-route")]
    NoRoute,
    #[serde(rename = "run-aborted")]
    RunAborted,
    #[serde(rename = "gateway-restart")]
    GatewayRestart,
    #[serde(rename = "storage-corrupt")]
    StorageCorrupt,
}

impl ApprovalTerminalReason {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::User => "user",
            Self::Timeout => "timeout",
            Self::MalformedVerdict => "malformed-verdict",
            Self::NoRoute => "no-route",
            Self::RunAborted => "run-aborted",
            Self::GatewayRestart => "gateway-restart",
            Self::StorageCorrupt => "storage-corrupt",
        }
    }
}

/// Terminal reason accepted for an allowed approval.
/// 对齐 TS: `Type.Union([Type.Literal("user")])`.
pub mod approval_allowed_reasons {
    pub const USER: &str = "user";

    pub fn all() -> &'static [&'static str] {
        &[USER]
    }

    pub fn from_str(s: &str) -> Option<&'static str> {
        match s {
            "user" => Some(USER),
            _ => None,
        }
    }
}

pub fn is_valid_approval_allowed_reason(s: &str) -> bool {
    approval_allowed_reasons::from_str(s).is_some()
}

/// Terminal reason accepted for an allowed approval.
/// 对齐 TS: `Type.Union([Type.Literal("user")])`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ApprovalAllowedReason {
    #[serde(rename = "user")]
    User,
}

impl ApprovalAllowedReason {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::User => "user",
        }
    }
}

impl Default for ApprovalAllowedReason {
    fn default() -> Self {
        Self::User
    }
}

/// Terminal reasons accepted for a denied approval.
/// 对齐 TS:
///   `Type.Union([
///      Type.Literal("user"),
///      Type.Literal("malformed-verdict"),
///      Type.Literal("no-route"),
///      Type.Literal("storage-corrupt"),
///   ])`.
pub mod approval_denied_reasons {
    pub const USER: &str = "user";
    pub const MALFORMED_VERDICT: &str = "malformed-verdict";
    pub const NO_ROUTE: &str = "no-route";
    pub const STORAGE_CORRUPT: &str = "storage-corrupt";

    pub fn all() -> &'static [&'static str] {
        &[USER, MALFORMED_VERDICT, NO_ROUTE, STORAGE_CORRUPT]
    }

    pub fn from_str(s: &str) -> Option<&'static str> {
        match s {
            "user" => Some(USER),
            "malformed-verdict" => Some(MALFORMED_VERDICT),
            "no-route" => Some(NO_ROUTE),
            "storage-corrupt" => Some(STORAGE_CORRUPT),
            _ => None,
        }
    }
}

pub fn is_valid_approval_denied_reason(s: &str) -> bool {
    approval_denied_reasons::from_str(s).is_some()
}

/// Terminal reasons accepted for a denied approval.
/// 对齐 TS: (see `approval_denied_reasons` for the literal list).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ApprovalDeniedReason {
    #[serde(rename = "user")]
    User,
    #[serde(rename = "malformed-verdict")]
    MalformedVerdict,
    #[serde(rename = "no-route")]
    NoRoute,
    #[serde(rename = "storage-corrupt")]
    StorageCorrupt,
}

impl ApprovalDeniedReason {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::User => "user",
            Self::MalformedVerdict => "malformed-verdict",
            Self::NoRoute => "no-route",
            Self::StorageCorrupt => "storage-corrupt",
        }
    }
}

/// Terminal reason accepted for an expired approval.
/// 对齐 TS: `Type.Union([Type.Literal("timeout")])`.
pub mod approval_expired_reasons {
    pub const TIMEOUT: &str = "timeout";

    pub fn all() -> &'static [&'static str] {
        &[TIMEOUT]
    }

    pub fn from_str(s: &str) -> Option<&'static str> {
        match s {
            "timeout" => Some(TIMEOUT),
            _ => None,
        }
    }
}

pub fn is_valid_approval_expired_reason(s: &str) -> bool {
    approval_expired_reasons::from_str(s).is_some()
}

/// Terminal reason accepted for an expired approval.
/// 对齐 TS: `Type.Union([Type.Literal("timeout")])`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ApprovalExpiredReason {
    #[serde(rename = "timeout")]
    Timeout,
}

impl ApprovalExpiredReason {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Timeout => "timeout",
        }
    }
}

impl Default for ApprovalExpiredReason {
    fn default() -> Self {
        Self::Timeout
    }
}

/// Terminal reasons accepted for a cancelled approval.
/// 对齐 TS:
///   `Type.Union([Type.Literal("run-aborted"), Type.Literal("gateway-restart")])`.
pub mod approval_cancelled_reasons {
    pub const RUN_ABORTED: &str = "run-aborted";
    pub const GATEWAY_RESTART: &str = "gateway-restart";

    pub fn all() -> &'static [&'static str] {
        &[RUN_ABORTED, GATEWAY_RESTART]
    }

    pub fn from_str(s: &str) -> Option<&'static str> {
        match s {
            "run-aborted" => Some(RUN_ABORTED),
            "gateway-restart" => Some(GATEWAY_RESTART),
            _ => None,
        }
    }
}

pub fn is_valid_approval_cancelled_reason(s: &str) -> bool {
    approval_cancelled_reasons::from_str(s).is_some()
}

/// Terminal reasons accepted for a cancelled approval.
/// 对齐 TS: (see `approval_cancelled_reasons` for the literal list).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ApprovalCancelledReason {
    #[serde(rename = "run-aborted")]
    RunAborted,
    #[serde(rename = "gateway-restart")]
    GatewayRestart,
}

impl ApprovalCancelledReason {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::RunAborted => "run-aborted",
            Self::GatewayRestart => "gateway-restart",
        }
    }
}

/// Reviewer-facing severity for plugin-owned approval requests.
/// 对齐 TS:
///   `Type.Union([
///      Type.Literal("info"), Type.Literal("warning"), Type.Literal("critical"),
///   ])`.
/// Re-exported from `plugin_approvals` to keep a single source of truth.
pub use super::plugin_approvals::PluginApprovalSeverity as PluginApprovalSeveritySchema;

pub fn is_valid_plugin_approval_severity(s: &str) -> bool {
    super::plugin_approvals::is_valid_plugin_approval_severity(s)
}

// ---------- Allowed-decisions array ----------

/// Validated list of allowed reviewer decisions.
/// 对齐 TS:
///   `Type.Array(ApprovalDecisionSchema, {
///      minItems: 1, maxItems: 3, uniqueItems: true,
///      contains: Type.Literal("deny"),
///      description: "Available reviewer decisions. Deny is always available ..."
///   })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalAllowedDecisionsSchema(pub Vec<ApprovalDecision>);

impl ApprovalAllowedDecisionsSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_approval_allowed_decisions(&self.0)
    }
}

impl AsRef<Vec<ApprovalDecision>> for ApprovalAllowedDecisionsSchema {
    fn as_ref(&self) -> &Vec<ApprovalDecision> {
        &self.0
    }
}

fn validate_approval_allowed_decisions(values: &[ApprovalDecision]) -> Result<(), String> {
    if values.len() < APPROVAL_ALLOWED_DECISIONS_MIN_ITEMS
        || values.len() > APPROVAL_ALLOWED_DECISIONS_MAX_ITEMS
    {
        return Err(format!(
            "allowedDecisions: expected length [{}, {}], got {}",
            APPROVAL_ALLOWED_DECISIONS_MIN_ITEMS,
            APPROVAL_ALLOWED_DECISIONS_MAX_ITEMS,
            values.len()
        ));
    }
    // uniqueItems
    for i in 0..values.len() {
        for j in (i + 1)..values.len() {
            if values[i] == values[j] {
                return Err(format!(
                    "allowedDecisions: expected unique items, found duplicate {:?}",
                    values[i].as_str()
                ));
            }
        }
    }
    // contains: Type.Literal("deny")
    if !values.iter().any(|v| matches!(v, ApprovalDecision::Deny)) {
        return Err(format!(
            "allowedDecisions: must contain \"deny\", got {:?}",
            values.iter().map(|v| v.as_str()).collect::<Vec<_>>()
        ));
    }
    Ok(())
}

// ---------- 基础验证原语 ----------

fn validate_optional_non_empty_string(field: &str, value: Option<&str>) -> Result<(), String> {
    if let Some(s) = value {
        if !is_non_empty_string(s) {
            return Err(format!(
                "{}: expected non-empty string, got {:?}",
                field, s
            ));
        }
    }
    Ok(())
}

fn validate_non_negative_integer(field: &str, n: i64) -> Result<(), String> {
    if n < 0 {
        return Err(format!("{}: expected non-negative integer, got {}", field, n));
    }
    Ok(())
}

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

// ---------- ExecApprovalPresentationSchema ----------

/// Redacted exec details safe to persist and render outside the requesting runtime.
/// 对齐 TS:
///   `Type.Object({
///      kind: Type.Literal("exec"),
///      commandText: NonEmptyString,
///      commandPreview: Type.Optional(Type.Union([Type.String(), Type.Null()])),
///      warningText:  Type.Optional(Type.Union([Type.String(), Type.Null()])),
///      host:         Type.Optional(Type.Union([Type.String(), Type.Null()])),
///      nodeId:       Type.Optional(Type.Union([NonEmptyString, Type.Null()])),
///      agentId:      Type.Optional(Type.Union([NonEmptyString, Type.Null()])),
///      allowedDecisions: ApprovalAllowedDecisionsSchema,
///   }, {
///      additionalProperties: false,
///      description: "Reviewer-safe exec presentation. Runtime cwd, environment,
///                    system-run binding, and execution plan are intentionally excluded."
///   })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecApprovalPresentationSchema {
    pub kind: ApprovalKind,
    pub command_text: NonEmptyString,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub command_preview: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub warning_text: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub host: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub node_id: Option<NonEmptyString>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub agent_id: Option<NonEmptyString>,
    pub allowed_decisions: ApprovalAllowedDecisionsSchema,
}

impl ExecApprovalPresentationSchema {
    pub fn validate(&self) -> Result<(), String> {
        if !matches!(self.kind, ApprovalKind::Exec) {
            return Err(format!(
                "kind: expected literal \"exec\", got {:?}",
                self.kind.as_str()
            ));
        }
        if !is_non_empty_string(&self.command_text) {
            return Err(format!(
                "commandText: expected non-empty string, got {:?}",
                self.command_text
            ));
        }
        validate_optional_non_empty_string("nodeId", self.node_id.as_deref())?;
        validate_optional_non_empty_string("agentId", self.agent_id.as_deref())?;
        self.allowed_decisions
            .validate()
            .map_err(|e| format!("allowedDecisions: {}", e))?;
        Ok(())
    }
}

// ---------- PluginApprovalPresentationSchema ----------

/// Plugin-supplied reviewer text safe to persist and render across surfaces.
/// 对齐 TS:
///   `Type.Object({
///      kind: Type.Literal("plugin"),
///      title: Type.String({ minLength: 1, maxLength: 80 }),
///      description: Type.String({ minLength: 1, maxLength: 512 }),
///      severity: PluginApprovalSeveritySchema,
///      pluginId: Type.Optional(Type.Union([NonEmptyString, Type.Null()])),
///      toolName: Type.Optional(Type.Union([NonEmptyString, Type.Null()])),
///      agentId:  Type.Optional(Type.Union([NonEmptyString, Type.Null()])),
///      allowedDecisions: ApprovalAllowedDecisionsSchema,
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginApprovalPresentationSchema {
    pub kind: ApprovalKind,
    pub title: String,
    pub description: String,
    pub severity: PluginApprovalSeveritySchema,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub plugin_id: Option<NonEmptyString>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool_name: Option<NonEmptyString>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub agent_id: Option<NonEmptyString>,
    pub allowed_decisions: ApprovalAllowedDecisionsSchema,
}

impl PluginApprovalPresentationSchema {
    pub fn validate(&self) -> Result<(), String> {
        if !matches!(self.kind, ApprovalKind::Plugin) {
            return Err(format!(
                "kind: expected literal \"plugin\", got {:?}",
                self.kind.as_str()
            ));
        }
        validate_plugin_approval_title("title", &self.title)?;
        validate_plugin_approval_description("description", &self.description)?;
        if !is_valid_plugin_approval_severity(self.severity.as_str()) {
            return Err(format!(
                "severity: expected one of {:?}, got {:?}",
                super::plugin_approvals::plugin_approval_severities::all(),
                self.severity.as_str()
            ));
        }
        validate_optional_non_empty_string("pluginId", self.plugin_id.as_deref())?;
        validate_optional_non_empty_string("toolName", self.tool_name.as_deref())?;
        validate_optional_non_empty_string("agentId", self.agent_id.as_deref())?;
        self.allowed_decisions
            .validate()
            .map_err(|e| format!("allowedDecisions: {}", e))?;
        Ok(())
    }
}

// ---------- ApprovalPresentationSchema ----------

/// Reviewer-safe presentation discriminated by the approval owner.
/// 对齐 TS:
///   `Type.Union([ExecApprovalPresentationSchema, PluginApprovalPresentationSchema])`.
/// Untagged at the JSON-Schema level; each variant carries its own `kind` literal.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ApprovalPresentationSchema {
    Exec(ExecApprovalPresentationSchema),
    Plugin(PluginApprovalPresentationSchema),
}

impl ApprovalPresentationSchema {
    pub fn validate(&self) -> Result<(), String> {
        match self {
            Self::Exec(e) => e.validate(),
            Self::Plugin(p) => p.validate(),
        }
    }
}

// ---------- Approval snapshot unions ----------

/// Literal `"pending"` status for `PendingApprovalSnapshotSchema`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PendingApprovalSnapshotStatus {
    #[serde(rename = "pending")]
    Pending,
}

impl Default for PendingApprovalSnapshotStatus {
    fn default() -> Self {
        Self::Pending
    }
}

/// Literal `"allowed"` status for `AllowedApprovalSnapshotSchema`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AllowedApprovalSnapshotStatus {
    #[serde(rename = "allowed")]
    Allowed,
}

impl Default for AllowedApprovalSnapshotStatus {
    fn default() -> Self {
        Self::Allowed
    }
}

/// Literal `"deny"` decision carried on `DeniedApprovalSnapshotSchema`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DeniedApprovalSnapshotDecision {
    #[serde(rename = "deny")]
    Deny,
}

impl Default for DeniedApprovalSnapshotDecision {
    fn default() -> Self {
        Self::Deny
    }
}

/// Literal `"denied"` status for `DeniedApprovalSnapshotSchema`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DeniedApprovalSnapshotStatus {
    #[serde(rename = "denied")]
    Denied,
}

impl Default for DeniedApprovalSnapshotStatus {
    fn default() -> Self {
        Self::Denied
    }
}

/// Literal `"expired"` status for `ExpiredApprovalSnapshotSchema`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ExpiredApprovalSnapshotStatus {
    #[serde(rename = "expired")]
    Expired,
}

impl Default for ExpiredApprovalSnapshotStatus {
    fn default() -> Self {
        Self::Expired
    }
}

/// Literal `"cancelled"` status for `CancelledApprovalSnapshotSchema`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CancelledApprovalSnapshotStatus {
    #[serde(rename = "cancelled")]
    Cancelled,
}

impl Default for CancelledApprovalSnapshotStatus {
    fn default() -> Self {
        Self::Cancelled
    }
}

// ---------- PendingApprovalSnapshotSchema ----------

/// Approval that has not yet accepted a reviewer decision.
/// 对齐 TS:
///   `Type.Object({
///      ...ApprovalRecordCommonFields,
///      status: Type.Literal("pending"),
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PendingApprovalSnapshotSchema {
    pub id: ApprovalIdSchema,
    pub url_path: NonEmptyString,
    pub created_at_ms: i64,
    pub expires_at_ms: i64,
    pub presentation: ApprovalPresentationSchema,
    pub status: PendingApprovalSnapshotStatus,
}

impl PendingApprovalSnapshotSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_approval_id("id", &self.id)?;
        if !is_non_empty_string(&self.url_path) {
            return Err(format!(
                "urlPath: expected non-empty string, got {:?}",
                self.url_path
            ));
        }
        validate_non_negative_integer("createdAtMs", self.created_at_ms)?;
        validate_non_negative_integer("expiresAtMs", self.expires_at_ms)?;
        self.presentation
            .validate()
            .map_err(|e| format!("presentation: {}", e))?;
        Ok(())
    }
}

// ---------- AllowedApprovalSnapshotSchema ----------

/// Approval whose first recorded reviewer decision allows the operation.
/// 对齐 TS:
///   `Type.Object({
///      ...ApprovalRecordCommonFields,
///      ...ApprovalResolutionFields,
///      status: Type.Literal("allowed"),
///      decision: ApprovalAllowDecisionSchema,
///      reason:   ApprovalAllowedReasonSchema,
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AllowedApprovalSnapshotSchema {
    pub id: ApprovalIdSchema,
    pub url_path: NonEmptyString,
    pub created_at_ms: i64,
    pub expires_at_ms: i64,
    pub presentation: ApprovalPresentationSchema,
    pub resolved_at_ms: i64,
    pub status: AllowedApprovalSnapshotStatus,
    pub decision: ApprovalAllowDecision,
    pub reason: ApprovalAllowedReason,
}

impl AllowedApprovalSnapshotSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_approval_id("id", &self.id)?;
        if !is_non_empty_string(&self.url_path) {
            return Err(format!(
                "urlPath: expected non-empty string, got {:?}",
                self.url_path
            ));
        }
        validate_non_negative_integer("createdAtMs", self.created_at_ms)?;
        validate_non_negative_integer("expiresAtMs", self.expires_at_ms)?;
        validate_non_negative_integer("resolvedAtMs", self.resolved_at_ms)?;
        self.presentation
            .validate()
            .map_err(|e| format!("presentation: {}", e))?;
        Ok(())
    }
}

// ---------- DeniedApprovalSnapshotSchema ----------

/// Approval whose first recorded reviewer decision denies the operation.
/// 对齐 TS:
///   `Type.Object({
///      ...ApprovalRecordCommonFields,
///      ...ApprovalResolutionFields,
///      status: Type.Literal("denied"),
///      decision: Type.Literal("deny"),
///      reason:   ApprovalDeniedReasonSchema,
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeniedApprovalSnapshotSchema {
    pub id: ApprovalIdSchema,
    pub url_path: NonEmptyString,
    pub created_at_ms: i64,
    pub expires_at_ms: i64,
    pub presentation: ApprovalPresentationSchema,
    pub resolved_at_ms: i64,
    pub status: DeniedApprovalSnapshotStatus,
    pub decision: DeniedApprovalSnapshotDecision,
    pub reason: ApprovalDeniedReason,
}

impl DeniedApprovalSnapshotSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_approval_id("id", &self.id)?;
        if !is_non_empty_string(&self.url_path) {
            return Err(format!(
                "urlPath: expected non-empty string, got {:?}",
                self.url_path
            ));
        }
        validate_non_negative_integer("createdAtMs", self.created_at_ms)?;
        validate_non_negative_integer("expiresAtMs", self.expires_at_ms)?;
        validate_non_negative_integer("resolvedAtMs", self.resolved_at_ms)?;
        self.presentation
            .validate()
            .map_err(|e| format!("presentation: {}", e))?;
        Ok(())
    }
}

// ---------- ExpiredApprovalSnapshotSchema ----------

/// Approval that reached its deadline and therefore failed closed.
/// 对齐 TS:
///   `Type.Object({
///      ...ApprovalRecordCommonFields,
///      ...ApprovalResolutionFields,
///      status: Type.Literal("expired"),
///      reason: ApprovalExpiredReasonSchema,
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExpiredApprovalSnapshotSchema {
    pub id: ApprovalIdSchema,
    pub url_path: NonEmptyString,
    pub created_at_ms: i64,
    pub expires_at_ms: i64,
    pub presentation: ApprovalPresentationSchema,
    pub resolved_at_ms: i64,
    pub status: ExpiredApprovalSnapshotStatus,
    pub reason: ApprovalExpiredReason,
}

impl ExpiredApprovalSnapshotSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_approval_id("id", &self.id)?;
        if !is_non_empty_string(&self.url_path) {
            return Err(format!(
                "urlPath: expected non-empty string, got {:?}",
                self.url_path
            ));
        }
        validate_non_negative_integer("createdAtMs", self.created_at_ms)?;
        validate_non_negative_integer("expiresAtMs", self.expires_at_ms)?;
        validate_non_negative_integer("resolvedAtMs", self.resolved_at_ms)?;
        self.presentation
            .validate()
            .map_err(|e| format!("presentation: {}", e))?;
        Ok(())
    }
}

// ---------- CancelledApprovalSnapshotSchema ----------

/// Approval cancelled by its runtime owner before a reviewer decision.
/// 对齐 TS:
///   `Type.Object({
///      ...ApprovalRecordCommonFields,
///      ...ApprovalResolutionFields,
///      status: Type.Literal("cancelled"),
///      reason: ApprovalCancelledReasonSchema,
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CancelledApprovalSnapshotSchema {
    pub id: ApprovalIdSchema,
    pub url_path: NonEmptyString,
    pub created_at_ms: i64,
    pub expires_at_ms: i64,
    pub presentation: ApprovalPresentationSchema,
    pub resolved_at_ms: i64,
    pub status: CancelledApprovalSnapshotStatus,
    pub reason: ApprovalCancelledReason,
}

impl CancelledApprovalSnapshotSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_approval_id("id", &self.id)?;
        if !is_non_empty_string(&self.url_path) {
            return Err(format!(
                "urlPath: expected non-empty string, got {:?}",
                self.url_path
            ));
        }
        validate_non_negative_integer("createdAtMs", self.created_at_ms)?;
        validate_non_negative_integer("expiresAtMs", self.expires_at_ms)?;
        validate_non_negative_integer("resolvedAtMs", self.resolved_at_ms)?;
        self.presentation
            .validate()
            .map_err(|e| format!("presentation: {}", e))?;
        Ok(())
    }
}

// ---------- ApprovalSnapshotSchema / TerminalApprovalSnapshotSchema ----------

/// Durable approval projection returned identically to every authorized surface.
/// 对齐 TS:
///   `Type.Union([
///      PendingApprovalSnapshotSchema,
///      AllowedApprovalSnapshotSchema,
///      DeniedApprovalSnapshotSchema,
///      ExpiredApprovalSnapshotSchema,
///      CancelledApprovalSnapshotSchema,
///   ])`.
/// Untagged at the JSON-Schema level; each variant carries its own `status` literal.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ApprovalSnapshotSchema {
    Pending(PendingApprovalSnapshotSchema),
    Allowed(AllowedApprovalSnapshotSchema),
    Denied(DeniedApprovalSnapshotSchema),
    Expired(ExpiredApprovalSnapshotSchema),
    Cancelled(CancelledApprovalSnapshotSchema),
}

impl ApprovalSnapshotSchema {
    pub fn validate(&self) -> Result<(), String> {
        match self {
            Self::Pending(s) => s.validate(),
            Self::Allowed(s) => s.validate(),
            Self::Denied(s) => s.validate(),
            Self::Expired(s) => s.validate(),
            Self::Cancelled(s) => s.validate(),
        }
    }
}

/// Durable terminal approval state returned after a resolution attempt.
/// 对齐 TS:
///   `Type.Union([
///      AllowedApprovalSnapshotSchema,
///      DeniedApprovalSnapshotSchema,
///      ExpiredApprovalSnapshotSchema,
///      CancelledApprovalSnapshotSchema,
///   ])`.
/// Untagged at the JSON-Schema level; each variant carries its own `status` literal.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum TerminalApprovalSnapshotSchema {
    Allowed(AllowedApprovalSnapshotSchema),
    Denied(DeniedApprovalSnapshotSchema),
    Expired(ExpiredApprovalSnapshotSchema),
    Cancelled(CancelledApprovalSnapshotSchema),
}

impl TerminalApprovalSnapshotSchema {
    pub fn validate(&self) -> Result<(), String> {
        match self {
            Self::Allowed(s) => s.validate(),
            Self::Denied(s) => s.validate(),
            Self::Expired(s) => s.validate(),
            Self::Cancelled(s) => s.validate(),
        }
    }
}

// ---------- ApprovalGetParamsSchema ----------

/// Lookup payload for one approval by its exact full id.
/// 对齐 TS:
///   `Type.Object({ id: ApprovalRecordCommonFields.id }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApprovalGetParamsSchema {
    pub id: ApprovalIdSchema,
}

impl ApprovalGetParamsSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_approval_id("id", &self.id)?;
        Ok(())
    }
}

// ---------- ApprovalGetResultSchema ----------

/// Current durable state for one authorized approval lookup.
/// 对齐 TS:
///   `Type.Object({ approval: ApprovalSnapshotSchema }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApprovalGetResultSchema {
    pub approval: ApprovalSnapshotSchema,
}

impl ApprovalGetResultSchema {
    pub fn validate(&self) -> Result<(), String> {
        self.approval
            .validate()
            .map_err(|e| format!("approval: {}", e))?;
        Ok(())
    }
}

// ---------- ApprovalResolveParamsSchema ----------

/// Reviewer decision for one approval identified by its exact full id.
/// 对齐 TS:
///   `Type.Object({
///      id:       ApprovalRecordCommonFields.id,
///      kind:     ApprovalKindSchema,
///      decision: ApprovalDecisionSchema,
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApprovalResolveParamsSchema {
    pub id: ApprovalIdSchema,
    pub kind: ApprovalKind,
    pub decision: ApprovalDecision,
}

impl ApprovalResolveParamsSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_approval_id("id", &self.id)?;
        if !is_valid_approval_kind(self.kind.as_str()) {
            return Err(format!(
                "kind: expected one of {:?}, got {:?}",
                approval_kinds::all(),
                self.kind.as_str()
            ));
        }
        if !is_valid_approval_decision(self.decision.as_str()) {
            return Err(format!(
                "decision: expected one of {:?}, got {:?}",
                approval_decisions::all(),
                self.decision.as_str()
            ));
        }
        Ok(())
    }
}

// ---------- ApprovalResolveResultSchema ----------

/// First-answer outcome plus the canonical recorded state returned to all contenders.
/// 对齐 TS:
///   `Type.Object({
///      applied: Type.Boolean(),
///      approval: TerminalApprovalSnapshotSchema,
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApprovalResolveResultSchema {
    pub applied: bool,
    pub approval: TerminalApprovalSnapshotSchema,
}

impl ApprovalResolveResultSchema {
    pub fn validate(&self) -> Result<(), String> {
        self.approval
            .validate()
            .map_err(|e| format!("approval: {}", e))?;
        Ok(())
    }
}

// ---------- SessionApprovalEvent literal status enums ----------

/// Literal `"pending"` phase carried on `PendingSessionApprovalEventSchema`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PendingSessionApprovalEventPhase {
    #[serde(rename = "pending")]
    Pending,
}

impl Default for PendingSessionApprovalEventPhase {
    fn default() -> Self {
        Self::Pending
    }
}

/// Literal `"terminal"` phase carried on `TerminalSessionApprovalEventSchema`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TerminalSessionApprovalEventPhase {
    #[serde(rename = "terminal")]
    Terminal,
}

impl Default for TerminalSessionApprovalEventPhase {
    fn default() -> Self {
        Self::Terminal
    }
}

// ---------- PendingSessionApprovalEventSchema ----------

/// Sanitized pending transition delivered only to an opted-in session audience.
/// 对齐 TS:
///   `Type.Object({
///      ...SessionApprovalEventCommonFields,
///      phase: Type.Literal("pending"),
///      approval: PendingApprovalSnapshotSchema,
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PendingSessionApprovalEventSchema {
    pub session_key: NonEmptyString,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_session_key: Option<NonEmptyString>,
    pub updated_at_ms: i64,
    pub phase: PendingSessionApprovalEventPhase,
    pub approval: PendingApprovalSnapshotSchema,
}

impl PendingSessionApprovalEventSchema {
    pub fn validate(&self) -> Result<(), String> {
        if !is_non_empty_string(&self.session_key) {
            return Err(format!(
                "sessionKey: expected non-empty string, got {:?}",
                self.session_key
            ));
        }
        validate_optional_non_empty_string(
            "sourceSessionKey",
            self.source_session_key.as_deref(),
        )?;
        validate_non_negative_integer("updatedAtMs", self.updated_at_ms)?;
        self.approval
            .validate()
            .map_err(|e| format!("approval: {}", e))?;
        Ok(())
    }
}

// ---------- TerminalSessionApprovalEventSchema ----------

/// Sanitized terminal transition delivered only to an opted-in session audience.
/// 对齐 TS:
///   `Type.Object({
///      ...SessionApprovalEventCommonFields,
///      phase: Type.Literal("terminal"),
///      approval: TerminalApprovalSnapshotSchema,
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TerminalSessionApprovalEventSchema {
    pub session_key: NonEmptyString,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_session_key: Option<NonEmptyString>,
    pub updated_at_ms: i64,
    pub phase: TerminalSessionApprovalEventPhase,
    pub approval: TerminalApprovalSnapshotSchema,
}

impl TerminalSessionApprovalEventSchema {
    pub fn validate(&self) -> Result<(), String> {
        if !is_non_empty_string(&self.session_key) {
            return Err(format!(
                "sessionKey: expected non-empty string, got {:?}",
                self.session_key
            ));
        }
        validate_optional_non_empty_string(
            "sourceSessionKey",
            self.source_session_key.as_deref(),
        )?;
        validate_non_negative_integer("updatedAtMs", self.updated_at_ms)?;
        self.approval
            .validate()
            .map_err(|e| format!("approval: {}", e))?;
        Ok(())
    }
}

// ---------- SessionApprovalEventSchema ----------

/// Sanitized approval transition delivered only to an opted-in session audience.
/// 对齐 TS:
///   `Type.Union([
///      PendingSessionApprovalEventSchema,
///      TerminalSessionApprovalEventSchema,
///   ])`.
/// Untagged at the JSON-Schema level; each variant carries its own `phase` literal.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum SessionApprovalEventSchema {
    Pending(PendingSessionApprovalEventSchema),
    Terminal(TerminalSessionApprovalEventSchema),
}

impl SessionApprovalEventSchema {
    pub fn validate(&self) -> Result<(), String> {
        match self {
            Self::Pending(e) => e.validate(),
            Self::Terminal(e) => e.validate(),
        }
    }
}

// ---------- SessionApprovalReplaySchema ----------

/// Authoritative pending approval set returned when a session stream subscribes.
/// 对齐 TS:
///   `Type.Object({
///      sessionKey: NonEmptyString,
///      updatedAtMs: Type.Integer({ minimum: 0 }),
///      approvals: Type.Array(PendingApprovalSnapshotSchema),
///      truncated: Type.Boolean(),
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionApprovalReplaySchema {
    pub session_key: NonEmptyString,
    pub updated_at_ms: i64,
    pub approvals: Vec<PendingApprovalSnapshotSchema>,
    pub truncated: bool,
}

impl SessionApprovalReplaySchema {
    pub fn validate(&self) -> Result<(), String> {
        if !is_non_empty_string(&self.session_key) {
            return Err(format!(
                "sessionKey: expected non-empty string, got {:?}",
                self.session_key
            ));
        }
        validate_non_negative_integer("updatedAtMs", self.updated_at_ms)?;
        for (i, approval) in self.approvals.iter().enumerate() {
            approval
                .validate()
                .map_err(|e| format!("approvals[{}]: {}", i, e))?;
        }
        Ok(())
    }
}

// Owner-local wire types derived directly from local schema consts so the
// public plugin-sdk declaration graph never pulls in the ProtocolSchemas registry.
// 对应 TS:
//   export type ApprovalKind = Static<typeof ApprovalKindSchema>;
//   export type ApprovalDecision = Static<typeof ApprovalDecisionSchema>;
//   export type ApprovalAllowDecision = Static<typeof ApprovalAllowDecisionSchema>;
//   export type ApprovalTerminalReason = Static<typeof ApprovalTerminalReasonSchema>;
//   export type PluginApprovalSeverity = Static<typeof PluginApprovalSeveritySchema>;
//   export type ExecApprovalPresentation = Static<typeof ExecApprovalPresentationSchema>;
//   export type PluginApprovalPresentation = Static<typeof PluginApprovalPresentationSchema>;
//   export type ApprovalPresentation = Static<typeof ApprovalPresentationSchema>;
//   export type PendingApprovalSnapshot = Static<typeof PendingApprovalSnapshotSchema>;
//   export type ApprovalSnapshot = Static<typeof ApprovalSnapshotSchema>;
//   export type ApprovalGetParams = Static<typeof ApprovalGetParamsSchema>;
//   export type ApprovalGetResult = Static<typeof ApprovalGetResultSchema>;
//   export type ApprovalResolveParams = Static<typeof ApprovalResolveParamsSchema>;
//   export type ApprovalResolveResult = Static<typeof ApprovalResolveResultSchema>;
//   export type AllowedApprovalSnapshot = Static<typeof AllowedApprovalSnapshotSchema>;
//   export type DeniedApprovalSnapshot = Static<typeof DeniedApprovalSnapshotSchema>;
//   export type ExpiredApprovalSnapshot = Static<typeof ExpiredApprovalSnapshotSchema>;
//   export type CancelledApprovalSnapshot = Static<typeof CancelledApprovalSnapshotSchema>;
//   export type TerminalApprovalSnapshot = Static<typeof TerminalApprovalSnapshotSchema>;
//   export type SessionApprovalEvent = Static<typeof SessionApprovalEventSchema>;
//   export type SessionApprovalReplay = Static<typeof SessionApprovalReplaySchema>;
pub type ApprovalKindType = ApprovalKind;
pub type ApprovalDecisionType = ApprovalDecision;
pub type ApprovalAllowDecisionType = ApprovalAllowDecision;
pub type ApprovalTerminalReasonType = ApprovalTerminalReason;
pub type PluginApprovalSeverityType = PluginApprovalSeverity;
pub type ExecApprovalPresentationType = ExecApprovalPresentationSchema;
pub type PluginApprovalPresentationType = PluginApprovalPresentationSchema;
pub type ApprovalPresentationType = ApprovalPresentationSchema;
pub type PendingApprovalSnapshotType = PendingApprovalSnapshotSchema;
pub type ApprovalSnapshotType = ApprovalSnapshotSchema;
pub type ApprovalGetParamsType = ApprovalGetParamsSchema;
pub type ApprovalGetResultType = ApprovalGetResultSchema;
pub type ApprovalResolveParamsType = ApprovalResolveParamsSchema;
pub type ApprovalResolveResultType = ApprovalResolveResultSchema;
pub type AllowedApprovalSnapshotType = AllowedApprovalSnapshotSchema;
pub type DeniedApprovalSnapshotType = DeniedApprovalSnapshotSchema;
pub type ExpiredApprovalSnapshotType = ExpiredApprovalSnapshotSchema;
pub type CancelledApprovalSnapshotType = CancelledApprovalSnapshotSchema;
pub type TerminalApprovalSnapshotType = TerminalApprovalSnapshotSchema;
pub type SessionApprovalEventType = SessionApprovalEventSchema;
pub type SessionApprovalReplayType = SessionApprovalReplaySchema;