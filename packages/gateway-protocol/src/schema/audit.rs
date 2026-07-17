// Gateway Protocol schema module defines metadata-only audit query payloads.
// 翻译自 packages/gateway-protocol/src/schema/audit.ts
//
// TS 用 TypeBox 定义 schema（运行时验证 + 类型）。
// Rust 用 serde struct + 验证函数实现等价的序列化/反序列化语义。

use serde::{Deserialize, Serialize};

// ---------- 基础验证原语 (对齐 TypeBox: NonEmptyString / Integer{min}) ----------

/// 对齐 TS: `NonEmptyString = Type.String({ minLength: 1 })`
fn is_non_empty_string(s: &str) -> bool {
    !s.is_empty()
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

/// 对齐 TS: `Type.Integer({ minimum: 0 })`
fn validate_non_negative_integer(n: i64) -> Result<(), String> {
    if n >= 0 {
        Ok(())
    } else {
        Err(format!("expected integer >= 0, got {}", n))
    }
}

/// 对齐 TS: `Type.Integer({ minimum: 1 })`
fn validate_positive_integer(n: i64) -> Result<(), String> {
    if n >= 1 {
        Ok(())
    } else {
        Err(format!("expected integer >= 1, got {}", n))
    }
}

// ---------- AuditEventKindSchema ----------

/// Audit event kind discriminator.
/// 对齐 TS: `Type.Union([Type.Literal("agent_run"), Type.Literal("tool_action")])`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AuditEventKindSchema {
    #[serde(rename = "agent_run")]
    AgentRun,
    #[serde(rename = "tool_action")]
    ToolAction,
}

impl AuditEventKindSchema {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::AgentRun => "agent_run",
            Self::ToolAction => "tool_action",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "agent_run" => Some(Self::AgentRun),
            "tool_action" => Some(Self::ToolAction),
            _ => None,
        }
    }

    pub fn all() -> &'static [AuditEventKindSchema] {
        &[Self::AgentRun, Self::ToolAction]
    }
}

pub fn is_valid_audit_event_kind(kind: &str) -> bool {
    AuditEventKindSchema::from_str(kind).is_some()
}

// ---------- AuditEventActionSchema ----------

/// Audit event action discriminator.
/// 对齐 TS:
///   `Type.Union([Type.Literal("agent.run.started"), Type.Literal("agent.run.finished"),
///                Type.Literal("tool.action.started"), Type.Literal("tool.action.finished")])`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AuditEventActionSchema {
    #[serde(rename = "agent.run.started")]
    AgentRunStarted,
    #[serde(rename = "agent.run.finished")]
    AgentRunFinished,
    #[serde(rename = "tool.action.started")]
    ToolActionStarted,
    #[serde(rename = "tool.action.finished")]
    ToolActionFinished,
}

impl AuditEventActionSchema {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::AgentRunStarted => "agent.run.started",
            Self::AgentRunFinished => "agent.run.finished",
            Self::ToolActionStarted => "tool.action.started",
            Self::ToolActionFinished => "tool.action.finished",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "agent.run.started" => Some(Self::AgentRunStarted),
            "agent.run.finished" => Some(Self::AgentRunFinished),
            "tool.action.started" => Some(Self::ToolActionStarted),
            "tool.action.finished" => Some(Self::ToolActionFinished),
            _ => None,
        }
    }

    pub fn all() -> &'static [AuditEventActionSchema] {
        &[
            Self::AgentRunStarted,
            Self::AgentRunFinished,
            Self::ToolActionStarted,
            Self::ToolActionFinished,
        ]
    }
}

pub fn is_valid_audit_event_action(action: &str) -> bool {
    AuditEventActionSchema::from_str(action).is_some()
}

// ---------- AuditEventStatusSchema ----------

/// Audit event terminal/intermediate status.
/// 对齐 TS:
///   `Type.Union([Type.Literal("started"), Type.Literal("succeeded"),
///                Type.Literal("failed"), Type.Literal("cancelled"),
///                Type.Literal("timed_out"), Type.Literal("blocked"),
///                Type.Literal("unknown")])`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AuditEventStatusSchema {
    #[serde(rename = "started")]
    Started,
    #[serde(rename = "succeeded")]
    Succeeded,
    #[serde(rename = "failed")]
    Failed,
    #[serde(rename = "cancelled")]
    Cancelled,
    #[serde(rename = "timed_out")]
    TimedOut,
    #[serde(rename = "blocked")]
    Blocked,
    #[serde(rename = "unknown")]
    Unknown,
}

