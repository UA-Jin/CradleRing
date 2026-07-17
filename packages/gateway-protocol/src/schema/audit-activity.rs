// Gateway Protocol schema: audit activity.
// 翻译自 packages/gateway-protocol/src/schema/audit-activity.ts
//
// Versioned metadata-only activity audit query payloads.
//
// TS 用 TypeBox 定义 schema（运行时验证 + 类型）。
// Rust 用 serde struct + 验证函数实现等价的序列化/反序列化语义。
//
// Wire types derive directly from local schema consts so public d.ts graphs
// never pull in the ProtocolSchemas registry. 见文件末尾的 type aliases.

use serde::{Deserialize, Serialize};

// ============================================================================
// 基础验证原语 (对齐 TypeBox: NonEmptyString / Integer{min} / String{pattern})
// ============================================================================

/// 对齐 TS: `NonEmptyString = Type.String({ minLength: 1 })`.
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

/// 对齐 TS: `Type.Integer({ minimum: 0 })`.
fn validate_non_negative_integer(field: &str, n: i64) -> Result<(), String> {
    if n >= 0 {
        Ok(())
    } else {
        Err(format!(
            "{}: expected integer >= 0, got {}",
            field, n
        ))
    }
}

fn validate_optional_non_negative_integer(field: &str, n: Option<i64>) -> Result<(), String> {
    if let Some(v) = n {
        validate_non_negative_integer(field, v)?;
    }
    Ok(())
}

/// 对齐 TS: `Type.Integer({ minimum: 1 })`.
fn validate_positive_integer(field: &str, n: i64) -> Result<(), String> {
    if n >= 1 {
        Ok(())
    } else {
        Err(format!(
            "{}: expected integer >= 1, got {}",
            field, n
        ))
    }
}

fn validate_integer_in_range(field: &str, n: i64, min: i64, max: i64) -> Result<(), String> {
    if (min..=max).contains(&n) {
        Ok(())
    } else {
        Err(format!(
            "{}: expected {}..={}, got {}",
            field, min, max, n
        ))
    }
}

fn regex(pattern: &str) -> regex::Regex {
    regex::Regex::new(pattern).expect("invalid regex pattern compiled into audit_activity")
}

fn validate_pattern(field: &str, value: &str, pattern: &str) -> Result<(), String> {
    if regex(pattern).is_match(value) {
        Ok(())
    } else {
        Err(format!(
            "{}: expected string matching {}, got {:?}",
            field, pattern, value
        ))
    }
}

fn validate_optional_pattern(
    field: &str,
    value: Option<&str>,
    pattern: &str,
) -> Result<(), String> {
    if let Some(s) = value {
        validate_pattern(field, s, pattern)?;
    }
    Ok(())
}

// ============================================================================
// 常量
// ============================================================================

/// 对齐 TS: `AuditActivitySchemaVersionV1Schema = Type.Integer({ minimum: 1, maximum: 1 })`.
pub const AUDIT_ACTIVITY_SCHEMA_VERSION_V1: i64 = 1;

/// Redaction policy literal. Currently fixed to `metadata_only`.
/// 对齐 TS: `Type.Literal("metadata_only")`.
pub const AUDIT_ACTIVITY_REDACTION_METADATA_ONLY: &str = "metadata_only";

/// HMAC reference id pattern.
/// 对齐 TS: `AuditActivityHmacRefV1Schema = Type.String({ pattern: "^hmac-sha256:v1:[a-f0-9]{32}:[a-f0-9]{64}$" })`.
const AUDIT_ACTIVITY_HMAC_REF_PATTERN: &str = r"^hmac-sha256:v1:[a-f0-9]{32}:[a-f0-9]{64}$";

/// Returns true when `id` matches the HMAC ref id grammar.
pub fn is_valid_audit_activity_hmac_ref(id: &str) -> bool {
    regex(AUDIT_ACTIVITY_HMAC_REF_PATTERN).is_match(id)
}

/// 对齐 TS: `Type.Integer({ minimum: 1, maximum: 500 })` (list `limit` 上限).
pub const AUDIT_ACTIVITY_LIST_LIMIT_MAX: i64 = 500;

// ============================================================================
// Enums
// ============================================================================

// ---------- AuditActivityStatusV1Schema ----------

/// Activity terminal/intermediate status (closed across all activity kinds).
/// 对齐 TS:
///   `Type.Union([
///      Type.Literal("started"), Type.Literal("succeeded"),
///      Type.Literal("failed"), Type.Literal("cancelled"),
///      Type.Literal("timed_out"), Type.Literal("blocked"),
///      Type.Literal("unknown"),
///   ])`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuditActivityStatusV1Schema {
    Started,
    Succeeded,
    Failed,
    Cancelled,
    TimedOut,
    Blocked,
    Unknown,
}

impl AuditActivityStatusV1Schema {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Started => "started",
            Self::Succeeded => "succeeded",
            Self::Failed => "failed",
            Self::Cancelled => "cancelled",
            Self::TimedOut => "timed_out",
            Self::Blocked => "blocked",
            Self::Unknown => "unknown",
        }
    }
}

pub fn is_valid_audit_activity_status(s: &str) -> bool {
    matches!(
        s,
        "started" | "succeeded" | "failed" | "cancelled" | "timed_out" | "blocked" | "unknown"
    )
}

// ---------- AuditActivityKindV1Schema ----------

/// Activity kind discriminator (closed across all activity kinds).
/// 对齐 TS:
///   `Type.Union([
///      Type.Literal("agent_run"), Type.Literal("tool_action"),
///      Type.Literal("message"),
///   ])`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuditActivityKindV1Schema {
    AgentRun,
    ToolAction,
    Message,
}

impl AuditActivityKindV1Schema {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::AgentRun => "agent_run",
            Self::ToolAction => "tool_action",
            Self::Message => "message",
        }
    }
}

pub fn is_valid_audit_activity_kind(s: &str) -> bool {
    matches!(s, "agent_run" | "tool_action" | "message")
}

// ---------- AuditActivityDirectionV1Schema ----------

/// Activity direction discriminator.
/// 对齐 TS: `Type.Union([Type.Literal("inbound"), Type.Literal("outbound")])`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuditActivityDirectionV1Schema {
    Inbound,
    Outbound,
}

impl AuditActivityDirectionV1Schema {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Inbound => "inbound",
            Self::Outbound => "outbound",
        }
    }
}

pub fn is_valid_audit_activity_direction(s: &str) -> bool {
    matches!(s, "inbound" | "outbound")
}

// ---------- AuditActivityEventTypeV1Schema ----------

/// `eventType` literal discriminator for `AuditActivityEventV1Schema`.
/// 对齐 TS: 每个 record 的 `eventType` 字面量.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuditActivityEventTypeV1Schema {
    AgentRun,
    ToolAction,
    InboundMessage,
    OutboundMessage,
}

impl AuditActivityEventTypeV1Schema {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::AgentRun => "agent_run",
            Self::ToolAction => "tool_action",
            Self::InboundMessage => "inbound_message",
            Self::OutboundMessage => "outbound_message",
        }
    }
}

pub fn is_valid_audit_activity_event_type(s: &str) -> bool {
    matches!(
        s,
        "agent_run" | "tool_action" | "inbound_message" | "outbound_message"
    )
}

// ---------- AuditActivityConversationKindV1Schema ----------

/// 对齐 TS:
///   `AuditActivityConversationKindV1Schema = Type.Union([
///      Type.Literal("direct"), Type.Literal("group"),
///      Type.Literal("channel"), Type.Literal("unknown"),
///   ])`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuditActivityConversationKindV1Schema {
    Direct,
    Group,
    Channel,
    Unknown,
}

impl AuditActivityConversationKindV1Schema {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Direct => "direct",
            Self::Group => "group",
            Self::Channel => "channel",
            Self::Unknown => "unknown",
        }
    }
}

pub fn is_valid_audit_activity_conversation_kind(s: &str) -> bool {
    matches!(s, "direct" | "group" | "channel" | "unknown")
}

// ---------- AuditActivityAgentActorTypeV1Schema ----------

