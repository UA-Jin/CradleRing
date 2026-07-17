// Gateway Protocol schema: gateway_suspend.
// 翻译自 packages/gateway-protocol/src/schema/gateway-suspend.ts
//
// Gateway Protocol schemas for cooperative host suspension.
//
// TS 用 TypeBox 定义 schema（运行时验证 + 类型）。
// Rust 用 serde struct + 验证函数实现等价的序列化/反序列化语义。

use serde::{Deserialize, Serialize};

use super::primitives::NonEmptyString;

// ---------- Length / pattern constants ----------

/// Maximum length of a suspension token (requestId / suspensionId).
/// 对齐 TS: `SuspensionTokenSchema = Type.String({ minLength: 1, maxLength: 128, pattern: "\\S" })`.
pub const SUSPENSION_TOKEN_MAX_LENGTH: usize = 128;
/// Minimum length of a suspension token.
pub const SUSPENSION_TOKEN_MIN_LENGTH: usize = 1;

// ---------- Validation primitives ----------

/// 对齐 TS: `Type.Integer({ minimum: 0 })`.
fn validate_non_negative_integer(field: &str, n: i64) -> Result<(), String> {
    if n >= 0 {
        Ok(())
    } else {
        Err(format!("{}: expected integer >= 0, got {}", field, n))
    }
}

/// 对齐 TS: `SuspensionTokenSchema`: 1..=128 chars and no whitespace allowed.
fn validate_suspension_token(field: &str, value: &str) -> Result<(), String> {
    let len = value.chars().count();
    if len < SUSPENSION_TOKEN_MIN_LENGTH || len > SUSPENSION_TOKEN_MAX_LENGTH {
        return Err(format!(
            "{}: expected length in [{}, {}], got {}",
            field, SUSPENSION_TOKEN_MIN_LENGTH, SUSPENSION_TOKEN_MAX_LENGTH, len
        ));
    }
    if value.chars().any(|c| c.is_whitespace()) {
        return Err(format!(
            "{}: expected non-whitespace token, got {:?}",
            field, value
        ));
    }
    Ok(())
}

// ---------- Closed enums ----------

/// Runtime kind carried by a suspend blocker task payload.
/// 对齐 TS:
///   `runtime: Type.Union([
///       Type.Literal("subagent"),
///       Type.Literal("acp"),
///       Type.Literal("cli"),
///       Type.Literal("cron"),
///   ])`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GatewaySuspendTaskRuntimeSchema {
    #[serde(rename = "subagent")]
    Subagent,
    #[serde(rename = "acp")]
    Acp,
    #[serde(rename = "cli")]
    Cli,
    #[serde(rename = "cron")]
    Cron,
}

impl GatewaySuspendTaskRuntimeSchema {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Subagent => "subagent",
            Self::Acp => "acp",
            Self::Cli => "cli",
            Self::Cron => "cron",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "subagent" => Some(Self::Subagent),
            "acp" => Some(Self::Acp),
            "cli" => Some(Self::Cli),
            "cron" => Some(Self::Cron),
            _ => None,
        }
    }

    pub fn all() -> &'static [GatewaySuspendTaskRuntimeSchema] {
        &[Self::Subagent, Self::Acp, Self::Cli, Self::Cron]
    }
}

pub fn is_valid_gateway_suspend_task_runtime(s: &str) -> bool {
    GatewaySuspendTaskRuntimeSchema::from_str(s).is_some()
}

/// Closed enumeration of blocker categories reported in `GatewaySuspendBlockerSchema`.
/// 对齐 TS:
///   `kind: Type.Union([
///       Type.Literal("queue"),
///       Type.Literal("reply"),
///       Type.Literal("embedded-run"),
///       Type.Literal("background-exec"),
///       Type.Literal("cron-run"),
///       Type.Literal("task"),
///       Type.Literal("root-request"),
///       Type.Literal("session-admission"),
///       Type.Literal("session-mutation"),
///       Type.Literal("chat-run"),
///       Type.Literal("queued-turn"),
///       Type.Literal("terminal-persistence"),
///       Type.Literal("terminal-session"),
///   ])`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum GatewaySuspendBlockerKindSchema {
    Queue,
    Reply,
    EmbeddedRun,
    BackgroundExec,
    CronRun,
    Task,
    RootRequest,
    SessionAdmission,
    SessionMutation,
    ChatRun,
    QueuedTurn,
    TerminalPersistence,
    TerminalSession,
}

