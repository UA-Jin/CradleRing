// Gateway Protocol schema: tasks.
// 翻译自 packages/gateway-protocol/src/schema/tasks.ts
//
// Task ledger protocol schemas.
//
// Tasks represent long-running SDK/agent operations exposed through the gateway;
// these schemas keep list/get/cancel payloads bounded and status values closed.
//
// TS 用 TypeBox 定义 schema（运行时验证 + 类型）。
// Rust 用 serde struct + 验证函数实现等价的序列化/反序列化语义。

use serde::{Deserialize, Serialize};

use super::primitives::NonEmptyString;

// ---------- 基础验证原语 (对齐 TypeBox: NonEmptyString / Integer) ----------

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

/// 对齐 TS: `Type.Integer({ minimum: 0 })`
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

// ---------- TaskLedgerStatusSchema ----------

/// Closed task lifecycle statuses visible in the gateway task ledger.
/// 对齐 TS:
///   `Type.Union([
///      Type.Literal("queued"),
///      Type.Literal("running"),
///      Type.Literal("completed"),
///      Type.Literal("failed"),
///      Type.Literal("cancelled"),
///      Type.Literal("timed_out"),
///   ])`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskLedgerStatusSchema {
    Queued,
    Running,
    Completed,
    Failed,
    Cancelled,
    #[serde(rename = "timed_out")]
    TimedOut,
}

impl TaskLedgerStatusSchema {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Queued => "queued",
            Self::Running => "running",
            Self::Completed => "completed",
            Self::Failed => "failed",
            Self::Cancelled => "cancelled",
            Self::TimedOut => "timed_out",
        }
    }
}

pub fn is_valid_task_ledger_status(s: &str) -> bool {
    matches!(
        s,
        "queued" | "running" | "completed" | "failed" | "cancelled" | "timed_out"
    )
}

// ---------- TimestampSchema (module-private) ----------

/// 对齐 TS: `Type.Union([Type.String(), Type.Integer({ minimum: 0 })])`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum TimestampSchema {
    String(String),
    Integer(i64),
}

impl TimestampSchema {
    pub fn validate(&self) -> Result<(), String> {
        match self {
            TimestampSchema::String(_) => Ok(()),
            TimestampSchema::Integer(n) => validate_non_negative_integer("timestamp", *n),
        }
    }
}

// ---------- TaskSummarySchema ----------

/// Public task summary returned by task list/get/cancel responses.
/// 对齐 TS:
///   `Type.Object({
///      id: NonEmptyString,
///      kind: Type.Optional(Type.String()),
///      runtime: Type.Optional(Type.String()),
///      status: TaskLedgerStatusSchema,
///      title: Type.Optional(Type.String()),
///      agentId: Type.Optional(Type.String()),
///      sessionKey: Type.Optional(Type.String()),
///      childSessionKey: Type.Optional(Type.String()),
///      ownerKey: Type.Optional(Type.String()),
///      runId: Type.Optional(Type.String()),
///      taskId: Type.Optional(Type.String()),
///      flowId: Type.Optional(Type.String()),
///      parentTaskId: Type.Optional(Type.String()),
///      sourceId: Type.Optional(Type.String()),
///      createdAt: Type.Optional(TimestampSchema),
///      updatedAt: Type.Optional(TimestampSchema),
///      startedAt: Type.Optional(TimestampSchema),
///      endedAt: Type.Optional(TimestampSchema),
///      toolUseCount: Type.Optional(Type.Integer({ minimum: 0 })),
///      lastToolName: Type.Optional(Type.String()),
///      progressSummary: Type.Optional(Type.String()),
///      terminalSummary: Type.Optional(Type.String()),
///      error: Type.Optional(Type.String()),
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskSummarySchema {
    pub id: NonEmptyString,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub kind: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub runtime: Option<String>,
    pub status: TaskLedgerStatusSchema,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub agent_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub session_key: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub child_session_key: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub owner_key: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub run_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub task_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub flow_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent_task_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created_at: Option<TimestampSchema>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<TimestampSchema>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub started_at: Option<TimestampSchema>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ended_at: Option<TimestampSchema>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool_use_count: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_tool_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub progress_summary: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub terminal_summary: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl TaskSummarySchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_non_empty_string("id", &self.id)?;
        if let Some(n) = self.tool_use_count {
            validate_non_negative_integer("toolUseCount", n)?;
        }
        for (field, ts) in [
            ("createdAt", self.created_at.as_ref()),
            ("updatedAt", self.updated_at.as_ref()),
            ("startedAt", self.started_at.as_ref()),
            ("endedAt", self.ended_at.as_ref()),
        ] {
            if let Some(ts) = ts {
                ts.validate().map_err(|e| format!("{}: {}", field, e))?;
            }
        }
        Ok(())
    }
}

// ---------- TasksListParamsSchema ----------

/// Pagination bounds for task list responses.
pub const TASKS_LIST_LIMIT_MIN: i64 = 1;
pub const TASKS_LIST_LIMIT_MAX: i64 = 500;

/// `status` filter accepts either a single status or an array of statuses.
/// 对齐 TS: `Type.Optional(Type.Union([TaskLedgerStatusSchema, Type.Array(TaskLedgerStatusSchema)]))`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum TasksListStatusFilter {
    Single(TaskLedgerStatusSchema),
    Many(Vec<TaskLedgerStatusSchema>),
}