/// Actor type for agent_run/tool_action/outbound_message records.
/// 对齐 TS: `Type.Union([Type.Literal("agent"), Type.Literal("system")])`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuditActivityAgentActorTypeV1Schema {
    Agent,
    System,
}

impl AuditActivityAgentActorTypeV1Schema {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Agent => "agent",
            Self::System => "system",
        }
    }
}

pub fn is_valid_audit_activity_agent_actor_type(s: &str) -> bool {
    matches!(s, "agent" | "system")
}

// ---------- AuditActivityInboundActorTypeV1Schema ----------

/// Actor type for inbound_message records.
/// 对齐 TS: `Type.Literal("channel_sender") | Type.Literal("system")`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuditActivityInboundActorTypeV1Schema {
    ChannelSender,
    System,
}

impl AuditActivityInboundActorTypeV1Schema {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::ChannelSender => "channel_sender",
            Self::System => "system",
        }
    }
}

pub fn is_valid_audit_activity_inbound_actor_type(s: &str) -> bool {
    matches!(s, "channel_sender" | "system")
}

// ---------- AuditActivityAgentRunActionV1Schema ----------

/// Action discriminator for agent-run records.
/// 对齐 TS: `Type.Union([
///   Type.Literal("agent.run.started"), Type.Literal("agent.run.finished"),
/// ])`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AuditActivityAgentRunActionV1Schema {
    #[serde(rename = "agent.run.started")]
    AgentRunStarted,
    #[serde(rename = "agent.run.finished")]
    AgentRunFinished,
}

impl AuditActivityAgentRunActionV1Schema {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::AgentRunStarted => "agent.run.started",
            Self::AgentRunFinished => "agent.run.finished",
        }
    }
}

pub fn is_valid_audit_activity_agent_run_action(s: &str) -> bool {
    matches!(s, "agent.run.started" | "agent.run.finished")
}

// ---------- AuditActivityAgentRunStatusV1Schema ----------

/// Status subset for agent-run records (excludes `unknown`).
/// 对齐 TS:
///   `Type.Union([
///      Type.Literal("started"), Type.Literal("succeeded"),
///      Type.Literal("failed"), Type.Literal("cancelled"),
///      Type.Literal("timed_out"), Type.Literal("blocked"),
///   ])`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuditActivityAgentRunStatusV1Schema {
    Started,
    Succeeded,
    Failed,
    Cancelled,
    TimedOut,
    Blocked,
}

impl AuditActivityAgentRunStatusV1Schema {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Started => "started",
            Self::Succeeded => "succeeded",
            Self::Failed => "failed",
            Self::Cancelled => "cancelled",
            Self::TimedOut => "timed_out",
            Self::Blocked => "blocked",
        }
    }
}

pub fn is_valid_audit_activity_agent_run_status(s: &str) -> bool {
    matches!(
        s,
        "started" | "succeeded" | "failed" | "cancelled" | "timed_out" | "blocked"
    )
}

// ---------- AuditActivityAgentRunErrorCodeV1Schema ----------

/// Error code discriminator for agent-run records.
/// 对齐 TS:
///   `Type.Union([
///      Type.Literal("run_failed"), Type.Literal("run_cancelled"),
///      Type.Literal("run_timed_out"), Type.Literal("run_blocked"),
///   ])`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuditActivityAgentRunErrorCodeV1Schema {
    RunFailed,
    RunCancelled,
    RunTimedOut,
    RunBlocked,
}

impl AuditActivityAgentRunErrorCodeV1Schema {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::RunFailed => "run_failed",
            Self::RunCancelled => "run_cancelled",
            Self::RunTimedOut => "run_timed_out",
            Self::RunBlocked => "run_blocked",
        }
    }
}

pub fn is_valid_audit_activity_agent_run_error_code(s: &str) -> bool {
    matches!(s, "run_failed" | "run_cancelled" | "run_timed_out" | "run_blocked")
}

// ---------- AuditActivityToolActionActionV1Schema ----------

/// Action discriminator for tool-action records.
/// 对齐 TS: `Type.Union([
///   Type.Literal("tool.action.started"), Type.Literal("tool.action.finished"),
/// ])`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AuditActivityToolActionActionV1Schema {
    #[serde(rename = "tool.action.started")]
    ToolActionStarted,
    #[serde(rename = "tool.action.finished")]
    ToolActionFinished,
}

impl AuditActivityToolActionActionV1Schema {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::ToolActionStarted => "tool.action.started",
            Self::ToolActionFinished => "tool.action.finished",
        }
    }
}

pub fn is_valid_audit_activity_tool_action_action(s: &str) -> bool {
    matches!(s, "tool.action.started" | "tool.action.finished")
}

// ---------- AuditActivityToolActionErrorCodeV1Schema ----------

/// Error code discriminator for tool-action records.
/// 对齐 TS:
///   `Type.Union([
///      Type.Literal("tool_failed"), Type.Literal("tool_cancelled"),
///      Type.Literal("tool_timed_out"), Type.Literal("tool_blocked"),
///      Type.Literal("tool_outcome_unknown"),
///   ])`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuditActivityToolActionErrorCodeV1Schema {
    ToolFailed,
    ToolCancelled,
    ToolTimedOut,
    ToolBlocked,
    ToolOutcomeUnknown,
}

impl AuditActivityToolActionErrorCodeV1Schema {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::ToolFailed => "tool_failed",
            Self::ToolCancelled => "tool_cancelled",
            Self::ToolTimedOut => "tool_timed_out",
            Self::ToolBlocked => "tool_blocked",
            Self::ToolOutcomeUnknown => "tool_outcome_unknown",
        }
    }
}

pub fn is_valid_audit_activity_tool_action_error_code(s: &str) -> bool {
    matches!(
        s,
        "tool_failed"
            | "tool_cancelled"
            | "tool_timed_out"
            | "tool_blocked"
            | "tool_outcome_unknown"
    )
}

// ---------- AuditActivityInboundActionV1Schema ----------

/// Action literal for inbound-message records (always fixed).
/// 对齐 TS: `Type.Literal("message.inbound.processed")`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AuditActivityInboundActionV1Schema {
    #[serde(rename = "message.inbound.processed")]
    MessageInboundProcessed,
}

impl AuditActivityInboundActionV1Schema {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::MessageInboundProcessed => "message.inbound.processed",
        }
    }
}

pub fn is_valid_audit_activity_inbound_action(s: &str) -> bool {
    matches!(s, "message.inbound.processed")
}

// ---------- AuditActivityInboundStatusV1Schema ----------

/// Status subset for inbound-message records.
/// 对齐 TS:
///   `Type.Union([
///      Type.Literal("succeeded"), Type.Literal("blocked"), Type.Literal("failed"),
///   ])`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuditActivityInboundStatusV1Schema {
    Succeeded,
    Blocked,
    Failed,
}

impl AuditActivityInboundStatusV1Schema {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Succeeded => "succeeded",
            Self::Blocked => "blocked",
            Self::Failed => "failed",
        }
    }
}

pub fn is_valid_audit_activity_inbound_status(s: &str) -> bool {
    matches!(s, "succeeded" | "blocked" | "failed")
}

// ---------- AuditActivityInboundOutcomeV1Schema ----------

/// Outcome discriminator for inbound-message records.
/// 对齐 TS:
///   `Type.Union([
///      Type.Literal("completed"), Type.Literal("skipped"), Type.Literal("failed"),
///   ])`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuditActivityInboundOutcomeV1Schema {
    Completed,
    Skipped,
    Failed,
}

impl AuditActivityInboundOutcomeV1Schema {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Completed => "completed",
            Self::Skipped => "skipped",
            Self::Failed => "failed",
        }
    }
}

pub fn is_valid_audit_activity_inbound_outcome(s: &str) -> bool {
    matches!(s, "completed" | "skipped" | "failed")
}

// ---------- AuditActivityInboundErrorCodeV1Schema ----------

/// Error code literal for inbound-message records (only present on `failed`).
/// 对齐 TS: `Type.Literal("message_processing_failed")`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename = "message_processing_failed")]
pub enum AuditActivityInboundErrorCodeV1Schema {
    MessageProcessingFailed,
}

