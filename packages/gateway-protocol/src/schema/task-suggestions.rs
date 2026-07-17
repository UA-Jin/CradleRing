// Gateway Protocol schema module defines ephemeral follow-up task suggestions.
// 翻译自 packages/gateway-protocol/src/schema/task-suggestions.ts
//
// TS 用 TypeBox 定义 schema（运行时验证 + 类型）。
// Rust 用 serde struct + 验证函数实现等价的序列化/反序列化语义。

use serde::{Deserialize, Serialize};

// ---------- 长度常量 (对齐 TypeBox: maxLength) ----------

/// 对齐 TS: `TaskIdSchema = Type.String({ minLength: 1, maxLength: 128 })`
pub const TASK_ID_MAX_LENGTH: usize = 128;
/// 对齐 TS: `TaskTitleSchema = Type.String({ minLength: 1, maxLength: 60 })`
pub const TASK_TITLE_MAX_LENGTH: usize = 60;
/// 对齐 TS: `TaskPromptSchema = Type.String({ minLength: 1, maxLength: 32_768 })`
pub const TASK_PROMPT_MAX_LENGTH: usize = 32_768;
/// 对齐 TS: `TaskTldrSchema = Type.String({ minLength: 1, maxLength: 1_024 })`
pub const TASK_TLDR_MAX_LENGTH: usize = 1_024;
/// 对齐 TS: `TaskCwdSchema = Type.String({ minLength: 1, maxLength: 4_096 })`
pub const TASK_CWD_MAX_LENGTH: usize = 4_096;
/// 对齐 TS: `TaskSessionKeySchema = Type.String({ minLength: 1, maxLength: 512 })`
pub const TASK_SESSION_KEY_MAX_LENGTH: usize = 512;
/// 对齐 TS: `TaskAgentIdSchema = Type.String({ minLength: 1, maxLength: 128 })`
pub const TASK_AGENT_ID_MAX_LENGTH: usize = 128;
/// 对齐 TS: `Type.String({ maxLength: 1_024 })` (in dismiss reason)
pub const TASK_DISMISS_REASON_MAX_LENGTH: usize = 1_024;

// ---------- 基础验证原语 (对齐 TypeBox: NonEmptyString / Integer{min}) ----------

fn validate_bounded_string(
    field: &str,
    value: &str,
    min_len: usize,
    max_len: usize,
) -> Result<(), String> {
    let len = value.chars().count();
    if len < min_len || len > max_len {
        return Err(format!(
            "{}: expected length in [{}, {}], got {}",
            field, min_len, max_len, len
        ));
    }
    Ok(())
}

fn validate_optional_bounded_string(
    field: &str,
    value: Option<&str>,
    min_len: usize,
    max_len: usize,
) -> Result<(), String> {
    if let Some(s) = value {
        validate_bounded_string(field, s, min_len, max_len)?;
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

// ---------- TaskSuggestionResolutionSchema ----------

/// Resolution discriminator for a closed suggestion lifecycle.
/// 对齐 TS:
///   `TaskSuggestionResolutionSchema = Type.Union([
///       Type.Literal("dismissed"),
///       Type.Literal("accepted"),
///       Type.Literal("expired"),
///   ])`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TaskSuggestionResolutionSchema {
    #[serde(rename = "dismissed")]
    Dismissed,
    #[serde(rename = "accepted")]
    Accepted,
    #[serde(rename = "expired")]
    Expired,
}

impl TaskSuggestionResolutionSchema {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Dismissed => "dismissed",
            Self::Accepted => "accepted",
            Self::Expired => "expired",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "dismissed" => Some(Self::Dismissed),
            "accepted" => Some(Self::Accepted),
            "expired" => Some(Self::Expired),
            _ => None,
        }
    }

    pub fn all() -> &'static [TaskSuggestionResolutionSchema] {
        &[Self::Dismissed, Self::Accepted, Self::Expired]
    }
}

pub fn is_valid_task_suggestion_resolution(s: &str) -> bool {
    TaskSuggestionResolutionSchema::from_str(s).is_some()
}