impl AuditEventStatusSchema {
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

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "started" => Some(Self::Started),
            "succeeded" => Some(Self::Succeeded),
            "failed" => Some(Self::Failed),
            "cancelled" => Some(Self::Cancelled),
            "timed_out" => Some(Self::TimedOut),
            "blocked" => Some(Self::Blocked),
            "unknown" => Some(Self::Unknown),
            _ => None,
        }
    }

    pub fn all() -> &'static [AuditEventStatusSchema] {
        &[
            Self::Started,
            Self::Succeeded,
            Self::Failed,
            Self::Cancelled,
            Self::TimedOut,
            Self::Blocked,
            Self::Unknown,
        ]
    }
}

pub fn is_valid_audit_event_status(status: &str) -> bool {
    AuditEventStatusSchema::from_str(status).is_some()
}

// ---------- AuditEventErrorCodeSchema ----------

/// Audit event error code discriminator.
/// 对齐 TS:
///   `Type.Union([Type.Literal("run_failed"), Type.Literal("run_cancelled"),
///                Type.Literal("run_timed_out"), Type.Literal("run_blocked"),
///                Type.Literal("tool_failed"), Type.Literal("tool_cancelled"),
///                Type.Literal("tool_timed_out"), Type.Literal("tool_blocked"),
///                Type.Literal("tool_outcome_unknown")])`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AuditEventErrorCodeSchema {
    #[serde(rename = "run_failed")]
    RunFailed,
    #[serde(rename = "run_cancelled")]
    RunCancelled,
    #[serde(rename = "run_timed_out")]
    RunTimedOut,
    #[serde(rename = "run_blocked")]
    RunBlocked,
    #[serde(rename = "tool_failed")]
    ToolFailed,
    #[serde(rename = "tool_cancelled")]
    ToolCancelled,
    #[serde(rename = "tool_timed_out")]
    ToolTimedOut,
    #[serde(rename = "tool_blocked")]
    ToolBlocked,
    #[serde(rename = "tool_outcome_unknown")]
    ToolOutcomeUnknown,
}

impl AuditEventErrorCodeSchema {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::RunFailed => "run_failed",
            Self::RunCancelled => "run_cancelled",
            Self::RunTimedOut => "run_timed_out",
            Self::RunBlocked => "run_blocked",
            Self::ToolFailed => "tool_failed",
            Self::ToolCancelled => "tool_cancelled",
            Self::ToolTimedOut => "tool_timed_out",
            Self::ToolBlocked => "tool_blocked",
            Self::ToolOutcomeUnknown => "tool_outcome_unknown",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "run_failed" => Some(Self::RunFailed),
            "run_cancelled" => Some(Self::RunCancelled),
            "run_timed_out" => Some(Self::RunTimedOut),
            "run_blocked" => Some(Self::RunBlocked),
            "tool_failed" => Some(Self::ToolFailed),
            "tool_cancelled" => Some(Self::ToolCancelled),
            "tool_timed_out" => Some(Self::ToolTimedOut),
            "tool_blocked" => Some(Self::ToolBlocked),
            "tool_outcome_unknown" => Some(Self::ToolOutcomeUnknown),
            _ => None,
        }
    }

    pub fn all() -> &'static [AuditEventErrorCodeSchema] {
        &[
            Self::RunFailed,
            Self::RunCancelled,
            Self::RunTimedOut,
            Self::RunBlocked,
            Self::ToolFailed,
            Self::ToolCancelled,
            Self::ToolTimedOut,
            Self::ToolBlocked,
            Self::ToolOutcomeUnknown,
        ]
    }
}

pub fn is_valid_audit_event_error_code(code: &str) -> bool {
    AuditEventErrorCodeSchema::from_str(code).is_some()
}

// ---------- AuditEventActorSchema ----------

/// Audit actor type discriminator.
/// 对齐 TS: `Type.Union([Type.Literal("agent"), Type.Literal("system")])`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AuditEventActorTypeSchema {
    #[serde(rename = "agent")]
    Agent,
    #[serde(rename = "system")]
    System,
}

impl AuditEventActorTypeSchema {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Agent => "agent",
            Self::System => "system",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "agent" => Some(Self::Agent),
            "system" => Some(Self::System),
            _ => None,
        }
    }

    pub fn all() -> &'static [AuditEventActorTypeSchema] {
        &[Self::Agent, Self::System]
    }
}

/// Audit event actor descriptor.
/// 对齐 TS: `Type.Object({ type: ..., id: NonEmptyString }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEventActorSchema {
    #[serde(rename = "type")]
    pub actor_type: AuditEventActorTypeSchema,
    pub id: String,
}

impl AuditEventActorSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_non_empty_string(&self.id)?;
        Ok(())
    }
}