impl AuditActivityInboundErrorCodeV1Schema {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::MessageProcessingFailed => "message_processing_failed",
        }
    }
}

pub fn is_valid_audit_activity_inbound_error_code(s: &str) -> bool {
    matches!(s, "message_processing_failed")
}

// ---------- AuditActivityInboundCompletedReasonV1Schema ----------

/// Completed reason codes for inbound-message records (`status=succeeded`).
/// 对齐 TS: `inboundCompletedReasonSchema`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuditActivityInboundCompletedReasonV1Schema {
    FastAbort,
    PluginBoundHandled,
    PluginBoundUnavailable,
    PluginBoundDeclined,
    BeforeDispatchHandled,
    AcpDispatchCompleted,
    AcpDispatchEmpty,
}

impl AuditActivityInboundCompletedReasonV1Schema {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::FastAbort => "fast_abort",
            Self::PluginBoundHandled => "plugin_bound_handled",
            Self::PluginBoundUnavailable => "plugin_bound_unavailable",
            Self::PluginBoundDeclined => "plugin_bound_declined",
            Self::BeforeDispatchHandled => "before_dispatch_handled",
            Self::AcpDispatchCompleted => "acp_dispatch_completed",
            Self::AcpDispatchEmpty => "acp_dispatch_empty",
        }
    }
}

pub fn is_valid_audit_activity_inbound_completed_reason(s: &str) -> bool {
    matches!(
        s,
        "fast_abort"
            | "plugin_bound_handled"
            | "plugin_bound_unavailable"
            | "plugin_bound_declined"
            | "before_dispatch_handled"
            | "acp_dispatch_completed"
            | "acp_dispatch_empty"
    )
}

// ---------- AuditActivityInboundSkippedReasonV1Schema ----------

/// Skipped reason codes for inbound-message records (`status=blocked`).
/// 对齐 TS: `inboundSkippedReasonSchema`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuditActivityInboundSkippedReasonV1Schema {
    Duplicate,
    ReplyOperationActive,
    ReplyOperationAborted,
    AcpDispatchAborted,
}

impl AuditActivityInboundSkippedReasonV1Schema {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Duplicate => "duplicate",
            Self::ReplyOperationActive => "reply_operation_active",
            Self::ReplyOperationAborted => "reply_operation_aborted",
            Self::AcpDispatchAborted => "acp_dispatch_aborted",
        }
    }
}

pub fn is_valid_audit_activity_inbound_skipped_reason(s: &str) -> bool {
    matches!(
        s,
        "duplicate" | "reply_operation_active" | "reply_operation_aborted" | "acp_dispatch_aborted"
    )
}

// ---------- AuditActivityInboundFailureReasonV1Schema ----------

/// Failure reason codes for inbound-message records (`status=failed`).
/// 对齐 TS: `inboundFailureReasonSchema`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuditActivityInboundFailureReasonV1Schema {
    AcpDispatchFailed,
    PluginBoundError,
}

impl AuditActivityInboundFailureReasonV1Schema {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::AcpDispatchFailed => "acp_dispatch_failed",
            Self::PluginBoundError => "plugin_bound_error",
        }
    }
}

pub fn is_valid_audit_activity_inbound_failure_reason(s: &str) -> bool {
    matches!(s, "acp_dispatch_failed" | "plugin_bound_error")
}

// ---------- AuditActivityInboundReasonCodeV1Schema ----------

/// Wire-level union of all inbound reason codes. Per-(status,outcome) subset
/// enforcement happens in `AuditActivityInboundMessageV1Schema::validate()`.
/// 对齐 TS: `reasonCode: Type.Optional(Type.Union([
///   ...inboundCompletedReasonSchema.anyOf,
///   ...inboundSkippedReasonSchema.anyOf,
///   ...inboundFailureReasonSchema.anyOf,
/// ]))`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuditActivityInboundReasonCodeV1Schema {
    FastAbort,
    PluginBoundHandled,
    PluginBoundUnavailable,
    PluginBoundDeclined,
    BeforeDispatchHandled,
    AcpDispatchCompleted,
    AcpDispatchEmpty,
    Duplicate,
    ReplyOperationActive,
    ReplyOperationAborted,
    AcpDispatchAborted,
    AcpDispatchFailed,
    PluginBoundError,
}

impl AuditActivityInboundReasonCodeV1Schema {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::FastAbort => "fast_abort",
            Self::PluginBoundHandled => "plugin_bound_handled",
            Self::PluginBoundUnavailable => "plugin_bound_unavailable",
            Self::PluginBoundDeclined => "plugin_bound_declined",
            Self::BeforeDispatchHandled => "before_dispatch_handled",
            Self::AcpDispatchCompleted => "acp_dispatch_completed",
            Self::AcpDispatchEmpty => "acp_dispatch_empty",
            Self::Duplicate => "duplicate",
            Self::ReplyOperationActive => "reply_operation_active",
            Self::ReplyOperationAborted => "reply_operation_aborted",
            Self::AcpDispatchAborted => "acp_dispatch_aborted",
            Self::AcpDispatchFailed => "acp_dispatch_failed",
            Self::PluginBoundError => "plugin_bound_error",
        }
    }

    pub fn is_completed_reason(&self) -> bool {
        matches!(
            self,
            Self::FastAbort
                | Self::PluginBoundHandled
                | Self::PluginBoundUnavailable
                | Self::PluginBoundDeclined
                | Self::BeforeDispatchHandled
                | Self::AcpDispatchCompleted
                | Self::AcpDispatchEmpty
        )
    }

    pub fn is_skipped_reason(&self) -> bool {
        matches!(
            self,
            Self::Duplicate
                | Self::ReplyOperationActive
                | Self::ReplyOperationAborted
                | Self::AcpDispatchAborted
        )
    }

    pub fn is_failure_reason(&self) -> bool {
        matches!(self, Self::AcpDispatchFailed | Self::PluginBoundError)
    }
}

pub fn is_valid_audit_activity_inbound_reason_code(s: &str) -> bool {
    matches!(
        s,
        "fast_abort"
            | "plugin_bound_handled"
            | "plugin_bound_unavailable"
            | "plugin_bound_declined"
            | "before_dispatch_handled"
            | "acp_dispatch_completed"
            | "acp_dispatch_empty"
            | "duplicate"
            | "reply_operation_active"
            | "reply_operation_aborted"
            | "acp_dispatch_aborted"
            | "acp_dispatch_failed"
            | "plugin_bound_error"
    )
}

// ---------- AuditActivityOutboundActionV1Schema ----------

/// Action literal for outbound-message records (always fixed).
/// 对齐 TS: `Type.Literal("message.outbound.finished")`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AuditActivityOutboundActionV1Schema {
    #[serde(rename = "message.outbound.finished")]
    MessageOutboundFinished,
}

impl AuditActivityOutboundActionV1Schema {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::MessageOutboundFinished => "message.outbound.finished",
        }
    }
}

pub fn is_valid_audit_activity_outbound_action(s: &str) -> bool {
    matches!(s, "message.outbound.finished")
}

// ---------- AuditActivityOutboundStatusV1Schema ----------

/// Status discriminator for outbound-message records.
/// 对齐 TS:
///   `Type.Union([
///      Type.Literal("succeeded"), Type.Literal("blocked"),
///      Type.Literal("failed"), Type.Literal("unknown"),
///   ])`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuditActivityOutboundStatusV1Schema {
    Succeeded,
    Blocked,
    Failed,
    Unknown,
}

impl AuditActivityOutboundStatusV1Schema {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Succeeded => "succeeded",
            Self::Blocked => "blocked",
            Self::Failed => "failed",
            Self::Unknown => "unknown",
        }
    }
}

pub fn is_valid_audit_activity_outbound_status(s: &str) -> bool {
    matches!(s, "succeeded" | "blocked" | "failed" | "unknown")
}

// ---------- AuditActivityOutboundOutcomeV1Schema ----------

/// Outcome discriminator for outbound-message records.
/// 对齐 TS:
///   `Type.Union([
///      Type.Literal("sent"), Type.Literal("suppressed"),
///      Type.Literal("failed"), Type.Literal("unknown"),
///   ])`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuditActivityOutboundOutcomeV1Schema {
    Sent,
    Suppressed,
    Failed,
    Unknown,
}