impl GatewaySuspendBlockerKindSchema {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Queue => "queue",
            Self::Reply => "reply",
            Self::EmbeddedRun => "embedded-run",
            Self::BackgroundExec => "background-exec",
            Self::CronRun => "cron-run",
            Self::Task => "task",
            Self::RootRequest => "root-request",
            Self::SessionAdmission => "session-admission",
            Self::SessionMutation => "session-mutation",
            Self::ChatRun => "chat-run",
            Self::QueuedTurn => "queued-turn",
            Self::TerminalPersistence => "terminal-persistence",
            Self::TerminalSession => "terminal-session",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "queue" => Some(Self::Queue),
            "reply" => Some(Self::Reply),
            "embedded-run" => Some(Self::EmbeddedRun),
            "background-exec" => Some(Self::BackgroundExec),
            "cron-run" => Some(Self::CronRun),
            "task" => Some(Self::Task),
            "root-request" => Some(Self::RootRequest),
            "session-admission" => Some(Self::SessionAdmission),
            "session-mutation" => Some(Self::SessionMutation),
            "chat-run" => Some(Self::ChatRun),
            "queued-turn" => Some(Self::QueuedTurn),
            "terminal-persistence" => Some(Self::TerminalPersistence),
            "terminal-session" => Some(Self::TerminalSession),
            _ => None,
        }
    }

    pub fn all() -> &'static [GatewaySuspendBlockerKindSchema] {
        &[
            Self::Queue,
            Self::Reply,
            Self::EmbeddedRun,
            Self::BackgroundExec,
            Self::CronRun,
            Self::Task,
            Self::RootRequest,
            Self::SessionAdmission,
            Self::SessionMutation,
            Self::ChatRun,
            Self::QueuedTurn,
            Self::TerminalPersistence,
            Self::TerminalSession,
        ]
    }
}

pub fn is_valid_gateway_suspend_blocker_kind(s: &str) -> bool {
    GatewaySuspendBlockerKindSchema::from_str(s).is_some()
}

/// Reason discriminator for the busy prepare-result variant.
/// 对齐 TS:
///   `reason: Type.Union([Type.Literal("active-work"), Type.Literal("gateway-draining")])`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum GatewaySuspendBusyReasonSchema {
    ActiveWork,
    GatewayDraining,
}

impl GatewaySuspendBusyReasonSchema {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::ActiveWork => "active-work",
            Self::GatewayDraining => "gateway-draining",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "active-work" => Some(Self::ActiveWork),
            "gateway-draining" => Some(Self::GatewayDraining),
            _ => None,
        }
    }
}

pub fn is_valid_gateway_suspend_busy_reason(s: &str) -> bool {
    GatewaySuspendBusyReasonSchema::from_str(s).is_some()
}

// ---------- GatewaySuspendTaskBlockerSchema ----------

/// Task-shaped blocker carried by `GatewaySuspendBlockerSchema.task`.
/// 对齐 TS:
///   `Type.Object({
///       taskId: Type.String(),
///       status: Type.Literal("running"),
///       runtime: Type.Union([Type.Literal("subagent"), Type.Literal("acp"),
///                            Type.Literal("cli"), Type.Literal("cron")]),
///       runId:  Type.Optional(Type.String()),
///       label:  Type.Optional(Type.String()),
///       title:  Type.Optional(Type.String()),
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GatewaySuspendTaskBlockerSchema {
    pub task_id: String,
    pub status: GatewaySuspendTaskRunningStatus,
    pub runtime: GatewaySuspendTaskRuntimeSchema,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub run_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
}