impl TasksListStatusFilter {
    pub fn validate(&self) -> Result<(), String> {
        // Per-element validation happens at deserialize time via serde enum.
        Ok(())
    }
}

/// Task list filters with bounded pagination.
/// 对齐 TS:
///   `Type.Object({
///      status: Type.Optional(Type.Union([TaskLedgerStatusSchema, Type.Array(TaskLedgerStatusSchema)])),
///      agentId: Type.Optional(NonEmptyString),
///      sessionKey: Type.Optional(NonEmptyString),
///      limit: Type.Optional(Type.Integer({ minimum: 1, maximum: 500 })),
///      cursor: Type.Optional(Type.String()),
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TasksListParamsSchema {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status: Option<TasksListStatusFilter>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub agent_id: Option<NonEmptyString>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub session_key: Option<NonEmptyString>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub limit: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,
}

impl TasksListParamsSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_optional_non_empty_string("agentId", self.agent_id.as_deref())?;
        validate_optional_non_empty_string("sessionKey", self.session_key.as_deref())?;
        if let Some(limit) = self.limit {
            if !(TASKS_LIST_LIMIT_MIN..=TASKS_LIST_LIMIT_MAX).contains(&limit) {
                return Err(format!(
                    "limit: expected {}..={}, got {}",
                    TASKS_LIST_LIMIT_MIN, TASKS_LIST_LIMIT_MAX, limit
                ));
            }
        }
        if let Some(status) = &self.status {
            status.validate()?;
        }
        Ok(())
    }
}

// ---------- TasksListResultSchema ----------

/// Task list page response.
/// 对齐 TS:
///   `Type.Object({
///      tasks: Type.Array(TaskSummarySchema),
///      nextCursor: Type.Optional(Type.String()),
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TasksListResultSchema {
    pub tasks: Vec<TaskSummarySchema>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub next_cursor: Option<String>,
}

impl TasksListResultSchema {
    pub fn validate(&self) -> Result<(), String> {
        for (i, task) in self.tasks.iter().enumerate() {
            task.validate()
                .map_err(|e| format!("tasks[{}]: {}", i, e))?;
        }
        Ok(())
    }
}

// ---------- TasksGetParamsSchema ----------

/// Lookup request for one task id.
/// 对齐 TS:
///   `Type.Object({
///      taskId: NonEmptyString,
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TasksGetParamsSchema {
    pub task_id: NonEmptyString,
}

impl TasksGetParamsSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_non_empty_string("taskId", &self.task_id)?;
        Ok(())
    }
}

// ---------- TasksGetResultSchema ----------

/// Lookup result for one task summary.
/// 对齐 TS:
///   `Type.Object({
///      task: TaskSummarySchema,
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TasksGetResultSchema {
    pub task: TaskSummarySchema,
}

impl TasksGetResultSchema {
    pub fn validate(&self) -> Result<(), String> {
        self.task.validate()
    }
}

// ---------- TasksCancelParamsSchema ----------

/// Cancel request for one task id with optional operator reason.
/// 对齐 TS:
///   `Type.Object({
///      taskId: NonEmptyString,
///      reason: Type.Optional(Type.String()),
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TasksCancelParamsSchema {
    pub task_id: NonEmptyString,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

impl TasksCancelParamsSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_non_empty_string("taskId", &self.task_id)?;
        Ok(())
    }
}

// ---------- TasksCancelResultSchema ----------

/// Cancel result, including the task snapshot when it was found.
/// 对齐 TS:
///   `Type.Object({
///      found: Type.Boolean(),
///      cancelled: Type.Boolean(),
///      reason: Type.Optional(Type.String()),
///      task: Type.Optional(TaskSummarySchema),
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TasksCancelResultSchema {
    pub found: bool,
    pub cancelled: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub task: Option<TaskSummarySchema>,
}

impl TasksCancelResultSchema {
    pub fn validate(&self) -> Result<(), String> {
        if let Some(task) = &self.task {
            task.validate().map_err(|e| format!("task: {}", e))?;
        }
        Ok(())
    }
}

// ============================================================
// Wire types
// 对应 TS:
//   export type TaskSummary = Static<typeof TaskSummarySchema>;
//   export type TasksListParams = Static<typeof TasksListParamsSchema>;
//   export type TasksListResult = Static<typeof TasksListResultSchema>;
//   export type TasksGetParams = Static<typeof TasksGetParamsSchema>;
//   export type TasksGetResult = Static<typeof TasksGetResultSchema>;
//   export type TasksCancelParams = Static<typeof TasksCancelParamsSchema>;
//   export type TasksCancelResult = Static<typeof TasksCancelResultSchema>;
// Wire types derive directly from local schema consts so public d.ts graphs never
// pull in the ProtocolSchemas registry.
// ============================================================

pub type TaskSummary = TaskSummarySchema;
pub type TasksListParams = TasksListParamsSchema;
pub type TasksListResult = TasksListResultSchema;
pub type TasksGetParams = TasksGetParamsSchema;
pub type TasksGetResult = TasksGetResultSchema;
pub type TasksCancelParams = TasksCancelParamsSchema;
pub type TasksCancelResult = TasksCancelResultSchema;