impl AuditActivityOutboundOutcomeV1Schema {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Sent => "sent",
            Self::Suppressed => "suppressed",
            Self::Failed => "failed",
            Self::Unknown => "unknown",
        }
    }
}

pub fn is_valid_audit_activity_outbound_outcome(s: &str) -> bool {
    matches!(s, "sent" | "suppressed" | "failed" | "unknown")
}

// ---------- AuditActivityOutboundFailureErrorV1Schema ----------

/// Failure error code discriminator for outbound-message records.
/// 对齐 TS: `outboundFailureErrorSchema`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuditActivityOutboundFailureErrorV1Schema {
    MessageDeliveryFailed,
    MessageDeliveryPartialFailure,
}

impl AuditActivityOutboundFailureErrorV1Schema {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::MessageDeliveryFailed => "message_delivery_failed",
            Self::MessageDeliveryPartialFailure => "message_delivery_partial_failure",
        }
    }
}

pub fn is_valid_audit_activity_outbound_failure_error(s: &str) -> bool {
    matches!(
        s,
        "message_delivery_failed" | "message_delivery_partial_failure"
    )
}

// ---------- AuditActivityOutboundSuppressedReasonV1Schema ----------

/// Suppressed reason codes for outbound-message records (`status=blocked`).
/// 对齐 TS: `outboundSuppressedReasonSchema`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuditActivityOutboundSuppressedReasonV1Schema {
    CancelledByMessageSendingHook,
    CancelledByReplyPayloadSendingHook,
    EmptyAfterMessageSendingHook,
    EmptyAfterReplyPayloadSendingHook,
    NoVisiblePayload,
}

impl AuditActivityOutboundSuppressedReasonV1Schema {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::CancelledByMessageSendingHook => "cancelled_by_message_sending_hook",
            Self::CancelledByReplyPayloadSendingHook => "cancelled_by_reply_payload_sending_hook",
            Self::EmptyAfterMessageSendingHook => "empty_after_message_sending_hook",
            Self::EmptyAfterReplyPayloadSendingHook => "empty_after_reply_payload_sending_hook",
            Self::NoVisiblePayload => "no_visible_payload",
        }
    }
}

pub fn is_valid_audit_activity_outbound_suppressed_reason(s: &str) -> bool {
    matches!(
        s,
        "cancelled_by_message_sending_hook"
            | "cancelled_by_reply_payload_sending_hook"
            | "empty_after_message_sending_hook"
            | "empty_after_reply_payload_sending_hook"
            | "no_visible_payload"
    )
}

// ---------- AuditActivityOutboundReasonCodeV1Schema ----------

/// Wire-level reason code for outbound-message records.
/// 对齐 TS: `reasonCode: Type.Optional(outboundSuppressedReasonSchema)`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuditActivityOutboundReasonCodeV1Schema {
    CancelledByMessageSendingHook,
    CancelledByReplyPayloadSendingHook,
    EmptyAfterMessageSendingHook,
    EmptyAfterReplyPayloadSendingHook,
    NoVisiblePayload,
}

impl AuditActivityOutboundReasonCodeV1Schema {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::CancelledByMessageSendingHook => "cancelled_by_message_sending_hook",
            Self::CancelledByReplyPayloadSendingHook => "cancelled_by_reply_payload_sending_hook",
            Self::EmptyAfterMessageSendingHook => "empty_after_message_sending_hook",
            Self::EmptyAfterReplyPayloadSendingHook => "empty_after_reply_payload_sending_hook",
            Self::NoVisiblePayload => "no_visible_payload",
        }
    }
}

pub fn is_valid_audit_activity_outbound_reason_code(s: &str) -> bool {
    matches!(
        s,
        "cancelled_by_message_sending_hook"
            | "cancelled_by_reply_payload_sending_hook"
            | "empty_after_message_sending_hook"
            | "empty_after_reply_payload_sending_hook"
            | "no_visible_payload"
    )
}

// ---------- AuditActivityOutboundFailureStageV1Schema ----------

/// Failure stage discriminator for outbound-message records.
/// 对齐 TS: `outboundFailureStageSchema`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuditActivityOutboundFailureStageV1Schema {
    PlatformSend,
    Queue,
    Unknown,
}

impl AuditActivityOutboundFailureStageV1Schema {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::PlatformSend => "platform_send",
            Self::Queue => "queue",
            Self::Unknown => "unknown",
        }
    }
}

pub fn is_valid_audit_activity_outbound_failure_stage(s: &str) -> bool {
    matches!(s, "platform_send" | "queue" | "unknown")
}

// ---------- AuditActivityOutboundDeliveryKindV1Schema ----------

/// Delivery kind discriminator for outbound-message records.
/// 对齐 TS:
///   `Type.Optional(Type.Union([
///      Type.Literal("text"), Type.Literal("media"), Type.Literal("other"),
///   ]))`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuditActivityOutboundDeliveryKindV1Schema {
    Text,
    Media,
    Other,
}

impl AuditActivityOutboundDeliveryKindV1Schema {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Text => "text",
            Self::Media => "media",
            Self::Other => "other",
        }
    }
}

pub fn is_valid_audit_activity_outbound_delivery_kind(s: &str) -> bool {
    matches!(s, "text" | "media" | "other")
}

// ============================================================================
// Actor 共享结构
// ============================================================================

// ---------- AuditActivityAgentActorV1Schema ----------

/// Actor descriptor for agent-run / tool-action / outbound-message records.
/// 对齐 TS:
///   `AuditActivityAgentActorV1Schema = Type.Object({
///      type: Type.Union([Type.Literal("agent"), Type.Literal("system")]),
///      id: NonEmptyString,
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuditActivityAgentActorV1Schema {
    #[serde(rename = "type")]
    pub actor_type: AuditActivityAgentActorTypeV1Schema,
    pub id: String,
}

impl AuditActivityAgentActorV1Schema {
    pub fn validate(&self) -> Result<(), String> {
        validate_non_empty_string("id", &self.id)?;
        Ok(())
    }
}

/// 对齐 TS:
///   `AuditActivityOutboundActorV1Schema = Type.Object({
///      type: Type.Union([Type.Literal("agent"), Type.Literal("system")]),
///      id: NonEmptyString,
///   }, { additionalProperties: false })`
/// (结构与 `AuditActivityAgentActorV1Schema` 完全相同, 这里用 type alias 复用).
pub type AuditActivityOutboundActorV1Schema = AuditActivityAgentActorV1Schema;

// ---------- AuditActivityInboundActorV1Schema ----------

/// Inbound actor descriptor (tagged union over `type`).
/// 对齐 TS:
///   `AuditActivityInboundActorV1Schema = Type.Union([
///      Type.Object({
///        type: Type.Literal("channel_sender"),
///        id: AuditActivityHmacRefV1Schema,
///      }, { additionalProperties: false }),
///      Type.Object({
///        type: Type.Literal("system"),
///        id: NonEmptyString,
///      }, { additionalProperties: false }),
///   ])`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AuditActivityInboundActorV1Schema {
    /// `channel_sender`: `id` 必须匹配 HMAC ref pattern.
    ChannelSender { id: String },
    /// `system`: `id` 是非空字符串.
    System { id: String },
}

impl AuditActivityInboundActorV1Schema {
    pub fn validate(&self) -> Result<(), String> {
        match self {
            AuditActivityInboundActorV1Schema::ChannelSender { id } => {
                validate_pattern("id", id, AUDIT_ACTIVITY_HMAC_REF_PATTERN)?;
            }
            AuditActivityInboundActorV1Schema::System { id } => {
                validate_non_empty_string("id", id)?;
            }
        }
        Ok(())
    }
}

// ============================================================================
// 基础 record 验证 (common fields shared by every activity record)
// ============================================================================