/// Literal "running" marker for `GatewaySuspendTaskBlockerSchema.status`.
/// 对齐 TS: `status: Type.Literal("running")`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GatewaySuspendTaskRunningStatus {
    #[serde(rename = "running")]
    Running,
}

impl Default for GatewaySuspendTaskRunningStatus {
    fn default() -> Self {
        Self::Running
    }
}

impl GatewaySuspendTaskBlockerSchema {
    pub fn validate(&self) -> Result<(), String> {
        if self.task_id.is_empty() {
            return Err(format!("taskId: expected non-empty string, got {:?}", self.task_id));
        }
        // `runtime` is a closed enum; serde rejects unknown values at deserialization.
        if GatewaySuspendTaskRuntimeSchema::from_str(self.runtime.as_str()).is_none() {
            return Err(format!("runtime: invalid value: {}", self.runtime.as_str()));
        }
        Ok(())
    }
}

// ---------- GatewaySuspendBlockerSchema ----------

/// One blocker preventing gateway suspension.
/// 对齐 TS:
///   `Type.Object({
///       kind:    Type.Union([...] /* 13 literals */),
///       count:   CountSchema,
///       message: Type.String(),
///       task:    Type.Optional(GatewaySuspendTaskBlockerSchema),
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GatewaySuspendBlockerSchema {
    pub kind: GatewaySuspendBlockerKindSchema,
    pub count: i64,
    pub message: NonEmptyString,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub task: Option<GatewaySuspendTaskBlockerSchema>,
}

impl GatewaySuspendBlockerSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_non_negative_integer("count", self.count)?;
        if self.message.is_empty() {
            return Err(format!(
                "message: expected non-empty string, got {:?}",
                self.message
            ));
        }
        if let Some(task) = &self.task {
            task.validate().map_err(|e| format!("task: {}", e))?;
        }
        Ok(())
    }
}

// ---------- GatewaySuspendPrepareParamsSchema ----------

/// Parameters for `gateway.suspend.prepare`.
/// 对齐 TS:
///   `Type.Object({ requestId: SuspensionTokenSchema }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GatewaySuspendPrepareParamsSchema {
    pub request_id: String,
}

impl GatewaySuspendPrepareParamsSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_suspension_token("requestId", &self.request_id)?;
        Ok(())
    }
}

// ---------- GatewaySuspendPrepareBusyResultSchema ----------

/// Busy variant returned by `gateway.suspend.prepare`.
/// 对齐 TS:
///   `Type.Object({
///       status:      Type.Literal("busy"),
///       reason:      Type.Union([Type.Literal("active-work"), Type.Literal("gateway-draining")]),
///       retryAfterMs: CountSchema,
///       activeCount:  CountSchema,
///       blockers:    Type.Array(GatewaySuspendBlockerSchema),
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GatewaySuspendPrepareBusyResultSchema {
    pub status: GatewaySuspendBusyStatus,
    pub reason: GatewaySuspendBusyReasonSchema,
    pub retry_after_ms: i64,
    pub active_count: i64,
    pub blockers: Vec<GatewaySuspendBlockerSchema>,
}

/// Literal "busy" marker for `GatewaySuspendPrepareBusyResultSchema.status`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GatewaySuspendBusyStatus {
    #[serde(rename = "busy")]
    Busy,
}

impl Default for GatewaySuspendBusyStatus {
    fn default() -> Self {
        Self::Busy
    }
}

impl GatewaySuspendPrepareBusyResultSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_non_negative_integer("retryAfterMs", self.retry_after_ms)?;
        validate_non_negative_integer("activeCount", self.active_count)?;
        for (i, blocker) in self.blockers.iter().enumerate() {
            blocker
                .validate()
                .map_err(|e| format!("blockers[{}]: {}", i, e))?;
        }
        Ok(())
    }
}

// ---------- GatewaySuspendPrepareReadyResultSchema ----------