// ---------- TaskSuggestionSchema ----------

/// One model-proposed follow-up task waiting for operator action.
/// 对齐 TS:
///   `TaskSuggestionSchema = Type.Object({
///       id: TaskIdSchema,
///       title: TaskTitleSchema,
///       prompt: TaskPromptSchema,
///       tldr: TaskTldrSchema,
///       cwd: TaskCwdSchema,
///       sessionKey: TaskSessionKeySchema,
///       agentId: Type.Optional(TaskAgentIdSchema),
///       createdAt: Type.Integer({ minimum: 0 }),
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskSuggestionSchema {
    pub id: String,
    pub title: String,
    pub prompt: String,
    pub tldr: String,
    pub cwd: String,
    pub session_key: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub agent_id: Option<String>,
    pub created_at: i64,
}

impl TaskSuggestionSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_bounded_string("id", &self.id, 1, TASK_ID_MAX_LENGTH)?;
        validate_bounded_string("title", &self.title, 1, TASK_TITLE_MAX_LENGTH)?;
        validate_bounded_string("prompt", &self.prompt, 1, TASK_PROMPT_MAX_LENGTH)?;
        validate_bounded_string("tldr", &self.tldr, 1, TASK_TLDR_MAX_LENGTH)?;
        validate_bounded_string("cwd", &self.cwd, 1, TASK_CWD_MAX_LENGTH)?;
        validate_bounded_string(
            "sessionKey",
            &self.session_key,
            1,
            TASK_SESSION_KEY_MAX_LENGTH,
        )?;
        validate_optional_bounded_string(
            "agentId",
            self.agent_id.as_deref(),
            1,
            TASK_AGENT_ID_MAX_LENGTH,
        )?;
        validate_non_negative_integer(self.created_at).map_err(|e| format!("createdAt: {}", e))?;
        Ok(())
    }
}

// ---------- TaskSuggestionsListParamsSchema ----------

/// Lists pending suggestions, optionally narrowed to one source session.
/// 对齐 TS:
///   `TaskSuggestionsListParamsSchema = Type.Object({
///       sessionKey: Type.Optional(TaskSessionKeySchema),
///       agentId: Type.Optional(TaskAgentIdSchema),
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskSuggestionsListParamsSchema {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub session_key: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub agent_id: Option<String>,
}

impl TaskSuggestionsListParamsSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_optional_bounded_string(
            "sessionKey",
            self.session_key.as_deref(),
            1,
            TASK_SESSION_KEY_MAX_LENGTH,
        )?;
        validate_optional_bounded_string(
            "agentId",
            self.agent_id.as_deref(),
            1,
            TASK_AGENT_ID_MAX_LENGTH,
        )?;
        Ok(())
    }
}

// ---------- TaskSuggestionsListResultSchema ----------

/// 对齐 TS:
///   `TaskSuggestionsListResultSchema = Type.Object({
///       suggestions: Type.Array(TaskSuggestionSchema),
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskSuggestionsListResultSchema {
    pub suggestions: Vec<TaskSuggestionSchema>,
}

impl TaskSuggestionsListResultSchema {
    pub fn validate(&self) -> Result<(), String> {
        for (i, s) in self.suggestions.iter().enumerate() {
            s.validate().map_err(|e| format!("suggestions[{}]: {}", i, e))?;
        }
        Ok(())
    }
}

// ---------- TaskSuggestionsCreateParamsSchema ----------

/// Creates a pending suggestion without starting any work.
/// 对齐 TS:
///   `TaskSuggestionsCreateParamsSchema = Type.Object({
///       title: TaskTitleSchema,
///       prompt: TaskPromptSchema,
///       tldr: TaskTldrSchema,
///       cwd: TaskCwdSchema,
///       sessionKey: TaskSessionKeySchema,
///       agentId: Type.Optional(TaskAgentIdSchema),
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskSuggestionsCreateParamsSchema {
    pub title: String,
    pub prompt: String,
    pub tldr: String,
    pub cwd: String,
    pub session_key: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub agent_id: Option<String>,
}

impl TaskSuggestionsCreateParamsSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_bounded_string("title", &self.title, 1, TASK_TITLE_MAX_LENGTH)?;
        validate_bounded_string("prompt", &self.prompt, 1, TASK_PROMPT_MAX_LENGTH)?;
        validate_bounded_string("tldr", &self.tldr, 1, TASK_TLDR_MAX_LENGTH)?;
        validate_bounded_string("cwd", &self.cwd, 1, TASK_CWD_MAX_LENGTH)?;
        validate_bounded_string(
            "sessionKey",
            &self.session_key,
            1,
            TASK_SESSION_KEY_MAX_LENGTH,
        )?;
        validate_optional_bounded_string(
            "agentId",
            self.agent_id.as_deref(),
            1,
            TASK_AGENT_ID_MAX_LENGTH,
        )?;
        Ok(())
    }
}

// ---------- TaskSuggestionsCreateResultSchema ----------

/// 对齐 TS:
///   `TaskSuggestionsCreateResultSchema = Type.Object({
///       taskId: TaskIdSchema,
///       suggestion: TaskSuggestionSchema,
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskSuggestionsCreateResultSchema {
    pub task_id: String,
    pub suggestion: TaskSuggestionSchema,
}

impl TaskSuggestionsCreateResultSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_bounded_string("taskId", &self.task_id, 1, TASK_ID_MAX_LENGTH)?;
        self.suggestion.validate()?;
        Ok(())
    }
}

// ---------- TaskSuggestionsAcceptParamsSchema / AcceptResultSchema ----------

/// Atomically claims a pending suggestion and starts its server-owned worktree session.
/// 对齐 TS:
///   `TaskSuggestionsAcceptParamsSchema = Type.Object({
///       taskId: TaskIdSchema,
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskSuggestionsAcceptParamsSchema {
    pub task_id: String,
}

impl TaskSuggestionsAcceptParamsSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_bounded_string("taskId", &self.task_id, 1, TASK_ID_MAX_LENGTH)?;
        Ok(())
    }
}

/// 对齐 TS:
///   `TaskSuggestionsAcceptResultSchema = Type.Object({
///       taskId: TaskIdSchema,
///       key: TaskSessionKeySchema,
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskSuggestionsAcceptResultSchema {
    pub task_id: String,
    pub key: String,
}

impl TaskSuggestionsAcceptResultSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_bounded_string("taskId", &self.task_id, 1, TASK_ID_MAX_LENGTH)?;
        validate_bounded_string("key", &self.key, 1, TASK_SESSION_KEY_MAX_LENGTH)?;
        Ok(())
    }
}

// ---------- TaskSuggestionsDismissParamsSchema / DismissResultSchema ----------

/// Removes a pending suggestion without starting work.
/// 对齐 TS:
///   `TaskSuggestionsDismissParamsSchema = Type.Object({
///       taskId: TaskIdSchema,
///       reason: Type.Optional(Type.String({ maxLength: 1_024 })),
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskSuggestionsDismissParamsSchema {
    pub task_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

impl TaskSuggestionsDismissParamsSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_bounded_string("taskId", &self.task_id, 1, TASK_ID_MAX_LENGTH)?;
        // `reason` is optional with only `maxLength` in TS — `minLength` is not constrained,
        // so an empty string is accepted; mirror that by only enforcing the upper bound.
        if let Some(r) = self.reason.as_deref() {
            if r.chars().count() > TASK_DISMISS_REASON_MAX_LENGTH {
                return Err(format!(
                    "reason: expected length <= {}, got {}",
                    TASK_DISMISS_REASON_MAX_LENGTH,
                    r.chars().count()
                ));
            }
        }
        Ok(())
    }
}

/// 对齐 TS:
///   `TaskSuggestionsDismissResultSchema = Type.Object({
///       taskId: TaskIdSchema,
///       dismissed: Type.Boolean(),
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskSuggestionsDismissResultSchema {
    pub task_id: String,
    pub dismissed: bool,
}

impl TaskSuggestionsDismissResultSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_bounded_string("taskId", &self.task_id, 1, TASK_ID_MAX_LENGTH)?;
        Ok(())
    }
}

// ---------- TaskSuggestionEventSchema ----------

/// Live update emitted when a pending suggestion is created or resolved.
/// 对齐 TS:
///   `TaskSuggestionEventSchema = Type.Union([
///       Type.Object({ action: Type.Literal("created"), suggestion: TaskSuggestionSchema },
///                   { additionalProperties: false }),
///       Type.Object({ action: Type.Literal("resolved"),
///                     taskId: TaskIdSchema,
///                     resolution: TaskSuggestionResolutionSchema },
///                   { additionalProperties: false }),
///   ])`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "action", rename_all = "camelCase")]
pub enum TaskSuggestionEventSchema {
    #[serde(rename = "created")]
    Created { suggestion: TaskSuggestionSchema },
    #[serde(rename = "resolved")]
    Resolved {
        task_id: String,
        resolution: TaskSuggestionResolutionSchema,
    },
}

impl TaskSuggestionEventSchema {
    pub fn validate(&self) -> Result<(), String> {
        match self {
            Self::Created { suggestion } => suggestion.validate(),
            Self::Resolved { task_id, resolution } => {
                validate_bounded_string("taskId", task_id, 1, TASK_ID_MAX_LENGTH)?;
                // `resolution` is a closed enum; serde rejects unknown values at deserialization.
                if TaskSuggestionResolutionSchema::from_str(resolution.as_str()).is_none() {
                    return Err(format!(
                        "invalid task suggestion resolution: {}",
                        resolution.as_str()
                    ));
                }
                Ok(())
            }
        }
    }
}

// Wire types derive directly from local schema consts so public d.ts graphs never
// pull in the ProtocolSchemas registry.
// 对应 TS:
//   export type TaskSuggestion = Static<typeof TaskSuggestionSchema>;
//   export type TaskSuggestionEvent = Static<typeof TaskSuggestionEventSchema>;
//   export type TaskSuggestionResolution = Static<typeof TaskSuggestionResolutionSchema>;
//   export type TaskSuggestionsAcceptParams = Static<typeof TaskSuggestionsAcceptParamsSchema>;
//   export type TaskSuggestionsAcceptResult = Static<typeof TaskSuggestionsAcceptResultSchema>;
//   export type TaskSuggestionsCreateParams = Static<typeof TaskSuggestionsCreateParamsSchema>;
//   export type TaskSuggestionsCreateResult = Static<typeof TaskSuggestionsCreateResultSchema>;
//   export type TaskSuggestionsDismissParams = Static<typeof TaskSuggestionsDismissParamsSchema>;
//   export type TaskSuggestionsDismissResult = Static<typeof TaskSuggestionsDismissResultSchema>;
//   export type TaskSuggestionsListParams = Static<typeof TaskSuggestionsListParamsSchema>;
//   export type TaskSuggestionsListResult = Static<typeof TaskSuggestionsListResultSchema>;
pub type TaskSuggestion = TaskSuggestionSchema;
pub type TaskSuggestionEvent = TaskSuggestionEventSchema;
pub type TaskSuggestionResolution = TaskSuggestionResolutionSchema;
pub type TaskSuggestionsAcceptParams = TaskSuggestionsAcceptParamsSchema;
pub type TaskSuggestionsAcceptResult = TaskSuggestionsAcceptResultSchema;
pub type TaskSuggestionsCreateParams = TaskSuggestionsCreateParamsSchema;
pub type TaskSuggestionsCreateResult = TaskSuggestionsCreateResultSchema;
pub type TaskSuggestionsDismissParams = TaskSuggestionsDismissParamsSchema;
pub type TaskSuggestionsDismissResult = TaskSuggestionsDismissResultSchema;
pub type TaskSuggestionsListParams = TaskSuggestionsListParamsSchema;
pub type TaskSuggestionsListResult = TaskSuggestionsListResultSchema;