// ---------- AuditEventRedactionSchema ----------

/// Audit event redaction policy literal. Currently fixed to `metadata_only`.
/// 对齐 TS: `Type.Literal("metadata_only")`.
pub const AUDIT_EVENT_REDACTION_METADATA_ONLY: &str = "metadata_only";

// ---------- AuditEventSchema ----------

/// One content-free run/tool audit record.
/// 对齐 TS: `AuditEventSchema = Type.Object({ ... }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuditEventSchema {
    pub event_id: String,
    pub sequence: i64,
    pub source_sequence: i64,
    pub occurred_at: i64,
    pub kind: AuditEventKindSchema,
    pub action: AuditEventActionSchema,
    pub status: AuditEventStatusSchema,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error_code: Option<AuditEventErrorCodeSchema>,
    pub actor: AuditEventActorSchema,
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
    pub redaction: String,
}

impl AuditEventSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_non_empty_string(&self.event_id)?;
        validate_positive_integer(self.sequence)?;
        validate_positive_integer(self.source_sequence)?;
        validate_non_negative_integer(self.occurred_at)?;
        self.actor.validate().map_err(|e| format!("actor: {}", e))?;
        validate_non_empty_string(&self.agent_id)?;
        validate_optional_non_empty_string(self.session_key.as_deref())?;
        validate_optional_non_empty_string(self.session_id.as_deref())?;
        validate_non_empty_string(&self.run_id)?;
        validate_optional_non_empty_string(self.tool_call_id.as_deref())?;
        validate_optional_non_empty_string(self.tool_name.as_deref())?;
        if self.redaction != AUDIT_EVENT_REDACTION_METADATA_ONLY {
            return Err(format!(
                "expected redaction {:?}, got {:?}",
                AUDIT_EVENT_REDACTION_METADATA_ONLY, self.redaction
            ));
        }
        Ok(())
    }
}

// ---------- AuditListParamsSchema ----------

/// Bounded newest-first audit query filters.
/// 对齐 TS:
///   `AuditListParamsSchema = Type.Object({ ... }, { additionalProperties: false })`
///   with `limit: Type.Integer({ minimum: 1, maximum: 500 })`.
pub const AUDIT_LIST_LIMIT_MAX: i64 = 500;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuditListParamsSchema {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub agent_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub session_key: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub run_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub kind: Option<AuditEventKindSchema>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status: Option<AuditEventStatusSchema>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub after: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub before: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub limit: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,
}

impl AuditListParamsSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_optional_non_empty_string(self.agent_id.as_deref())?;
        validate_optional_non_empty_string(self.session_key.as_deref())?;
        validate_optional_non_empty_string(self.run_id.as_deref())?;
        if let Some(n) = self.after {
            validate_non_negative_integer(n).map_err(|e| format!("after: {}", e))?;
        }
        if let Some(n) = self.before {
            validate_non_negative_integer(n).map_err(|e| format!("before: {}", e))?;
        }
        if let Some(n) = self.limit {
            if !(1..=AUDIT_LIST_LIMIT_MAX).contains(&n) {
                return Err(format!(
                    "limit must be between 1 and {}, got {}",
                    AUDIT_LIST_LIMIT_MAX, n
                ));
            }
        }
        validate_optional_non_empty_string(self.cursor.as_deref())?;
        Ok(())
    }
}

// ---------- AuditListResultSchema ----------

/// Stable sequence-cursor page suitable for bounded JSON export.
/// 对齐 TS:
///   `AuditListResultSchema = Type.Object({
///       events: Type.Array(AuditEventSchema),
///       nextCursor: Type.Optional(NonEmptyString),
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuditListResultSchema {
    pub events: Vec<AuditEventSchema>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub next_cursor: Option<String>,
}

impl AuditListResultSchema {
    pub fn validate(&self) -> Result<(), String> {
        for (i, ev) in self.events.iter().enumerate() {
            ev.validate().map_err(|e| format!("events[{}]: {}", i, e))?;
        }
        validate_optional_non_empty_string(self.next_cursor.as_deref())?;
        Ok(())
    }
}

// Wire types derive directly from local schema consts so public d.ts graphs never
// pull in the ProtocolSchemas registry.
// 对应 TS:
//   export type AuditEvent = Static<typeof AuditEventSchema>;
//   export type AuditListParams = Static<typeof AuditListParamsSchema>;
//   export type AuditListResult = Static<typeof AuditListResultSchema>;
pub type AuditEvent = AuditEventSchema;
pub type AuditListParams = AuditListParamsSchema;
pub type AuditListResult = AuditListResultSchema;