/// Ready variant returned by `gateway.suspend.prepare`.
/// 对齐 TS:
///   `Type.Object({
///       status:       Type.Literal("ready"),
///       suspensionId: SuspensionTokenSchema,
///       expiresAtMs:  CountSchema,
///       activeCount:  CountSchema,
///       blockers:     Type.Array(GatewaySuspendBlockerSchema),
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GatewaySuspendPrepareReadyResultSchema {
    pub status: GatewaySuspendPrepareReadyStatus,
    pub suspension_id: String,
    pub expires_at_ms: i64,
    pub active_count: i64,
    pub blockers: Vec<GatewaySuspendBlockerSchema>,
}

/// Literal "ready" marker for `GatewaySuspendPrepareReadyResultSchema.status`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GatewaySuspendPrepareReadyStatus {
    #[serde(rename = "ready")]
    Ready,
}

impl Default for GatewaySuspendPrepareReadyStatus {
    fn default() -> Self {
        Self::Ready
    }
}

impl GatewaySuspendPrepareReadyResultSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_suspension_token("suspensionId", &self.suspension_id)?;
        validate_non_negative_integer("expiresAtMs", self.expires_at_ms)?;
        validate_non_negative_integer("activeCount", self.active_count)?;
        for (i, blocker) in self.blockers.iter().enumerate() {
            blocker
                .validate()
                .map_err(|e| format!("blockers[{}]: {}", i, e))?;
        }
        Ok(())
    }
}

// ---------- GatewaySuspendPrepareResultSchema ----------

/// Discriminated union for `gateway.suspend.prepare` results (busy vs ready).
/// 对齐 TS:
///   `Type.Union([GatewaySuspendPrepareBusyResultSchema, GatewaySuspendPrepareReadyResultSchema])`.
/// TS `Type.Union` is untagged at the JSON-Schema level; each variant carries its
/// own `status` literal so serde can disambiguate via the inner `status` field.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum GatewaySuspendPrepareResultSchema {
    Busy(GatewaySuspendPrepareBusyResultSchema),
    Ready(GatewaySuspendPrepareReadyResultSchema),
}

impl GatewaySuspendPrepareResultSchema {
    pub fn validate(&self) -> Result<(), String> {
        match self {
            Self::Busy(b) => b.validate(),
            Self::Ready(r) => r.validate(),
        }
    }
}

// ---------- GatewaySuspendStatusParamsSchema ----------

/// Parameters for `gateway.suspend.status`.
/// 对齐 TS:
///   `Type.Object({ suspensionId: SuspensionTokenSchema }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GatewaySuspendStatusParamsSchema {
    pub suspension_id: String,
}

impl GatewaySuspendStatusParamsSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_suspension_token("suspensionId", &self.suspension_id)?;
        Ok(())
    }
}

// ---------- GatewaySuspendStatusRunningResultSchema / ReadyResultSchema ----------

/// Running variant returned by `gateway.suspend.status`.
/// 对齐 TS: `Type.Object({ status: Type.Literal("running") }, { additionalProperties: false })`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GatewaySuspendStatusRunningResultSchema {
    pub status: GatewaySuspendStatusRunningStatus,
}

/// Literal "running" marker for the running status-result variant.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GatewaySuspendStatusRunningStatus {
    #[serde(rename = "running")]
    Running,
}

impl Default for GatewaySuspendStatusRunningStatus {
    fn default() -> Self {
        Self::Running
    }
}

/// Ready variant returned by `gateway.suspend.status`.
/// 对齐 TS:
///   `Type.Object({ status: Type.Literal("ready"), expiresAtMs: CountSchema },
///                { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GatewaySuspendStatusReadyResultSchema {
    pub status: GatewaySuspendStatusReadyStatus,
    pub expires_at_ms: i64,
}

impl GatewaySuspendStatusReadyResultSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_non_negative_integer("expiresAtMs", self.expires_at_ms)?;
        Ok(())
    }
}

/// Literal "ready" marker for the ready status-result variant.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GatewaySuspendStatusReadyStatus {
    #[serde(rename = "ready")]
    Ready,
}