/// 校验 `AuditActivityEventV1Schema` 四类 record 共有的字段 (commonProperties).
fn validate_record_common(
    schema_version: i64,
    event_id: &str,
    sequence: i64,
    source_sequence: i64,
    occurred_at: i64,
    redaction: &str,
    actor_id: &str,
    run_id: &str,
    agent_id: Option<&str>,
    session_key: Option<&str>,
    session_id: Option<&str>,
) -> Result<(), String> {
    validate_integer_in_range(
        "schemaVersion",
        schema_version,
        AUDIT_ACTIVITY_SCHEMA_VERSION_V1,
        AUDIT_ACTIVITY_SCHEMA_VERSION_V1,
    )?;
    validate_non_empty_string("eventId", event_id)?;
    validate_positive_integer("sequence", sequence)?;
    validate_positive_integer("sourceSequence", source_sequence)?;
    validate_non_negative_integer("occurredAt", occurred_at)?;
    if redaction != AUDIT_ACTIVITY_REDACTION_METADATA_ONLY {
        return Err(format!(
            "redaction: expected {:?}, got {:?}",
            AUDIT_ACTIVITY_REDACTION_METADATA_ONLY, redaction
        ));
    }
    validate_non_empty_string("actor.id", actor_id)?;
    validate_optional_non_empty_string("agentId", agent_id)?;
    validate_optional_non_empty_string("sessionKey", session_key)?;
    validate_optional_non_empty_string("sessionId", session_id)?;
    validate_non_empty_string("runId", run_id)?;
    Ok(())
}

// ============================================================================
// Agent-run record
// ============================================================================

/// V1 agent-run activity record.
/// 对齐 TS:
///   `AuditActivityAgentRunV1Schema = correlatedObject(
///      {
///        ...agentRunProperties, // eventType, schemaVersion, eventId,
///                              // sequence, sourceSequence, occurredAt,
///                              // redaction, actor, agentId, sessionKey?,
///                              // sessionId?, runId, kind
///        action: Type.Union([Type.Literal("agent.run.started"),
///                            Type.Literal("agent.run.finished")]),
///        status: Type.Union([Type.Literal("started"), Type.Literal("succeeded"),
///                            Type.Literal("failed"), Type.Literal("cancelled"),
///                            Type.Literal("timed_out"), Type.Literal("blocked")]),
///        errorCode: Type.Optional(Type.Union([
///            Type.Literal("run_failed"), Type.Literal("run_cancelled"),
///            Type.Literal("run_timed_out"), Type.Literal("run_blocked"),
///        ])),
///      },
///      Type.Union([ /* (action, status, errorCode) correlation */ ]),
///   )`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuditActivityAgentRunV1Schema {
    pub schema_version: i64,
    pub event_id: String,
    pub sequence: i64,
    pub source_sequence: i64,
    pub occurred_at: i64,
    pub redaction: String,
    pub event_type: AuditActivityEventTypeV1Schema,
    pub kind: AuditActivityKindV1Schema,
    pub action: AuditActivityAgentRunActionV1Schema,
    pub status: AuditActivityAgentRunStatusV1Schema,
    pub actor: AuditActivityAgentActorV1Schema,
    pub agent_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub session_key: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
    pub run_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error_code: Option<AuditActivityAgentRunErrorCodeV1Schema>,
}

impl AuditActivityAgentRunV1Schema {
    pub fn validate(&self) -> Result<(), String> {
        self.actor.validate().map_err(|e| format!("actor: {}", e))?;
        validate_record_common(
            self.schema_version,
            &self.event_id,
            self.sequence,
            self.source_sequence,
            self.occurred_at,
            &self.redaction,
            &self.actor.id,
            &self.run_id,
            Some(&self.agent_id),
            self.session_key.as_deref(),
            self.session_id.as_deref(),
        )?;
        if self.event_type != AuditActivityEventTypeV1Schema::AgentRun {
            return Err(format!(
                "eventType: expected agent_run, got {}",
                self.event_type.as_str()
            ));
        }
        if self.kind != AuditActivityKindV1Schema::AgentRun {
            return Err(format!(
                "kind: expected agent_run, got {}",
                self.kind.as_str()
            ));
        }
        // action+status+errorCode correlation, 对应 TS `correlatedObject(... allOf ...)`.
        let ec = self.error_code.as_ref().map(|c| c.as_str());
        let ok = matches!(
            (self.action, self.status, &self.error_code),
            (AuditActivityAgentRunActionV1Schema::AgentRunStarted, AuditActivityAgentRunStatusV1Schema::Started, None)
            | (AuditActivityAgentRunActionV1Schema::AgentRunFinished, AuditActivityAgentRunStatusV1Schema::Succeeded, None)
            | (AuditActivityAgentRunActionV1Schema::AgentRunFinished, AuditActivityAgentRunStatusV1Schema::Failed, Some(AuditActivityAgentRunErrorCodeV1Schema::RunFailed))
            | (AuditActivityAgentRunActionV1Schema::AgentRunFinished, AuditActivityAgentRunStatusV1Schema::Cancelled, Some(AuditActivityAgentRunErrorCodeV1Schema::RunCancelled))
            | (AuditActivityAgentRunActionV1Schema::AgentRunFinished, AuditActivityAgentRunStatusV1Schema::TimedOut, Some(AuditActivityAgentRunErrorCodeV1Schema::RunTimedOut))
            | (AuditActivityAgentRunActionV1Schema::AgentRunFinished, AuditActivityAgentRunStatusV1Schema::Blocked, Some(AuditActivityAgentRunErrorCodeV1Schema::RunBlocked))
        );
        if !ok {
            return Err(format!(
                "invalid (action, status, errorCode) correlation: ({}, {}, {:?})",
                self.action.as_str(),
                self.status.as_str(),
                ec
            ));
        }
        Ok(())
    }
}

// ============================================================================
// Tool-action record
// ============================================================================

/// V1 tool-action activity record.
/// 对齐 TS:
///   `AuditActivityToolActionV1Schema = correlatedObject(
///      {
///        ...toolActionProperties, // eventType, schemaVersion, eventId,
///                                // sequence, sourceSequence, occurredAt,
///                                // redaction, actor, agentId, sessionKey?,
///                                // sessionId?, runId, kind, toolCallId?,
///                                // toolName?
///        action: Type.Union([Type.Literal("tool.action.started"),
///                            Type.Literal("tool.action.finished")]),
///        status: AuditActivityStatusV1Schema,
///        errorCode: Type.Optional(Type.Union([
///            Type.Literal("tool_failed"), Type.Literal("tool_cancelled"),
///            Type.Literal("tool_timed_out"), Type.Literal("tool_blocked"),
///            Type.Literal("tool_outcome_unknown"),
///        ])),
///      },
///      Type.Union([ /* (action, status, errorCode) correlation */ ]),
///   )`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuditActivityToolActionV1Schema {
    pub schema_version: i64,
    pub event_id: String,
    pub sequence: i64,
    pub source_sequence: i64,
    pub occurred_at: i64,
    pub redaction: String,
    pub event_type: AuditActivityEventTypeV1Schema,
    pub kind: AuditActivityKindV1Schema,
    pub action: AuditActivityToolActionActionV1Schema,
    pub status: AuditActivityStatusV1Schema,
    pub actor: AuditActivityAgentActorV1Schema,
    pub agent_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub session_key: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
    pub run_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error_code: Option<AuditActivityToolActionErrorCodeV1Schema>,
}

impl AuditActivityToolActionV1Schema {
    pub fn validate(&self) -> Result<(), String> {
        self.actor.validate().map_err(|e| format!("actor: {}", e))?;
        validate_record_common(
            self.schema_version,
            &self.event_id,
            self.sequence,
            self.source_sequence,
            self.occurred_at,
            &self.redaction,
            &self.actor.id,
            &self.run_id,
            Some(&self.agent_id),
            self.session_key.as_deref(),
            self.session_id.as_deref(),
        )?;
        validate_optional_non_empty_string("toolCallId", self.tool_call_id.as_deref())?;
        validate_optional_non_empty_string("toolName", self.tool_name.as_deref())?;
        if self.event_type != AuditActivityEventTypeV1Schema::ToolAction {
            return Err(format!(
                "eventType: expected tool_action, got {}",
                self.event_type.as_str()
            ));
        }
        if self.kind != AuditActivityKindV1Schema::ToolAction {
            return Err(format!(
                "kind: expected tool_action, got {}",
                self.kind.as_str()
            ));
        }
        // action+status+errorCode correlation, 对应 TS `correlatedObject(... allOf ...)`.
        let ok = matches!(
            (self.action, self.status, &self.error_code),
            (AuditActivityToolActionActionV1Schema::ToolActionStarted, AuditActivityStatusV1Schema::Started, None)
            | (AuditActivityToolActionActionV1Schema::ToolActionFinished, AuditActivityStatusV1Schema::Succeeded, None)
            | (AuditActivityToolActionActionV1Schema::ToolActionFinished, AuditActivityStatusV1Schema::Failed, Some(AuditActivityToolActionErrorCodeV1Schema::ToolFailed))
            | (AuditActivityToolActionActionV1Schema::ToolActionFinished, AuditActivityStatusV1Schema::Cancelled, Some(AuditActivityToolActionErrorCodeV1Schema::ToolCancelled))
            | (AuditActivityToolActionActionV1Schema::ToolActionFinished, AuditActivityStatusV1Schema::TimedOut, Some(AuditActivityToolActionErrorCodeV1Schema::ToolTimedOut))
            | (AuditActivityToolActionActionV1Schema::ToolActionFinished, AuditActivityStatusV1Schema::Blocked, Some(AuditActivityToolActionErrorCodeV1Schema::ToolBlocked))
            | (AuditActivityToolActionActionV1Schema::ToolActionFinished, AuditActivityStatusV1Schema::Unknown, Some(AuditActivityToolActionErrorCodeV1Schema::ToolOutcomeUnknown))
        );
        if !ok {
            return Err(format!(
                "invalid (action, status, errorCode) correlation: ({}, {}, {:?})",
                self.action.as_str(),
                self.status.as_str(),
                self.error_code.as_ref().map(|c| c.as_str())
            ));
        }
        Ok(())
    }
}

// ============================================================================
// Inbound-message record
// ============================================================================

/// V1 inbound-message activity record.
/// 对齐 TS:
///   `AuditActivityInboundMessageV1Schema = correlatedObject(
///      {
///        ...inboundMessageProperties, // eventType, schemaVersion, eventId,
///                                    // sequence, sourceSequence, occurredAt,
///                                    // redaction, channel, conversationKind,
///                                    // durationMs?, resultCount?, agentId?,
///                                    // runId?, accountRef?, conversationRef?,
///                                    // messageRef?, targetRef?, kind, action,
///                                    // direction, actor
///        status:  Type.Union([Type.Literal("succeeded"), Type.Literal("blocked"),
///                            Type.Literal("failed")]),
///        outcome: Type.Union([Type.Literal("completed"), Type.Literal("skipped"),
///                            Type.Literal("failed")]),
///        errorCode: Type.Optional(Type.Literal("message_processing_failed")),
///        reasonCode: Type.Optional(Type.Union([ /* completed/skipped/failure */ ])),
///      },
///      Type.Union([ /* (status, outcome, errorCode, reasonCode) correlation */ ]),
///   )`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuditActivityInboundMessageV1Schema {
    pub schema_version: i64,
    pub event_id: String,
    pub sequence: i64,
    pub source_sequence: i64,
    pub occurred_at: i64,
    pub redaction: String,
    pub event_type: AuditActivityEventTypeV1Schema,
    pub kind: AuditActivityKindV1Schema,
    pub channel: String,
    pub conversation_kind: AuditActivityConversationKindV1Schema,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub duration_ms: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub result_count: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub agent_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub run_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub account_ref: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub conversation_ref: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message_ref: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target_ref: Option<String>,
    pub action: AuditActivityInboundActionV1Schema,
    pub direction: AuditActivityDirectionV1Schema,
    pub actor: AuditActivityInboundActorV1Schema,
    pub status: AuditActivityInboundStatusV1Schema,
    pub outcome: AuditActivityInboundOutcomeV1Schema,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error_code: Option<AuditActivityInboundErrorCodeV1Schema>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason_code: Option<AuditActivityInboundReasonCodeV1Schema>,
}

impl AuditActivityInboundMessageV1Schema {
    pub fn validate(&self) -> Result<(), String> {
        self.actor.validate().map_err(|e| format!("actor: {}", e))?;
        // 注: commonProperties 中的 actor.id/runner 字段对 inbound_message 不适用,
        // 这里只校验通用 (schemaVersion/eventId/sequence/sourceSequence/occurredAt/redaction).
        validate_integer_in_range(
            "schemaVersion",
            self.schema_version,
            AUDIT_ACTIVITY_SCHEMA_VERSION_V1,
            AUDIT_ACTIVITY_SCHEMA_VERSION_V1,
        )?;
        validate_non_empty_string("eventId", &self.event_id)?;
        validate_positive_integer("sequence", self.sequence)?;
        validate_positive_integer("sourceSequence", self.source_sequence)?;
        validate_non_negative_integer("occurredAt", self.occurred_at)?;
        if self.redaction != AUDIT_ACTIVITY_REDACTION_METADATA_ONLY {
            return Err(format!(
                "redaction: expected {:?}, got {:?}",
                AUDIT_ACTIVITY_REDACTION_METADATA_ONLY, self.redaction
            ));
        }
        validate_non_empty_string("channel", &self.channel)?;
        validate_optional_non_negative_integer("durationMs", self.duration_ms)?;
        validate_optional_non_negative_integer("resultCount", self.result_count)?;
        validate_optional_non_empty_string("agentId", self.agent_id.as_deref())?;
        validate_optional_non_empty_string("runId", self.run_id.as_deref())?;
        validate_optional_pattern("accountRef", self.account_ref.as_deref(), AUDIT_ACTIVITY_HMAC_REF_PATTERN)?;
        validate_optional_pattern(
            "conversationRef",
            self.conversation_ref.as_deref(),
            AUDIT_ACTIVITY_HMAC_REF_PATTERN,
        )?;
        validate_optional_pattern(
            "messageRef",
            self.message_ref.as_deref(),
            AUDIT_ACTIVITY_HMAC_REF_PATTERN,
        )?;
        validate_optional_pattern(
            "targetRef",
            self.target_ref.as_deref(),
            AUDIT_ACTIVITY_HMAC_REF_PATTERN,
        )?;
        if self.event_type != AuditActivityEventTypeV1Schema::InboundMessage {
            return Err(format!(
                "eventType: expected inbound_message, got {}",
                self.event_type.as_str()
            ));
        }
        if self.kind != AuditActivityKindV1Schema::Message {
            return Err(format!(
                "kind: expected message, got {}",
                self.kind.as_str()
            ));
        }
        if self.direction != AuditActivityDirectionV1Schema::Inbound {
            return Err(format!(
                "direction: expected inbound, got {}",
                self.direction.as_str()
            ));
        }
        // status/outcome/errorCode/reasonCode correlation, 对应 TS `correlatedObject(...)`.
        let status = self.status;
        let outcome = self.outcome;
        let ec = self.error_code;
        let rc = self.reason_code.as_ref();
        match (status, outcome, ec) {
            (
                AuditActivityInboundStatusV1Schema::Succeeded,
                AuditActivityInboundOutcomeV1Schema::Completed,
                None,
            ) => {
                if let Some(r) = rc {
                    if !r.is_completed_reason() {
                        return Err(format!(
                            "reasonCode: expected completed reason for status=\
                             succeeded/outcome=completed, got {}",
                            r.as_str()
                        ));
                    }
                }
                Ok(())
            }
            (
                AuditActivityInboundStatusV1Schema::Blocked,
                AuditActivityInboundOutcomeV1Schema::Skipped,
                None,
            ) => {
                if let Some(r) = rc {
                    if !r.is_skipped_reason() {
                        return Err(format!(
                            "reasonCode: expected skipped reason for status=\
                             blocked/outcome=skipped, got {}",
                            r.as_str()
                        ));
                    }
                }
                Ok(())
            }
            (
                AuditActivityInboundStatusV1Schema::Failed,
                AuditActivityInboundOutcomeV1Schema::Failed,
                Some(AuditActivityInboundErrorCodeV1Schema::MessageProcessingFailed),
            ) => {
                if let Some(r) = rc {
                    if !r.is_failure_reason() {
                        return Err(format!(
                            "reasonCode: expected failure reason for status=\
                             failed/outcome=failed, got {}",
                            r.as_str()
                        ));
                    }
                }
                Ok(())
            }
            _ => Err(format!(
                "invalid (status, outcome, errorCode) correlation: ({}, {}, {:?})",
                status.as_str(),
                outcome.as_str(),
                ec.map(|c| c.as_str())
            )),
        }
    }
}