impl Default for GatewaySuspendStatusReadyStatus {
    fn default() -> Self {
        Self::Ready
    }
}

/// Discriminated union for `gateway.suspend.status` results.
/// 对齐 TS:
///   `Type.Union([GatewaySuspendStatusRunningResultSchema, GatewaySuspendStatusReadyResultSchema])`.
/// Untagged at the JSON-Schema level; each variant carries its own `status` literal.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum GatewaySuspendStatusResultSchema {
    Running(GatewaySuspendStatusRunningResultSchema),
    Ready(GatewaySuspendStatusReadyResultSchema),
}

impl GatewaySuspendStatusResultSchema {
    pub fn validate(&self) -> Result<(), String> {
        match self {
            Self::Running(_) => Ok(()),
            Self::Ready(r) => r.validate(),
        }
    }
}

// ---------- GatewaySuspendResumeParamsSchema / ResumeResultSchema ----------

/// Parameters for `gateway.suspend.resume`.
/// 对齐 TS: `GatewaySuspendResumeParamsSchema = GatewaySuspendStatusParamsSchema`.
pub type GatewaySuspendResumeParamsSchema = GatewaySuspendStatusParamsSchema;

/// Result returned by `gateway.suspend.resume`.
/// 对齐 TS:
///   `Type.Object({
///       ok:      Type.Literal(true),
///       status:  Type.Literal("running"),
///       resumed: Type.Boolean(),
///   }, { additionalProperties: false })`.
/// `ok` is a JSON boolean literal `true` (TypeBox `Type.Literal(true)`).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GatewaySuspendResumeResultSchema {
    pub ok: bool,
    pub status: GatewaySuspendResumeRunningStatus,
    pub resumed: bool,
}

/// Literal "running" marker for `GatewaySuspendResumeResultSchema.status`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GatewaySuspendResumeRunningStatus {
    #[serde(rename = "running")]
    Running,
}

impl Default for GatewaySuspendResumeRunningStatus {
    fn default() -> Self {
        Self::Running
    }
}

impl GatewaySuspendResumeResultSchema {
    pub fn validate(&self) -> Result<(), String> {
        if !self.ok {
            return Err(format!("ok: expected literal true, got {}", self.ok));
        }
        Ok(())
    }
}

// Wire types derive directly from local schema consts so public d.ts graphs never
// pull in the ProtocolSchemas registry.
// 对应 TS:
//   export type GatewaySuspendTaskBlocker = Static<typeof GatewaySuspendTaskBlockerSchema>;
//   export type GatewaySuspendBlocker    = Static<typeof GatewaySuspendBlockerSchema>;
//   export type GatewaySuspendPrepareParams = Static<typeof GatewaySuspendPrepareParamsSchema>;
//   export type GatewaySuspendPrepareResult = Static<typeof GatewaySuspendPrepareResultSchema>;
//   export type GatewaySuspendStatusParams  = Static<typeof GatewaySuspendStatusParamsSchema>;
//   export type GatewaySuspendStatusResult  = Static<typeof GatewaySuspendStatusResultSchema>;
//   export type GatewaySuspendResumeParams  = Static<typeof GatewaySuspendResumeParamsSchema>;
//   export type GatewaySuspendResumeResult  = Static<typeof GatewaySuspendResumeResultSchema>;
pub type GatewaySuspendTaskBlocker = GatewaySuspendTaskBlockerSchema;
pub type GatewaySuspendBlocker = GatewaySuspendBlockerSchema;
pub type GatewaySuspendPrepareParams = GatewaySuspendPrepareParamsSchema;
pub type GatewaySuspendPrepareResult = GatewaySuspendPrepareResultSchema;
pub type GatewaySuspendStatusParams = GatewaySuspendStatusParamsSchema;
pub type GatewaySuspendStatusResult = GatewaySuspendStatusResultSchema;
pub type GatewaySuspendResumeParams = GatewaySuspendResumeParamsSchema;
pub type GatewaySuspendResumeResult = GatewaySuspendResumeResultSchema;