// ============================================================================
// Outbound-message record
// ============================================================================

/// V1 outbound-message activity record.
/// 对齐 TS:
///   `AuditActivityOutboundMessageV1Schema = correlatedObject(
///      {
///        ...outboundMessageProperties, // eventType, schemaVersion, eventId,
///                                     // sequence, sourceSequence, occurredAt,
///                                     // redaction, channel, conversationKind,
///                                     // durationMs?, resultCount?, agentId?,
///                                     // runId?, accountRef?, conversationRef?,
///                                     // messageRef?, targetRef?, kind, action,
///                                     // direction, actor, deliveryKind?
///        status:  Type.Union([Type.Literal("succeeded"), Type.Literal("blocked"),
///                            Type.Literal("failed"), Type.Literal("unknown")]),
///        outcome: Type.Union([Type.Literal("sent"), Type.Literal("suppressed"),
///                            Type.Literal("failed"), Type.Literal("unknown")]),
///        errorCode: Type.Optional(outboundFailureErrorSchema),
///        reasonCode: Type.Optional(outboundSuppressedReasonSchema),
///        failureStage: Type.Optional(outboundFailureStageSchema),
///      },
///      Type.Union([ /* (status, outcome, errorCode/reasonCode/failureStage/
///                         deliveryKind) correlation */ ]),
///   )`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuditActivityOutboundMessageV1Schema {
    pub schema_version: i64,
    pub event_id: String,
    pub sequence: i64,
    pub source_sequence: i64,
    pub occurred_at: i64,
    pub redaction: String,
    pub event_type: AuditActivityEventTypeV1Schema,
    pub kind: AuditActivityKindV1Schema,
    pub channel: String,
    pub conversation_kind: AuditActivityConversationKindV1Schema,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub duration_ms: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub result_count: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub agent_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub run_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub account_ref: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub conversation_ref: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message_ref: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target_ref: Option<String>,
    pub action: AuditActivityOutboundActionV1Schema,
    pub direction: AuditActivityDirectionV1Schema,
    pub actor: AuditActivityOutboundActorV1Schema,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub delivery_kind: Option<AuditActivityOutboundDeliveryKindV1Schema>,
    pub status: AuditActivityOutboundStatusV1Schema,
    pub outcome: AuditActivityOutboundOutcomeV1Schema,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error_code: Option<AuditActivityOutboundFailureErrorV1Schema>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason_code: Option<AuditActivityOutboundReasonCodeV1Schema>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub failure_stage: Option<AuditActivityOutboundFailureStageV1Schema>,
}

impl AuditActivityOutboundMessageV1Schema {
    pub fn validate(&self) -> Result<(), String> {
        self.actor.validate().map_err(|e| format!("actor: {}", e))?;
        validate_integer_in_range(
            "schemaVersion",
            self.schema_version,
            AUDIT_ACTIVITY_SCHEMA_VERSION_V1,
            AUDIT_ACTIVITY_SCHEMA_VERSION_V1,
        )?;
        validate_non_empty_string("eventId", &self.event_id)?;
        validate_positive_integer("sequence", self.sequence)?;
        validate_positive_integer("sourceSequence", self.source_sequence)?;
        validate_non_negative_integer("occurredAt", self.occurred_at)?;
        if self.redaction != AUDIT_ACTIVITY_REDACTION_METADATA_ONLY {
            return Err(format!(
                "redaction: expected {:?}, got {:?}",
                AUDIT_ACTIVITY_REDACTION_METADATA_ONLY, self.redaction
            ));
        }
        validate_non_empty_string("channel", &self.channel)?;
        validate_optional_non_negative_integer("durationMs", self.duration_ms)?;
        validate_optional_non_negative_integer("resultCount", self.result_count)?;
        validate_optional_non_empty_string("agentId", self.agent_id.as_deref())?;
        validate_optional_non_empty_string("runId", self.run_id.as_deref())?;
        validate_optional_pattern(
            "accountRef",
            self.account_ref.as_deref(),
            AUDIT_ACTIVITY_HMAC_REF_PATTERN,
        )?;
        validate_optional_pattern(
            "conversationRef",
            self.conversation_ref.as_deref(),
            AUDIT_ACTIVITY_HMAC_REF_PATTERN,
        )?;
        validate_optional_pattern(
            "messageRef",
            self.message_ref.as_deref(),
            AUDIT_ACTIVITY_HMAC_REF_PATTERN,
        )?;
        validate_optional_pattern(
            "targetRef",
            self.target_ref.as_deref(),
            AUDIT_ACTIVITY_HMAC_REF_PATTERN,
        )?;
        if self.event_type != AuditActivityEventTypeV1Schema::OutboundMessage {
            return Err(format!(
                "eventType: expected outbound_message, got {}",
                self.event_type.as_str()
            ));
        }
        if self.kind != AuditActivityKindV1Schema::Message {
            return Err(format!(
                "kind: expected message, got {}",
                self.kind.as_str()
            ));
        }
        if self.direction != AuditActivityDirectionV1Schema::Outbound {
            return Err(format!(
                "direction: expected outbound, got {}",
                self.direction.as_str()
            ));
        }
        // status/outcome/errorCode/reasonCode/failureStage/deliveryKind correlation,
        // 对应 TS `correlatedObject(...)` 中的 allOf 集合.
        let status = self.status;
        let outcome = self.outcome;
        let ec = self.error_code.as_ref();
        let rc = self.reason_code.as_ref();
        let fs = self.failure_stage.as_ref();
        let dk = self.delivery_kind.as_ref();
        let ok: bool = match (status, outcome) {
            (
                AuditActivityOutboundStatusV1Schema::Succeeded,
                AuditActivityOutboundOutcomeV1Schema::Sent,
            ) => {
                // errorCode / reasonCode / failureStage 必须 absent;
                // deliveryKind 可选.
                ec.is_none() && rc.is_none() && fs.is_none()
            }
            (
                AuditActivityOutboundStatusV1Schema::Blocked,
                AuditActivityOutboundOutcomeV1Schema::Suppressed,
            ) => {
                // errorCode / failureStage / deliveryKind 必须 absent;
                // reasonCode 必须是 suppressed reason.
                if ec.is_some() || fs.is_some() || dk.is_some() {
                    false
                } else {
                    match rc {
                        Some(r) => matches!(
                            r,
                            AuditActivityOutboundReasonCodeV1Schema::CancelledByMessageSendingHook
                                | AuditActivityOutboundReasonCodeV1Schema::CancelledByReplyPayloadSendingHook
                                | AuditActivityOutboundReasonCodeV1Schema::EmptyAfterMessageSendingHook
                                | AuditActivityOutboundReasonCodeV1Schema::EmptyAfterReplyPayloadSendingHook
                                | AuditActivityOutboundReasonCodeV1Schema::NoVisiblePayload
                        ),
                        None => false,
                    }
                }
            }
            (
                AuditActivityOutboundStatusV1Schema::Failed,
                AuditActivityOutboundOutcomeV1Schema::Failed,
            ) => {
                // reasonCode 必须 absent;
                // errorCode 必须是 failure error;
                // failureStage 必须是 failure stage;
                // deliveryKind 可选.
                if rc.is_some() {
                    false
                } else {
                    let ec_ok = matches!(
                        ec,
                        Some(
                            AuditActivityOutboundFailureErrorV1Schema::MessageDeliveryFailed
                                | AuditActivityOutboundFailureErrorV1Schema::MessageDeliveryPartialFailure,
                        )
                    );
                    let fs_ok = matches!(
                        fs,
                        Some(
                            AuditActivityOutboundFailureStageV1Schema::PlatformSend
                                | AuditActivityOutboundFailureStageV1Schema::Queue
                                | AuditActivityOutboundFailureStageV1Schema::Unknown,
                        )
                    );
                    ec_ok && fs_ok
                }
            }
            (
                AuditActivityOutboundStatusV1Schema::Unknown,
                AuditActivityOutboundOutcomeV1Schema::Unknown,
            ) => {
                // errorCode / reasonCode / deliveryKind 必须 absent;
                // failureStage 必须是 failure stage.
                if ec.is_some() || rc.is_some() || dk.is_some() {
                    false
                } else {
                    matches!(
                        fs,
                        Some(
                            AuditActivityOutboundFailureStageV1Schema::PlatformSend
                                | AuditActivityOutboundFailureStageV1Schema::Queue
                                | AuditActivityOutboundFailureStageV1Schema::Unknown,
                        )
                    )
                }
            }
            _ => false,
        };
        if !ok {
            return Err(format!(
                "invalid (status, outcome, errorCode, reasonCode, failureStage, deliveryKind) \
                 correlation: ({}, {}, ec={:?}, rc={:?}, fs={:?}, dk={:?})",
                status.as_str(),
                outcome.as_str(),
                ec.map(|c| c.as_str()),
                rc.map(|c| c.as_str()),
                fs.map(|c| c.as_str()),
                dk.map(|c| c.as_str()),
            ));
        }
        Ok(())
    }
}

// ============================================================================
// Discriminated V1 activity event union
// ============================================================================

/// Discriminated V1 activity record union (`eventType` 字段做 discriminator).
/// 对齐 TS:
///   `AuditActivityEventV1Schema = Type.Union([
///      AuditActivityAgentRunV1Schema,
///      AuditActivityToolActionV1Schema,
///      AuditActivityInboundMessageV1Schema,
///      AuditActivityOutboundMessageV1Schema,
///   ])`.
///
/// 实现策略: serde untagged, 每个 variant 自身有 `eventType` 字面量字段,
/// 反序列化时按 enum 顺序尝试, 命中第一个 eventType 匹配的 variant.
/// JSON 序列化时直接展开内层 struct (不引入额外 wrapper).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum AuditActivityEventV1Schema {
    AgentRun(AuditActivityAgentRunV1Schema),
    ToolAction(AuditActivityToolActionV1Schema),
    InboundMessage(AuditActivityInboundMessageV1Schema),
    OutboundMessage(AuditActivityOutboundMessageV1Schema),
}

impl AuditActivityEventV1Schema {
    pub fn validate(&self) -> Result<(), String> {
        match self {
            AuditActivityEventV1Schema::AgentRun(e) => {
                e.validate().map_err(|e| format!("agentRun: {}", e))
            }
            AuditActivityEventV1Schema::ToolAction(e) => {
                e.validate().map_err(|e| format!("toolAction: {}", e))
            }
            AuditActivityEventV1Schema::InboundMessage(e) => {
                e.validate().map_err(|e| format!("inboundMessage: {}", e))
            }
            AuditActivityEventV1Schema::OutboundMessage(e) => {
                e.validate().map_err(|e| format!("outboundMessage: {}", e))
            }
        }
    }
}

// ============================================================================
// List params / result
// ============================================================================

// ---------- AuditActivityListParamsSchema ----------

/// Bounded newest-first V1 activity query filters.
/// 对齐 TS:
///   `AuditActivityListParamsSchema = Type.Object({
///      agentId:    Type.Optional(NonEmptyString),
///      sessionKey: Type.Optional(NonEmptyString),
///      runId:      Type.Optional(NonEmptyString),
///      kind:       Type.Optional(AuditActivityKindV1Schema),
///      status:     Type.Optional(AuditActivityStatusV1Schema),
///      direction:  Type.Optional(AuditActivityDirectionV1Schema),
///      channel:    Type.Optional(NonEmptyString),
///      after:      Type.Optional(Type.Integer({ minimum: 0 })),
///      before:     Type.Optional(Type.Integer({ minimum: 0 })),
///      limit:      Type.Optional(Type.Integer({ minimum: 1, maximum: 500 })),
///      cursor:     Type.Optional(NonEmptyString),
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuditActivityListParamsSchema {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub agent_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub session_key: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub run_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub kind: Option<AuditActivityKindV1Schema>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status: Option<AuditActivityStatusV1Schema>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub direction: Option<AuditActivityDirectionV1Schema>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub channel: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub after: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub before: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub limit: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,
}

impl AuditActivityListParamsSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_optional_non_empty_string("agentId", self.agent_id.as_deref())?;
        validate_optional_non_empty_string("sessionKey", self.session_key.as_deref())?;
        validate_optional_non_empty_string("runId", self.run_id.as_deref())?;
        validate_optional_non_empty_string("channel", self.channel.as_deref())?;
        validate_optional_non_negative_integer("after", self.after)?;
        validate_optional_non_negative_integer("before", self.before)?;
        if let Some(n) = self.limit {
            validate_integer_in_range("limit", n, 1, AUDIT_ACTIVITY_LIST_LIMIT_MAX)?;
        }
        validate_optional_non_empty_string("cursor", self.cursor.as_deref())?;
        Ok(())
    }
}

// ---------- AuditActivityListResultSchema ----------

/// Stable sequence-cursor V1 activity page.
/// 对齐 TS:
///   `AuditActivityListResultSchema = Type.Object({
///      events:     Type.Array(AuditActivityEventV1Schema),
///      nextCursor: Type.Optional(NonEmptyString),
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuditActivityListResultSchema {
    pub events: Vec<AuditActivityEventV1Schema>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub next_cursor: Option<String>,
}

impl AuditActivityListResultSchema {
    pub fn validate(&self) -> Result<(), String> {
        for (i, ev) in self.events.iter().enumerate() {
            ev.validate()
                .map_err(|e| format!("events[{}]: {}", i, e))?;
        }
        validate_optional_non_empty_string("nextCursor", self.next_cursor.as_deref())?;
        Ok(())
    }
}

// ============================================================================
// Wire types (对应 TS 文件末尾的 `type X = ...` 集合)
// ============================================================================

// 对应 TS:
//   export type AuditActivityAgentRunV1 = AuditActivityAgentRunV1Schema &
//     AuditActivityAgentRecordBaseV1 & AuditActivityAgentRunV1Terminal;
//   export type AuditActivityToolActionV1 = AuditActivityToolActionV1Schema &
//     AuditActivityAgentRecordBaseV1 & AuditActivityToolActionV1Terminal;
//   export type AuditActivityInboundMessageV1 = AuditActivityInboundMessageV1Schema &
//     AuditActivityMessageRecordBaseV1 & AuditActivityInboundMessageV1Terminal;
//   export type AuditActivityOutboundMessageV1 = AuditActivityOutboundMessageV1Schema &
//     AuditActivityMessageRecordBaseV1 & AuditActivityOutboundMessageV1Terminal;
//   export type AuditActivityEventV1 = AuditActivityAgentRunV1
//     | AuditActivityToolActionV1 | AuditActivityInboundMessageV1
//     | AuditActivityOutboundMessageV1;
//   export type AuditActivityListParams = { ... };
//   export type AuditActivityListResult = { ... };
//
// Rust 端 schema struct 已经显式列出所有 required/optional 字段, 验证函数覆盖了
// `allOf` 终端不变量 (action+status+errorCode 等), 因此 wire type alias 直接绑到
// 对应的 schema 类型即可, 不再引入额外的复合类型.
pub type AuditActivityAgentRunV1 = AuditActivityAgentRunV1Schema;
pub type AuditActivityToolActionV1 = AuditActivityToolActionV1Schema;
pub type AuditActivityInboundMessageV1 = AuditActivityInboundMessageV1Schema;
pub type AuditActivityOutboundMessageV1 = AuditActivityOutboundMessageV1Schema;
pub type AuditActivityEventV1 = AuditActivityEventV1Schema;
pub type AuditActivityListParams = AuditActivityListParamsSchema;
pub type AuditActivityListResult = AuditActivityListResultSchema;