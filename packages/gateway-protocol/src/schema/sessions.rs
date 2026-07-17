// Gateway Protocol schema: sessions.
// 翻译自 packages/gateway-protocol/src/schema/sessions.ts
//
// Session protocol schemas.
//
// These requests and results cover transcript discovery, lifecycle control,
// compaction checkpoints, per-session plugin state, and usage reporting. The
// schemas are shared by dashboard, CLI, ACP, and gateway RPC callers.
//
// TS 用 TypeBox 定义 schema（运行时验证 + 类型）。
// Rust 用 serde struct + 验证函数实现等价的序列化/反序列化语义。

use serde::{Deserialize, Serialize};

use crate::frames::ErrorShape;

use super::primitives::SessionLabelString;

// ---------- 依赖占位: PluginJsonValueSchema ----------
// 对齐 TS: `PluginJsonValueSchema = Type.Unknown()` —— 任意 plugin-owned JSON
// payload, gateway 层透明透传. CradleRing 暂未翻译 plugins.rs, 这里用
// `serde_json::Value` 作为等价 Rust 表示.
pub type PluginJsonValueSchema = serde_json::Value;

// ============================================================================
// 基础验证原语
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

fn validate_non_empty_string_list(field: &str, values: &[String]) -> Result<(), String> {
    for (i, v) in values.iter().enumerate() {
        if !is_non_empty_string(v) {
            return Err(format!(
                "{}[{}]: expected non-empty string, got {:?}",
                field, i, v
            ));
        }
    }
    Ok(())
}

/// 对齐 TS: `Type.Integer({ minimum: 0 })`.
fn validate_non_negative_integer(field: &str, n: i64) -> Result<(), String> {
    if n >= 0 {
        Ok(())
    } else {
        Err(format!("{}: expected integer >= 0, got {}", field, n))
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
        Err(format!("{}: expected integer >= 1, got {}", field, n))
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

fn validate_string_min_length(field: &str, value: &str, min: usize) -> Result<(), String> {
    let len = value.chars().count();
    if len >= min {
        Ok(())
    } else {
        Err(format!(
            "{}: expected length >= {}, got {}",
            field, min, len
        ))
    }
}

fn validate_optional_string_min_length(
    field: &str,
    value: Option<&str>,
    min: usize,
) -> Result<(), String> {
    if let Some(s) = value {
        validate_string_min_length(field, s, min)?;
    }
    Ok(())
}

fn validate_string_length_range(
    field: &str,
    value: &str,
    min: usize,
    max: usize,
) -> Result<(), String> {
    let len = value.chars().count();
    if len < min || len > max {
        return Err(format!(
            "{}: expected length [{}, {}], got {}",
            field, min, max, len
        ));
    }
    Ok(())
}

fn regex(pattern: &str) -> regex::Regex {
    regex::Regex::new(pattern).expect("invalid regex pattern compiled into sessions")
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
// Enums
// ============================================================================

// ---------- SessionCompactionCheckpointReasonSchema ----------

/// Reason a compaction checkpoint was created.
/// 对齐 TS:
///   `Type.Union([
///      Type.Literal("manual"),
///      Type.Literal("auto-threshold"),
///      Type.Literal("overflow-retry"),
///      Type.Literal("timeout-retry"),
///   ])`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SessionCompactionCheckpointReasonSchema {
    Manual,
    #[serde(rename = "auto-threshold")]
    AutoThreshold,
    #[serde(rename = "overflow-retry")]
    OverflowRetry,
    #[serde(rename = "timeout-retry")]
    TimeoutRetry,
}

impl SessionCompactionCheckpointReasonSchema {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Manual => "manual",
            Self::AutoThreshold => "auto-threshold",
            Self::OverflowRetry => "overflow-retry",
            Self::TimeoutRetry => "timeout-retry",
        }
    }
}

pub fn is_valid_session_compaction_checkpoint_reason(s: &str) -> bool {
    matches!(
        s,
        "manual" | "auto-threshold" | "overflow-retry" | "timeout-retry"
    )
}

// ---------- SessionOperationEvent operation/phase enums ----------

/// Operation discriminator for `SessionOperationEventSchema`.
/// 对齐 TS: `Type.Literal("compact")`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SessionOperationEventOperation {
    #[serde(rename = "compact")]
    Compact,
}

impl SessionOperationEventOperation {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Compact => "compact",
        }
    }
}

pub fn is_valid_session_operation_event_operation(s: &str) -> bool {
    matches!(s, "compact")
}

/// Phase discriminator for `SessionOperationEventSchema`.
/// 对齐 TS: `Type.Union([Type.Literal("start"), Type.Literal("end")])`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SessionOperationEventPhase {
    Start,
    End,
}

impl SessionOperationEventPhase {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Start => "start",
            Self::End => "end",
        }
    }
}

pub fn is_valid_session_operation_event_phase(s: &str) -> bool {
    matches!(s, "start" | "end")
}

// ---------- SessionFileKindSchema ----------

/// Session file grouping used by the Control UI session workspace rail.
/// 对齐 TS: `Type.Union([Type.Literal("modified"), Type.Literal("read")])`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SessionFileKindSchema {
    Modified,
    Read,
}

impl SessionFileKindSchema {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Modified => "modified",
            Self::Read => "read",
        }
    }
}

pub fn is_valid_session_file_kind(s: &str) -> bool {
    matches!(s, "modified" | "read")
}

// ---------- SessionFileRelevanceSchema ----------

/// Session relevance marker for browser entries.
/// 对齐 TS:
///   `Type.Union([
///      Type.Literal("modified"),
///      Type.Literal("read"),
///      Type.Literal("mixed"),
///   ])`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SessionFileRelevanceSchema {
    Modified,
    Read,
    Mixed,
}

impl SessionFileRelevanceSchema {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Modified => "modified",
            Self::Read => "read",
            Self::Mixed => "mixed",
        }
    }
}

pub fn is_valid_session_file_relevance(s: &str) -> bool {
    matches!(s, "modified" | "read" | "mixed")
}

// ---------- SessionFileBrowserEntrySchema.kind ----------

/// Kind discriminator for `SessionFileBrowserEntrySchema`.
/// 对齐 TS: `Type.Union([Type.Literal("file"), Type.Literal("directory")])`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SessionFileBrowserEntryKind {
    File,
    Directory,
}

impl SessionFileBrowserEntryKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::File => "file",
            Self::Directory => "directory",
        }
    }
}

pub fn is_valid_session_file_browser_entry_kind(s: &str) -> bool {
    matches!(s, "file" | "directory")
}

// ---------- SessionDiffFileStatusSchema ----------

/// Change status for one file in a session checkout diff.
/// 对齐 TS:
///   `Type.Union([
///      Type.Literal("added"),
///      Type.Literal("modified"),
///      Type.Literal("deleted"),
///      Type.Literal("renamed"),
///   ])`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SessionDiffFileStatusSchema {
    Added,
    Modified,
    Deleted,
    Renamed,
}

impl SessionDiffFileStatusSchema {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Added => "added",
            Self::Modified => "modified",
            Self::Deleted => "deleted",
            Self::Renamed => "renamed",
        }
    }
}

pub fn is_valid_session_diff_file_status(s: &str) -> bool {
    matches!(s, "added" | "modified" | "deleted" | "renamed")
}

// ---------- SessionsDiffResultSchema.unavailableReason ----------

/// Reason a session diff could not be computed.
/// 对齐 TS:
///   `Type.Union([Type.Literal("unknown_session"), Type.Literal("not_git")])`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SessionsDiffUnavailableReason {
    #[serde(rename = "unknown_session")]
    UnknownSession,
    NotGit,
}

impl SessionsDiffUnavailableReason {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::UnknownSession => "unknown_session",
            Self::NotGit => "not_git",
        }
    }
}

pub fn is_valid_sessions_diff_unavailable_reason(s: &str) -> bool {
    matches!(s, "unknown_session" | "not_git")
}

// ---------- SessionsSearchHitSchema.role ----------

/// Role discriminator for `SessionsSearchHitSchema`.
/// 对齐 TS: `Type.Union([Type.Literal("user"), Type.Literal("assistant")])`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SessionsSearchHitRole {
    User,
    Assistant,
}

impl SessionsSearchHitRole {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::User => "user",
            Self::Assistant => "assistant",
        }
    }
}

pub fn is_valid_sessions_search_hit_role(s: &str) -> bool {
    matches!(s, "user" | "assistant")
}

// ---------- SessionsResetParamsSchema.reason ----------

/// Reset reason discriminator.
/// 对齐 TS: `Type.Union([Type.Literal("new"), Type.Literal("reset")])`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SessionsResetReason {
    New,
    Reset,
}

impl SessionsResetReason {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::New => "new",
            Self::Reset => "reset",
        }
    }
}

pub fn is_valid_sessions_reset_reason(s: &str) -> bool {
    matches!(s, "new" | "reset")
}

// ---------- SessionsUsageParamsSchema.mode / range / groupBy / agentScope ----------

/// Interpretation mode for usage date filters.
/// 对齐 TS:
///   `Type.Union([
///      Type.Literal("utc"),
///      Type.Literal("gateway"),
///      Type.Literal("specific"),
///   ])`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SessionsUsageMode {
    Utc,
    Gateway,
    Specific,
}

impl SessionsUsageMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Utc => "utc",
            Self::Gateway => "gateway",
            Self::Specific => "specific",
        }
    }
}

pub fn is_valid_sessions_usage_mode(s: &str) -> bool {
    matches!(s, "utc" | "gateway" | "specific")
}

/// Preset range for usage queries when explicit start/end dates are omitted.
/// 对齐 TS:
///   `Type.Union([
///      Type.Literal("7d"), Type.Literal("30d"), Type.Literal("90d"),
///      Type.Literal("1y"), Type.Literal("all"),
///   ])`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SessionsUsageRange {
    #[serde(rename = "7d")]
    SevenDays,
    #[serde(rename = "30d")]
    ThirtyDays,
    #[serde(rename = "90d")]
    NinetyDays,
    #[serde(rename = "1y")]
    OneYear,
    All,
}

impl SessionsUsageRange {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::SevenDays => "7d",
            Self::ThirtyDays => "30d",
            Self::NinetyDays => "90d",
            Self::OneYear => "1y",
            Self::All => "all",
        }
    }
}

pub fn is_valid_sessions_usage_range(s: &str) -> bool {
    matches!(s, "7d" | "30d" | "90d" | "1y" | "all")
}

/// Usage row grouping. `family` rolls up known rotated session ids for a logical key.
/// 对齐 TS: `Type.Union([Type.Literal("instance"), Type.Literal("family")])`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SessionsUsageGroupBy {
    Instance,
    Family,
}

impl SessionsUsageGroupBy {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Instance => "instance",
            Self::Family => "family",
        }
    }
}

pub fn is_valid_sessions_usage_group_by(s: &str) -> bool {
    matches!(s, "instance" | "family")
}

/// Explicit all-agent scope for list-style usage queries.
/// 对齐 TS: `Type.Literal("all")`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SessionsUsageAgentScope {
    #[serde(rename = "all")]
    All,
}

impl SessionsUsageAgentScope {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::All => "all",
        }
    }
}

pub fn is_valid_sessions_usage_agent_scope(s: &str) -> bool {
    matches!(s, "all")
}

// ---------- SessionsMessagesSubscribeParamsSchema.includeApprovals ----------
// 对齐 TS: `Type.Optional(Type.Literal(true))`. 该字段在反序列化时若出现
// 非 `true` 值会被 serde 拒绝 (符合 TS literal 语义).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SessionsMessagesSubscribeIncludeApprovals {
    #[serde(rename = "true")]
    True,
}

impl SessionsMessagesSubscribeIncludeApprovals {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::True => "true",
        }
    }
}

pub fn is_valid_sessions_messages_subscribe_include_approvals(v: &bool) -> bool {
    *v
}

// ---------- SessionsPatchParamsSchema enums ----------

/// Response usage reporting mode.
/// 对齐 TS:
///   `Type.Union([
///      Type.Literal("off"), Type.Literal("tokens"),
///      Type.Literal("full"),
///      // Backward compat with older clients/stores.
///      Type.Literal("on"),
///      Type.Null(),
///   ])`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SessionsPatchResponseUsage {
    Off,
    Tokens,
    Full,
    On,
}

impl SessionsPatchResponseUsage {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Off => "off",
            Self::Tokens => "tokens",
            Self::Full => "full",
            Self::On => "on",
        }
    }
}

pub fn is_valid_sessions_patch_response_usage(s: &str) -> bool {
    matches!(s, "off" | "tokens" | "full" | "on")
}

/// Sub-agent role for session patch.
/// 对齐 TS: `Type.Union([Type.Literal("orchestrator"), Type.Literal("leaf"), Type.Null()])`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SessionsPatchSubagentRole {
    Orchestrator,
    Leaf,
}

impl SessionsPatchSubagentRole {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Orchestrator => "orchestrator",
            Self::Leaf => "leaf",
        }
    }
}

pub fn is_valid_sessions_patch_subagent_role(s: &str) -> bool {
    matches!(s, "orchestrator" | "leaf")
}

/// Sub-agent control scope for session patch.
/// 对齐 TS: `Type.Union([Type.Literal("children"), Type.Literal("none"), Type.Null()])`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SessionsPatchSubagentControlScope {
    Children,
    None,
}

impl SessionsPatchSubagentControlScope {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Children => "children",
            Self::None => "none",
        }
    }
}

pub fn is_valid_sessions_patch_subagent_control_scope(s: &str) -> bool {
    matches!(s, "children" | "none")
}

/// Send policy discriminator for session patch.
/// 对齐 TS: `Type.Union([Type.Literal("allow"), Type.Literal("deny"), Type.Null()])`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SessionsPatchSendPolicy {
    Allow,
    Deny,
}

impl SessionsPatchSendPolicy {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Allow => "allow",
            Self::Deny => "deny",
        }
    }
}

pub fn is_valid_sessions_patch_send_policy(s: &str) -> bool {
    matches!(s, "allow" | "deny")
}

/// Group activation discriminator for session patch.
/// 对齐 TS: `Type.Union([Type.Literal("mention"), Type.Literal("always"), Type.Null()])`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SessionsPatchGroupActivation {
    Mention,
    Always,
}

impl SessionsPatchGroupActivation {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Mention => "mention",
            Self::Always => "always",
        }
    }
}

pub fn is_valid_sessions_patch_group_activation(s: &str) -> bool {
    matches!(s, "mention" | "always")
}

/// Fast mode discriminator for session patch.
/// 对齐 TS: `Type.Union([Type.Boolean(), Type.Literal("auto"), Type.Null()])`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum SessionsPatchFastMode {
    Bool(bool),
    #[serde(rename = "auto")]
    Auto,
}

impl SessionsPatchFastMode {
    pub fn validate(&self) -> Result<(), String> {
        // 布尔值或字面量 "auto" — serde 反序列化时已拒绝其它类型。
        Ok(())
    }
}

pub fn is_valid_sessions_patch_fast_mode_value(_value: &serde_json::Value) -> bool {
    // 由 serde 反序列化层保证; 这里仅占位.
    true
}

// ============================================================================
// Module-private constants and helpers
// ============================================================================

/// 64-char lowercase hex hash pattern.
/// 对齐 TS: `SessionFileHashSchema = Type.String({ minLength: 64, maxLength: 64, pattern: "^[a-f0-9]{64}$" })`.
const SESSION_FILE_HASH_PATTERN: &str = r"^[a-f0-9]{64}$";

/// Worktree name pattern: `[a-z0-9][a-z0-9-]{0,63}`.
/// 对齐 TS: `Type.String({ pattern: "^[a-z0-9][a-z0-9-]{0,63}$" })`.
const WORKTREE_NAME_PATTERN: &str = r"^[a-z0-9][a-z0-9-]{0,63}$";

/// ISO date pattern `YYYY-MM-DD`.
/// 对齐 TS: `Type.String({ pattern: "^\\d{4}-\\d{2}-\\d{2}$" })`.
const SESSIONS_USAGE_DATE_PATTERN: &str = r"^\d{4}-\d{2}-\d{2}$";

/// UTC offset pattern: `UTC+/-H[:MM]`.
/// 对齐 TS: `Type.String({ pattern: "^UTC[+-]\\d{1,2}(?::[0-5]\\d)?$" })`.
const SESSIONS_USAGE_UTC_OFFSET_PATTERN: &str = r"^UTC[+-]\d{1,2}(?::[0-5]\d)?$";

// ============================================================================
// Shared / nested schemas
// ============================================================================

// ---------- SessionOperationEventSchema ----------

/// Start/end event emitted while a session compaction operation runs.
/// 对齐 TS:
///   `Type.Object({
///      operationId: NonEmptyString,
///      operation:   Type.Literal("compact"),
///      phase:       Type.Union([Type.Literal("start"), Type.Literal("end")]),
///      sessionKey:  NonEmptyString,
///      agentId:     Type.Optional(NonEmptyString),
///      ts:          Type.Integer({ minimum: 0 }),
///      completed:   Type.Optional(Type.Boolean()),
///      reason:      Type.Optional(Type.String()),
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionOperationEventSchema {
    pub operation_id: String,
    pub operation: SessionOperationEventOperation,
    pub phase: SessionOperationEventPhase,
    pub session_key: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub agent_id: Option<String>,
    pub ts: i64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub completed: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

impl SessionOperationEventSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_non_empty_string("operationId", &self.operation_id)?;
        validate_non_empty_string("sessionKey", &self.session_key)?;
        validate_optional_non_empty_string("agentId", self.agent_id.as_deref())?;
        validate_non_negative_integer("ts", self.ts)?;
        Ok(())
    }
}

// ---------- SessionCompactionTranscriptReferenceSchema ----------

/// Reference to the transcript location before or after compaction.
/// 对齐 TS:
///   `Type.Object({
///      sessionId:   NonEmptyString,
///      sessionFile: Type.Optional(NonEmptyString),
///      leafId:      Type.Optional(NonEmptyString),
///      entryId:     Type.Optional(NonEmptyString),
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionCompactionTranscriptReferenceSchema {
    pub session_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub session_file: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub leaf_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub entry_id: Option<String>,
}

impl SessionCompactionTranscriptReferenceSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_non_empty_string("sessionId", &self.session_id)?;
        validate_optional_non_empty_string("sessionFile", self.session_file.as_deref())?;
        validate_optional_non_empty_string("leafId", self.leaf_id.as_deref())?;
        validate_optional_non_empty_string("entryId", self.entry_id.as_deref())?;
        Ok(())
    }
}

// ---------- SessionCompactionCheckpointSchema ----------

/// Stored compaction checkpoint metadata for branching or restoring a session.
/// 对齐 TS:
///   `Type.Object({
///      checkpointId:   NonEmptyString,
///      sessionKey:     NonEmptyString,
///      sessionId:      NonEmptyString,
///      createdAt:      Type.Integer({ minimum: 0 }),
///      reason:         SessionCompactionCheckpointReasonSchema,
///      tokensBefore:   Type.Optional(Type.Integer({ minimum: 0 })),
///      tokensAfter:    Type.Optional(Type.Integer({ minimum: 0 })),
///      summary:        Type.Optional(Type.String()),
///      firstKeptEntryId: Type.Optional(NonEmptyString),
///      preCompaction:  SessionCompactionTranscriptReferenceSchema,
///      postCompaction: SessionCompactionTranscriptReferenceSchema,
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionCompactionCheckpointSchema {
    pub checkpoint_id: String,
    pub session_key: String,
    pub session_id: String,
    pub created_at: i64,
    pub reason: SessionCompactionCheckpointReasonSchema,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tokens_before: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tokens_after: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub first_kept_entry_id: Option<String>,
    pub pre_compaction: SessionCompactionTranscriptReferenceSchema,
    pub post_compaction: SessionCompactionTranscriptReferenceSchema,
}

impl SessionCompactionCheckpointSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_non_empty_string("checkpointId", &self.checkpoint_id)?;
        validate_non_empty_string("sessionKey", &self.session_key)?;
        validate_non_empty_string("sessionId", &self.session_id)?;
        validate_non_negative_integer("createdAt", self.created_at)?;
        validate_optional_non_negative_integer("tokensBefore", self.tokens_before)?;
        validate_optional_non_negative_integer("tokensAfter", self.tokens_after)?;
        validate_optional_non_empty_string("firstKeptEntryId", self.first_kept_entry_id.as_deref())?;
        self.pre_compaction
            .validate()
            .map_err(|e| format!("preCompaction: {}", e))?;
        self.post_compaction
            .validate()
            .map_err(|e| format!("postCompaction: {}", e))?;
        Ok(())
    }
}

// ---------- SessionFileEntrySchema ----------

/// One file path referenced by a session transcript.
/// 对齐 TS:
///   `Type.Object({
///      path:         NonEmptyString,
///      workspacePath:Type.Optional(NonEmptyString),
///      name:         NonEmptyString,
///      kind:         SessionFileKindSchema,
///      missing:      Type.Boolean(),
///      size:         Type.Optional(Type.Integer({ minimum: 0 })),
///      updatedAtMs:  Type.Optional(Type.Integer({ minimum: 0 })),
///      content:      Type.Optional(Type.String()),
///      hash:         Type.Optional(SessionFileHashSchema),
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionFileEntrySchema {
    pub path: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub workspace_path: Option<String>,
    pub name: String,
    pub kind: SessionFileKindSchema,
    pub missing: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub size: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub updated_at_ms: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hash: Option<String>,
}

impl SessionFileEntrySchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_non_empty_string("path", &self.path)?;
        validate_optional_non_empty_string("workspacePath", self.workspace_path.as_deref())?;
        validate_non_empty_string("name", &self.name)?;
        validate_optional_non_negative_integer("size", self.size)?;
        validate_optional_non_negative_integer("updatedAtMs", self.updated_at_ms)?;
        if let Some(hash) = &self.hash {
            validate_string_length_range("hash", hash, 64, 64)?;
            validate_pattern("hash", hash, SESSION_FILE_HASH_PATTERN)?;
        }
        Ok(())
    }
}

// ---------- SessionFileBrowserEntrySchema ----------

/// One file or folder in the session-rooted browser.
/// 对齐 TS:
///   `Type.Object({
///      path:        Type.String(),
///      name:        NonEmptyString,
///      kind:        Type.Union([Type.Literal("file"), Type.Literal("directory")]),
///      sessionKind: Type.Optional(SessionFileRelevanceSchema),
///      size:        Type.Optional(Type.Integer({ minimum: 0 })),
///      updatedAtMs: Type.Optional(Type.Integer({ minimum: 0 })),
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionFileBrowserEntrySchema {
    pub path: String,
    pub name: String,
    pub kind: SessionFileBrowserEntryKind,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub session_kind: Option<SessionFileRelevanceSchema>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub size: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub updated_at_ms: Option<i64>,
}

impl SessionFileBrowserEntrySchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_non_empty_string("name", &self.name)?;
        validate_optional_non_negative_integer("size", self.size)?;
        validate_optional_non_negative_integer("updatedAtMs", self.updated_at_ms)?;
        Ok(())
    }
}

// ---------- SessionFileBrowserResultSchema ----------

/// Folder listing or search result rooted at the session workspace.
/// 对齐 TS:
///   `Type.Object({
///      path:       Type.String(),
///      parentPath: Type.Optional(Type.String()),
///      search:     Type.Optional(Type.String()),
///      entries:    Type.Array(SessionFileBrowserEntrySchema),
///      truncated:  Type.Optional(Type.Boolean()),
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionFileBrowserResultSchema {
    pub path: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent_path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub search: Option<String>,
    pub entries: Vec<SessionFileBrowserEntrySchema>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub truncated: Option<bool>,
}

impl SessionFileBrowserResultSchema {
    pub fn validate(&self) -> Result<(), String> {
        for (i, entry) in self.entries.iter().enumerate() {
            entry
                .validate()
                .map_err(|e| format!("entries[{}]: {}", i, e))?;
        }
        Ok(())
    }
}

// ---------- SessionsFilesListParamsSchema ----------

/// Lists files touched by a session transcript.
/// 对齐 TS:
///   `Type.Object({
///      sessionKey: NonEmptyString,
///      agentId:    Type.Optional(NonEmptyString),
///      path:       Type.Optional(Type.String()),
///      search:     Type.Optional(Type.String()),
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionsFilesListParamsSchema {
    pub session_key: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub agent_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub search: Option<String>,
}

impl SessionsFilesListParamsSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_non_empty_string("sessionKey", &self.session_key)?;
        validate_optional_non_empty_string("agentId", self.agent_id.as_deref())?;
        Ok(())
    }
}

// ---------- SessionsFilesListResultSchema ----------

/// File references visible in one session workspace.
/// 对齐 TS:
///   `Type.Object({
///      sessionKey: NonEmptyString,
///      root:       Type.Optional(NonEmptyString),
///      files:      Type.Array(SessionFileEntrySchema),
///      browser:    Type.Optional(SessionFileBrowserResultSchema),
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionsFilesListResultSchema {
    pub session_key: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub root: Option<String>,
    pub files: Vec<SessionFileEntrySchema>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub browser: Option<SessionFileBrowserResultSchema>,
}

impl SessionsFilesListResultSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_non_empty_string("sessionKey", &self.session_key)?;
        validate_optional_non_empty_string("root", self.root.as_deref())?;
        for (i, f) in self.files.iter().enumerate() {
            f.validate().map_err(|e| format!("files[{}]: {}", i, e))?;
        }
        if let Some(b) = &self.browser {
            b.validate().map_err(|e| format!("browser: {}", e))?;
        }
        Ok(())
    }
}

// ---------- SessionsFilesGetParamsSchema ----------

/// Reads one session-referenced file by path.
/// 对齐 TS:
///   `Type.Object({
///      sessionKey: NonEmptyString,
///      path:       NonEmptyString,
///      agentId:    Type.Optional(NonEmptyString),
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionsFilesGetParamsSchema {
    pub session_key: String,
    pub path: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub agent_id: Option<String>,
}

impl SessionsFilesGetParamsSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_non_empty_string("sessionKey", &self.session_key)?;
        validate_non_empty_string("path", &self.path)?;
        validate_optional_non_empty_string("agentId", self.agent_id.as_deref())?;
        Ok(())
    }
}

// ---------- SessionsFilesGetResultSchema ----------

/// Result for reading one session-referenced file.
/// 对齐 TS:
///   `Type.Object({
///      sessionKey: NonEmptyString,
///      root:       Type.Optional(NonEmptyString),
///      file:       SessionFileEntrySchema,
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionsFilesGetResultSchema {
    pub session_key: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub root: Option<String>,
    pub file: SessionFileEntrySchema,
}

impl SessionsFilesGetResultSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_non_empty_string("sessionKey", &self.session_key)?;
        validate_optional_non_empty_string("root", self.root.as_deref())?;
        self.file.validate().map_err(|e| format!("file: {}", e))?;
        Ok(())
    }
}

// ---------- SessionsFilesSetParamsSchema ----------

/// Overwrites one existing session workspace file with hash-based CAS.
/// 对齐 TS:
///   `Type.Object({
///      sessionKey:   NonEmptyString,
///      path:         NonEmptyString,
///      agentId:      Type.Optional(NonEmptyString),
///      content:      Type.String(),
///      expectedHash: SessionFileHashSchema,
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionsFilesSetParamsSchema {
    pub session_key: String,
    pub path: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub agent_id: Option<String>,
    pub content: String,
    pub expected_hash: String,
}

impl SessionsFilesSetParamsSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_non_empty_string("sessionKey", &self.session_key)?;
        validate_non_empty_string("path", &self.path)?;
        validate_optional_non_empty_string("agentId", self.agent_id.as_deref())?;
        validate_string_length_range("expectedHash", &self.expected_hash, 64, 64)?;
        validate_pattern("expectedHash", &self.expected_hash, SESSION_FILE_HASH_PATTERN)?;
        Ok(())
    }
}

// ---------- SessionsFilesSetResultSchema ----------

/// Result for overwriting one session workspace file.
/// 对齐 TS:
///   `Type.Object({
///      sessionKey: NonEmptyString,
///      root:       Type.Optional(NonEmptyString),
///      file:       SessionFileEntrySchema,
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionsFilesSetResultSchema {
    pub session_key: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub root: Option<String>,
    pub file: SessionFileEntrySchema,
}

impl SessionsFilesSetResultSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_non_empty_string("sessionKey", &self.session_key)?;
        validate_optional_non_empty_string("root", self.root.as_deref())?;
        self.file.validate().map_err(|e| format!("file: {}", e))?;
        Ok(())
    }
}

// ---------- SessionDiffFileSchema ----------

/// One changed file in a session checkout diff.
/// 对齐 TS:
///   `Type.Object({
///      path:     NonEmptyString,
///      oldPath:  Type.Optional(NonEmptyString),
///      status:   SessionDiffFileStatusSchema,
///      additions:Type.Integer({ minimum: 0 }),
///      deletions:Type.Integer({ minimum: 0 }),
///      binary:   Type.Optional(Type.Boolean()),
///      untracked:Type.Optional(Type.Boolean()),
///      /** Per-file unified patch text; absent for binary or oversized files. */
///      patch:    Type.Optional(Type.String()),
///      truncated:Type.Optional(Type.Boolean()),
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionDiffFileSchema {
    pub path: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub old_path: Option<String>,
    pub status: SessionDiffFileStatusSchema,
    pub additions: i64,
    pub deletions: i64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub binary: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub untracked: Option<bool>,
    /// Per-file unified patch text; absent for binary or oversized files.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub patch: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub truncated: Option<bool>,
}

impl SessionDiffFileSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_non_empty_string("path", &self.path)?;
        validate_optional_non_empty_string("oldPath", self.old_path.as_deref())?;
        validate_non_negative_integer("additions", self.additions)?;
        validate_non_negative_integer("deletions", self.deletions)?;
        Ok(())
    }
}

// ---------- SessionsDiffParamsSchema ----------

/// Reads the git diff of a session checkout against its base branch.
/// 对齐 TS:
///   `Type.Object({
///      sessionKey: NonEmptyString,
///      agentId:    Type.Optional(NonEmptyString),
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionsDiffParamsSchema {
    pub session_key: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub agent_id: Option<String>,
}

impl SessionsDiffParamsSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_non_empty_string("sessionKey", &self.session_key)?;
        validate_optional_non_empty_string("agentId", self.agent_id.as_deref())?;
        Ok(())
    }
}

// ---------- SessionsDiffResultSchema ----------

/// Branch + working-tree diff for one session checkout.
/// 对齐 TS:
///   `Type.Object({
///      sessionKey:      NonEmptyString,
///      root:            Type.Optional(NonEmptyString),
///      branch:          Type.Optional(NonEmptyString),
///      /** Display label of the diff base: the default branch name or "HEAD". */
///      baseRef:         Type.Optional(NonEmptyString),
///      files:           Type.Array(SessionDiffFileSchema),
///      additions:       Type.Integer({ minimum: 0 }),
///      deletions:       Type.Integer({ minimum: 0 }),
///      truncated:       Type.Optional(Type.Boolean()),
///      unavailableReason: Type.Optional(
///        Type.Union([Type.Literal("unknown_session"), Type.Literal("not_git")]),
///      ),
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionsDiffResultSchema {
    pub session_key: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub root: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub branch: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub base_ref: Option<String>,
    pub files: Vec<SessionDiffFileSchema>,
    pub additions: i64,
    pub deletions: i64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub truncated: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub unavailable_reason: Option<SessionsDiffUnavailableReason>,
}

impl SessionsDiffResultSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_non_empty_string("sessionKey", &self.session_key)?;
        validate_optional_non_empty_string("root", self.root.as_deref())?;
        validate_optional_non_empty_string("branch", self.branch.as_deref())?;
        validate_optional_non_empty_string("baseRef", self.base_ref.as_deref())?;
        for (i, f) in self.files.iter().enumerate() {
            f.validate().map_err(|e| format!("files[{}]: {}", i, e))?;
        }
        validate_non_negative_integer("additions", self.additions)?;
        validate_non_negative_integer("deletions", self.deletions)?;
        Ok(())
    }
}

// ---------- SessionsListParamsSchema ----------

/// Lists sessions with optional scope, activity, label, and preview filters.
/// 对齐 TS:
///   `Type.Object({
///      limit: Type.Optional(Type.Integer({ minimum: 1 })),
///      offset: Type.Optional(Type.Integer({ minimum: 0 })),
///      activeMinutes: Type.Optional(Type.Integer({ minimum: 1 })),
///      includeGlobal: Type.Optional(Type.Boolean()),
///      includeUnknown: Type.Optional(Type.Boolean()),
///      configuredAgentsOnly: Type.Optional(Type.Boolean()),
///      includeDerivedTitles: Type.Optional(Type.Boolean()),
///      includeLastMessage: Type.Optional(Type.Boolean()),
///      label: Type.Optional(SessionLabelString),
///      spawnedBy: Type.Optional(NonEmptyString),
///      agentId: Type.Optional(NonEmptyString),
///      search: Type.Optional(Type.String()),
///      archived: Type.Optional(Type.Boolean()),
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionsListParamsSchema {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub limit: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub offset: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub active_minutes: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub include_global: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub include_unknown: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub configured_agents_only: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub include_derived_titles: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub include_last_message: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<SessionLabelString>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub spawned_by: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub agent_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub search: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub archived: Option<bool>,
}

impl SessionsListParamsSchema {
    pub fn validate(&self) -> Result<(), String> {
        if let Some(n) = self.limit {
            validate_positive_integer("limit", n)?;
        }
        validate_optional_non_negative_integer("offset", self.offset)?;
        if let Some(n) = self.active_minutes {
            validate_positive_integer("activeMinutes", n)?;
        }
        if let Some(label) = &self.label {
            validate_non_empty_string("label", label)?;
        }
        validate_optional_non_empty_string("spawnedBy", self.spawned_by.as_deref())?;
        validate_optional_non_empty_string("agentId", self.agent_id.as_deref())?;
        Ok(())
    }
}

// ---------- SessionsSearchParamsSchema ----------

/// Searches one agent's indexed session transcripts, optionally within selected sessions.
/// 对齐 TS:
///   `Type.Object({
///      agentId:     Type.Optional(NonEmptyString),
///      sessionKeys: Type.Optional(Type.Array(NonEmptyString, { minItems: 1, maxItems: 200 })),
///      query:       Type.String({ minLength: 1, maxLength: 4096 }),
///      limit:       Type.Optional(Type.Integer({ minimum: 1, maximum: 25 })),
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionsSearchParamsSchema {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub agent_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub session_keys: Option<Vec<String>>,
    pub query: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub limit: Option<i64>,
}

const SESSIONS_SEARCH_QUERY_MAX_LENGTH: usize = 4096;
const SESSIONS_SEARCH_SESSION_KEYS_MAX_ITEMS: usize = 200;
const SESSIONS_SEARCH_LIMIT_MAX: i64 = 25;

impl SessionsSearchParamsSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_optional_non_empty_string("agentId", self.agent_id.as_deref())?;
        if let Some(keys) = &self.session_keys {
            if keys.len() > SESSIONS_SEARCH_SESSION_KEYS_MAX_ITEMS {
                return Err(format!(
                    "sessionKeys: expected length <= {}, got {}",
                    SESSIONS_SEARCH_SESSION_KEYS_MAX_ITEMS,
                    keys.len()
                ));
            }
            validate_non_empty_string_list("sessionKeys", keys)?;
        }
        validate_string_length_range(
            "query",
            &self.query,
            1,
            SESSIONS_SEARCH_QUERY_MAX_LENGTH,
        )?;
        if let Some(limit) = self.limit {
            validate_integer_in_range("limit", limit, 1, SESSIONS_SEARCH_LIMIT_MAX)?;
        }
        Ok(())
    }
}

// ---------- SessionsSearchHitSchema ----------

/// One full-text session transcript match with follow-up provenance.
/// 对齐 TS:
///   `Type.Object({
///      sessionKey: NonEmptyString,
///      sessionId:  NonEmptyString,
///      messageId:  NonEmptyString,
///      role:       Type.Union([Type.Literal("user"), Type.Literal("assistant")]),
///      timestamp:  Type.Integer({ minimum: 0 }),
///      snippet:    Type.String(),
///      score:      Type.Number(),
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionsSearchHitSchema {
    pub session_key: String,
    pub session_id: String,
    pub message_id: String,
    pub role: SessionsSearchHitRole,
    pub timestamp: i64,
    pub snippet: String,
    pub score: f64,
}

impl SessionsSearchHitSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_non_empty_string("sessionKey", &self.session_key)?;
        validate_non_empty_string("sessionId", &self.session_id)?;
        validate_non_empty_string("messageId", &self.message_id)?;
        validate_non_negative_integer("timestamp", self.timestamp)?;
        Ok(())
    }
}

// ---------- SessionsSearchResultSchema ----------

/// Full-text search response; indexing marks a still-running first-use reconcile.
/// 对齐 TS:
///   `Type.Object({
///      results:  Type.Array(SessionsSearchHitSchema),
///      indexing: Type.Optional(Type.Boolean()),
///      truncated:Type.Optional(Type.Boolean()),
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionsSearchResultSchema {
    pub results: Vec<SessionsSearchHitSchema>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub indexing: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub truncated: Option<bool>,
}

impl SessionsSearchResultSchema {
    pub fn validate(&self) -> Result<(), String> {
        for (i, hit) in self.results.iter().enumerate() {
            hit.validate()
                .map_err(|e| format!("results[{}]: {}", i, e))?;
        }
        Ok(())
    }
}

// ---------- SessionsCleanupParamsSchema ----------

/// Repairs or removes invalid session records from the selected agent scope.
/// 对齐 TS:
///   `Type.Object({
///      agent:        Type.Optional(NonEmptyString),
///      allAgents:    Type.Optional(Type.Boolean()),
///      enforce:      Type.Optional(Type.Boolean()),
///      activeKey:    Type.Optional(NonEmptyString),
///      fixMissing:   Type.Optional(Type.Boolean()),
///      fixDmScope:   Type.Optional(Type.Boolean()),
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionsCleanupParamsSchema {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub agent: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub all_agents: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub enforce: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub active_key: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fix_missing: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fix_dm_scope: Option<bool>,
}

impl SessionsCleanupParamsSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_optional_non_empty_string("agent", self.agent.as_deref())?;
        validate_optional_non_empty_string("activeKey", self.active_key.as_deref())?;
        Ok(())
    }
}

// ---------- SessionsPreviewParamsSchema ----------

/// Reads short previews for selected session keys.
/// 对齐 TS:
///   `Type.Object({
///      keys:     Type.Array(NonEmptyString, { minItems: 1 }),
///      limit:    Type.Optional(Type.Integer({ minimum: 1 })),
///      maxChars: Type.Optional(Type.Integer({ minimum: 20 })),
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionsPreviewParamsSchema {
    pub keys: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub limit: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_chars: Option<i64>,
}

const SESSIONS_PREVIEW_MAX_CHARS_MIN: i64 = 20;

impl SessionsPreviewParamsSchema {
    pub fn validate(&self) -> Result<(), String> {
        if self.keys.is_empty() {
            return Err("keys: expected at least 1 item, got 0".to_string());
        }
        validate_non_empty_string_list("keys", &self.keys)?;
        if let Some(n) = self.limit {
            validate_positive_integer("limit", n)?;
        }
        if let Some(n) = self.max_chars {
            if n < SESSIONS_PREVIEW_MAX_CHARS_MIN {
                return Err(format!(
                    "maxChars: expected integer >= {}, got {}",
                    SESSIONS_PREVIEW_MAX_CHARS_MIN, n
                ));
            }
        }
        Ok(())
    }
}

// ---------- SessionsDescribeParamsSchema ----------

/// Describes one session and optional derived title/last-message previews.
/// 对齐 TS:
///   `Type.Object({
///      key:                   NonEmptyString,
///      includeDerivedTitles:  Type.Optional(Type.Boolean()),
///      includeLastMessage:    Type.Optional(Type.Boolean()),
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionsDescribeParamsSchema {
    pub key: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub include_derived_titles: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub include_last_message: Option<bool>,
}

impl SessionsDescribeParamsSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_non_empty_string("key", &self.key)?;
        Ok(())
    }
}

// ---------- SessionsResolveParamsSchema ----------

/// Resolves a session by key, raw session id, label, or parent/agent scope.
/// 对齐 TS:
///   `Type.Object({
///      key:           Type.Optional(NonEmptyString),
///      sessionId:     Type.Optional(NonEmptyString),
///      label:         Type.Optional(SessionLabelString),
///      agentId:       Type.Optional(NonEmptyString),
///      spawnedBy:     Type.Optional(NonEmptyString),
///      includeGlobal: Type.Optional(Type.Boolean()),
///      includeUnknown:Type.Optional(Type.Boolean()),
///      allowMissing:  Type.Optional(Type.Boolean()),
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionsResolveParamsSchema {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub key: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<SessionLabelString>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub agent_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub spawned_by: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub include_global: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub include_unknown: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub allow_missing: Option<bool>,
}

impl SessionsResolveParamsSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_optional_non_empty_string("key", self.key.as_deref())?;
        validate_optional_non_empty_string("sessionId", self.session_id.as_deref())?;
        if let Some(label) = &self.label {
            validate_non_empty_string("label", label)?;
        }
        validate_optional_non_empty_string("agentId", self.agent_id.as_deref())?;
        validate_optional_non_empty_string("spawnedBy", self.spawned_by.as_deref())?;
        Ok(())
    }
}

// ---------- SessionsCreateParamsSchema ----------

/// Creates or adopts a session with optional model, label, and parent linkage.
/// 对齐 TS:
///   `Type.Object({
///      key:              Type.Optional(NonEmptyString),
///      agentId:          Type.Optional(NonEmptyString),
///      label:            Type.Optional(SessionLabelString),
///      model:            Type.Optional(NonEmptyString),
///      parentSessionKey: Type.Optional(NonEmptyString),
///      fork:             Type.Optional(Type.Boolean({ description: ... })),
///      emitCommandHooks: Type.Optional(Type.Boolean()),
///      task:             Type.Optional(Type.String()),
///      message:          Type.Optional(Type.String()),
///      worktree:         Type.Optional(Type.Boolean()),
///      worktreeBaseRef:  Type.Optional(Type.String({ minLength: 1, description: ... })),
///      worktreeName:     Type.Optional(Type.String({ pattern: "...", description: ... })),
///      execNode:         Type.Optional(Type.String({ minLength: 1, description: ... })),
///      cwd:              Type.Optional(Type.String({ minLength: 1, description: ... })),
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionsCreateParamsSchema {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub key: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub agent_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<SessionLabelString>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent_session_key: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fork: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub emit_command_hooks: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub task: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub worktree: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub worktree_base_ref: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub worktree_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub exec_node: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cwd: Option<String>,
}

impl SessionsCreateParamsSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_optional_non_empty_string("key", self.key.as_deref())?;
        validate_optional_non_empty_string("agentId", self.agent_id.as_deref())?;
        if let Some(label) = &self.label {
            validate_non_empty_string("label", label)?;
        }
        validate_optional_non_empty_string("model", self.model.as_deref())?;
        validate_optional_non_empty_string("parentSessionKey", self.parent_session_key.as_deref())?;
        validate_optional_string_min_length("worktreeBaseRef", self.worktree_base_ref.as_deref(), 1)?;
        if let Some(name) = &self.worktree_name {
            validate_pattern("worktreeName", name, WORKTREE_NAME_PATTERN)?;
        }
        validate_optional_string_min_length("execNode", self.exec_node.as_deref(), 1)?;
        validate_optional_string_min_length("cwd", self.cwd.as_deref(), 1)?;
        Ok(())
    }
}

// ---------- SessionWorktreeInfoSchema ----------

/// 对齐 TS: `Type.Object({ id: NonEmptyString, path: NonEmptyString, branch: NonEmptyString }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionWorktreeInfoSchema {
    pub id: String,
    pub path: String,
    pub branch: String,
}

impl SessionWorktreeInfoSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_non_empty_string("id", &self.id)?;
        validate_non_empty_string("path", &self.path)?;
        validate_non_empty_string("branch", &self.branch)?;
        Ok(())
    }
}

// ---------- SessionsCreateResultSchema ----------

/// Result returned after creating or adopting a session.
/// 对齐 TS:
///   `Type.Object({
///      ok:          Type.Literal(true),
///      key:         NonEmptyString,
///      sessionId:   Type.Optional(NonEmptyString),
///      entry:       Type.Optional(Type.Record(Type.String(), Type.Unknown())),
///      runStarted:  Type.Optional(Type.Boolean()),
///      runError:    Type.Optional(ErrorShapeSchema),
///      worktree:    Type.Optional(SessionWorktreeInfoSchema),
///   }, { additionalProperties: true })`.
///
/// 注意: TS 用 `ok: Type.Literal(true)` 固定为 `true`; Rust 用 `bool` 即可,
/// runtime validate 时确认 `ok == true`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionsCreateResultSchema {
    pub ok: bool,
    pub key: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub entry: Option<PluginJsonValueSchema>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub run_started: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub run_error: Option<ErrorShape>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub worktree: Option<SessionWorktreeInfoSchema>,
}

impl SessionsCreateResultSchema {
    pub fn validate(&self) -> Result<(), String> {
        if !self.ok {
            return Err("ok: expected literal true".to_string());
        }
        validate_non_empty_string("key", &self.key)?;
        validate_optional_non_empty_string("sessionId", self.session_id.as_deref())?;
        if let Some(w) = &self.worktree {
            w.validate().map_err(|e| format!("worktree: {}", e))?;
        }
        Ok(())
    }
}

// ---------- SessionsSendParamsSchema ----------

/// Sends one message into an existing session.
/// 对齐 TS:
///   `Type.Object({
///      key:             NonEmptyString,
///      agentId:         Type.Optional(NonEmptyString),
///      message:         Type.String(),
///      thinking:        Type.Optional(Type.String()),
///      attachments:     Type.Optional(Type.Array(Type.Unknown())),
///      timeoutMs:       Type.Optional(Type.Integer({ minimum: 0 })),
///      idempotencyKey:  Type.Optional(NonEmptyString),
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionsSendParamsSchema {
    pub key: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub agent_id: Option<String>,
    pub message: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub thinking: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub attachments: Option<Vec<PluginJsonValueSchema>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub timeout_ms: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub idempotency_key: Option<String>,
}

impl SessionsSendParamsSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_non_empty_string("key", &self.key)?;
        validate_optional_non_empty_string("agentId", self.agent_id.as_deref())?;
        validate_optional_non_negative_integer("timeoutMs", self.timeout_ms)?;
        validate_optional_non_empty_string("idempotencyKey", self.idempotency_key.as_deref())?;
        Ok(())
    }
}

// ---------- SessionsMessagesSubscribeParamsSchema ----------

/// Subscribes a client to live message updates for one session.
/// 对齐 TS:
///   `Type.Object({
///      key:               NonEmptyString,
///      agentId:           Type.Optional(NonEmptyString),
///      includeApprovals:  Type.Optional(Type.Literal(true)),
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionsMessagesSubscribeParamsSchema {
    pub key: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub agent_id: Option<String>,
    /// Opt in to sanitized durable approval events for this session and its descendants.
    /// 对齐 TS: `Type.Optional(Type.Literal(true))`. Rust 端用 `bool` 替代,
    /// validate 时确保只有 `true` 是合法值.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub include_approvals: Option<bool>,
}

impl SessionsMessagesSubscribeParamsSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_non_empty_string("key", &self.key)?;
        validate_optional_non_empty_string("agentId", self.agent_id.as_deref())?;
        if let Some(v) = self.include_approvals {
            if !v {
                return Err("includeApprovals: expected literal true".to_string());
            }
        }
        Ok(())
    }
}

// ---------- SessionsMessagesUnsubscribeParamsSchema ----------

/// Removes a live message subscription for one session.
/// 对齐 TS:
///   `Type.Object({
///      key:     NonEmptyString,
///      agentId: Type.Optional(NonEmptyString),
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionsMessagesUnsubscribeParamsSchema {
    pub key: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub agent_id: Option<String>,
}

impl SessionsMessagesUnsubscribeParamsSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_non_empty_string("key", &self.key)?;
        validate_optional_non_empty_string("agentId", self.agent_id.as_deref())?;
        Ok(())
    }
}

// ---------- SessionsAbortParamsSchema ----------

/// Aborts the active or named run for a session.
/// 对齐 TS:
///   `Type.Object({
///      key:     Type.Optional(NonEmptyString),
///      runId:   Type.Optional(NonEmptyString),
///      agentId: Type.Optional(NonEmptyString),
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionsAbortParamsSchema {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub key: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub run_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub agent_id: Option<String>,
}

impl SessionsAbortParamsSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_optional_non_empty_string("key", self.key.as_deref())?;
        validate_optional_non_empty_string("runId", self.run_id.as_deref())?;
        validate_optional_non_empty_string("agentId", self.agent_id.as_deref())?;
        Ok(())
    }
}

// ---------- SessionsPatchParamsSchema ----------
//
// 该 schema 字段极多, TS 用 `Type.Union([X, Type.Null()])` 表示"可清除"
// 三态语义; Rust 端统一用 `Option<T>` 表示, 通过 validate 拒绝空串.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionsPatchParamsSchema {
    pub key: String,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub agent_id: Option<String>,

    /// `Some(None)` 表示清除 (TS 的 `Type.Union([X, Type.Null()])`).
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "optional_nullable_string"
    )]
    pub label: Option<Option<SessionLabelString>>,

    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "optional_nullable_string"
    )]
    pub category: Option<Option<SessionLabelString>>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub archived: Option<bool>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pinned: Option<bool>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub unread: Option<bool>,

    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "optional_nullable_string"
    )]
    pub thinking_level: Option<Option<String>>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fast_mode: Option<SessionsPatchFastMode>,

    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "optional_nullable_string"
    )]
    pub verbose_level: Option<Option<String>>,

    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "optional_nullable_string"
    )]
    pub trace_level: Option<Option<String>>,

    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "optional_nullable_string"
    )]
    pub reasoning_level: Option<Option<String>>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub response_usage: Option<Option<SessionsPatchResponseUsage>>,

    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "optional_nullable_string"
    )]
    pub elevated_level: Option<Option<String>>,

    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "optional_nullable_string"
    )]
    pub exec_host: Option<Option<String>>,

    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "optional_nullable_string"
    )]
    pub exec_security: Option<Option<String>>,

    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "optional_nullable_string"
    )]
    pub exec_ask: Option<Option<String>>,

    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "optional_nullable_string"
    )]
    pub exec_node: Option<Option<String>>,

    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "optional_nullable_string"
    )]
    pub model: Option<Option<String>>,

    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "optional_nullable_string"
    )]
    pub spawned_by: Option<Option<String>>,

    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "optional_nullable_string"
    )]
    pub spawned_workspace_dir: Option<Option<String>>,

    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "optional_nullable_string"
    )]
    pub spawned_cwd: Option<Option<String>>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub spawn_depth: Option<Option<i64>>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub subagent_role: Option<Option<SessionsPatchSubagentRole>>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub subagent_control_scope: Option<Option<SessionsPatchSubagentControlScope>>,

    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "optional_nullable_string_list"
    )]
    pub inherited_tool_allow: Option<Option<Vec<String>>>,

    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "optional_nullable_string_list"
    )]
    pub inherited_tool_deny: Option<Option<Vec<String>>>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub send_policy: Option<Option<SessionsPatchSendPolicy>>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub group_activation: Option<Option<SessionsPatchGroupActivation>>,
}

impl SessionsPatchParamsSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_non_empty_string("key", &self.key)?;
        validate_optional_non_empty_string("agentId", self.agent_id.as_deref())?;
        if let Some(label) = &self.label {
            validate_optional_non_empty_string("label", label.as_deref())?;
        }
        if let Some(category) = &self.category {
            validate_optional_non_empty_string("category", category.as_deref())?;
        }
        if let Some(level) = &self.thinking_level {
            validate_optional_non_empty_string("thinkingLevel", level.as_deref())?;
        }
        if let Some(mode) = &self.fast_mode {
            mode.validate()?;
        }
        if let Some(level) = &self.verbose_level {
            validate_optional_non_empty_string("verboseLevel", level.as_deref())?;
        }
        if let Some(level) = &self.trace_level {
            validate_optional_non_empty_string("traceLevel", level.as_deref())?;
        }
        if let Some(level) = &self.reasoning_level {
            validate_optional_non_empty_string("reasoningLevel", level.as_deref())?;
        }
        if let Some(level) = &self.elevated_level {
            validate_optional_non_empty_string("elevatedLevel", level.as_deref())?;
        }
        if let Some(v) = &self.exec_host {
            validate_optional_non_empty_string("execHost", v.as_deref())?;
        }
        if let Some(v) = &self.exec_security {
            validate_optional_non_empty_string("execSecurity", v.as_deref())?;
        }
        if let Some(v) = &self.exec_ask {
            validate_optional_non_empty_string("execAsk", v.as_deref())?;
        }
        if let Some(v) = &self.exec_node {
            validate_optional_non_empty_string("execNode", v.as_deref())?;
        }
        if let Some(v) = &self.model {
            validate_optional_non_empty_string("model", v.as_deref())?;
        }
        if let Some(v) = &self.spawned_by {
            validate_optional_non_empty_string("spawnedBy", v.as_deref())?;
        }
        if let Some(v) = &self.spawned_workspace_dir {
            validate_optional_non_empty_string("spawnedWorkspaceDir", v.as_deref())?;
        }
        if let Some(v) = &self.spawned_cwd {
            validate_optional_non_empty_string("spawnedCwd", v.as_deref())?;
        }
        if let Some(n) = self.spawn_depth.flatten() {
            validate_non_negative_integer("spawnDepth", n)?;
        }
        if let Some(list) = self.inherited_tool_allow.as_ref() {
            if let Some(arr) = list.as_ref() {
                validate_non_empty_string_list("inheritedToolAllow", arr)?;
            }
        }
        if let Some(list) = self.inherited_tool_deny.as_ref() {
            if let Some(arr) = list.as_ref() {
                validate_non_empty_string_list("inheritedToolDeny", arr)?;
            }
        }
        Ok(())
    }
}

// Serde helpers: 处理 `Option<Option<T>>` 三态序列化 (None / Some(null) / Some(value)).
mod optional_nullable_string {
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(value: &Option<Option<String>>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match value {
            Some(inner) => serializer.serialize_some(&inner.as_ref()),
            None => serializer.serialize_none(),
        }
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<Option<String>>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let opt: Option<Option<String>> = Option::deserialize(deserializer)?;
        Ok(opt)
    }
}

mod optional_nullable_string_list {
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(
        value: &Option<Option<Vec<String>>>,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match value {
            Some(inner) => serializer.serialize_some(&inner.as_ref()),
            None => serializer.serialize_none(),
        }
    }

    pub fn deserialize<'de, D>(
        deserializer: D,
    ) -> Result<Option<Option<Vec<String>>>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let opt: Option<Option<Vec<String>>> = Option::deserialize(deserializer)?;
        Ok(opt)
    }
}

// ---------- SessionsPluginPatchParamsSchema ----------

/// Updates or clears one plugin namespace value on a session record.
/// 对齐 TS:
///   `Type.Object({
///      key:       NonEmptyString,
///      pluginId:  NonEmptyString,
///      namespace: NonEmptyString,
///      value:     Type.Optional(PluginJsonValueSchema),
///      unset:     Type.Optional(Type.Boolean()),
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionsPluginPatchParamsSchema {
    pub key: String,
    pub plugin_id: String,
    pub namespace: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub value: Option<PluginJsonValueSchema>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub unset: Option<bool>,
}

impl SessionsPluginPatchParamsSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_non_empty_string("key", &self.key)?;
        validate_non_empty_string("pluginId", &self.plugin_id)?;
        validate_non_empty_string("namespace", &self.namespace)?;
        Ok(())
    }
}

// ---------- SessionsPluginPatchResultSchema ----------

/// Result returned after patching session plugin state.
/// 对齐 TS:
///   `Type.Object({
///      ok:    Type.Literal(true),
///      key:   NonEmptyString,
///      value: Type.Optional(PluginJsonValueSchema),
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionsPluginPatchResultSchema {
    pub ok: bool,
    pub key: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub value: Option<PluginJsonValueSchema>,
}

impl SessionsPluginPatchResultSchema {
    pub fn validate(&self) -> Result<(), String> {
        if !self.ok {
            return Err("ok: expected literal true".to_string());
        }
        validate_non_empty_string("key", &self.key)?;
        Ok(())
    }
}

// ---------- SessionsResetParamsSchema ----------

/// Resets a session to a new or reset transcript state.
/// 对齐 TS:
///   `Type.Object({
///      key:     NonEmptyString,
///      agentId: Type.Optional(NonEmptyString),
///      reason:  Type.Optional(Type.Union([Type.Literal("new"), Type.Literal("reset")])),
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionsResetParamsSchema {
    pub key: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub agent_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<SessionsResetReason>,
}

impl SessionsResetParamsSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_non_empty_string("key", &self.key)?;
        validate_optional_non_empty_string("agentId", self.agent_id.as_deref())?;
        Ok(())
    }
}

// ---------- SessionsDeleteParamsSchema ----------

/// Deletes a session record and optionally its transcript.
/// 对齐 TS:
///   `Type.Object({
///      key:                       NonEmptyString,
///      agentId:                   Type.Optional(NonEmptyString),
///      deleteTranscript:          Type.Optional(Type.Boolean()),
///      expectedSessionId:         Type.Optional(NonEmptyString),
///      expectedLifecycleRevision: Type.Optional(NonEmptyString),
///      expectedSessionUpdatedAt:  Type.Optional(Type.Number({ minimum: 0 })),
///      emitLifecycleHooks:        Type.Optional(Type.Boolean()),
///      archivedOnly:              Type.Optional(Type.Boolean()),
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionsDeleteParamsSchema {
    pub key: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub agent_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub delete_transcript: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expected_session_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expected_lifecycle_revision: Option<String>,
    /// 对齐 TS: `Type.Number({ minimum: 0 })`. Rust 端用 f64 表示.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expected_session_updated_at: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub emit_lifecycle_hooks: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub archived_only: Option<bool>,
}

impl SessionsDeleteParamsSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_non_empty_string("key", &self.key)?;
        validate_optional_non_empty_string("agentId", self.agent_id.as_deref())?;
        validate_optional_non_empty_string(
            "expectedSessionId",
            self.expected_session_id.as_deref(),
        )?;
        validate_optional_non_empty_string(
            "expectedLifecycleRevision",
            self.expected_lifecycle_revision.as_deref(),
        )?;
        if let Some(n) = self.expected_session_updated_at {
            if n < 0.0 {
                return Err(format!(
                    "expectedSessionUpdatedAt: expected number >= 0, got {}",
                    n
                ));
            }
        }
        Ok(())
    }
}

// ---------- SessionsGroupsListParamsSchema ----------

/// 对齐 TS: `Type.Object({}, { additionalProperties: false })`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SessionsGroupsListParamsSchema {}

impl SessionsGroupsListParamsSchema {
    pub fn validate(&self) -> Result<(), String> {
        Ok(())
    }
}

// ---------- SessionGroupSchema ----------

/// One custom session group catalog entry.
/// 对齐 TS: `Type.Object({ name: SessionLabelString, position: Type.Integer({ minimum: 0 }) }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionGroupSchema {
    pub name: SessionLabelString,
    pub position: i64,
}

impl SessionGroupSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_non_empty_string("name", &self.name)?;
        validate_non_negative_integer("position", self.position)?;
        Ok(())
    }
}

// ---------- SessionsGroupsListResultSchema ----------

/// 对齐 TS: `Type.Object({ groups: Type.Array(SessionGroupSchema) }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionsGroupsListResultSchema {
    pub groups: Vec<SessionGroupSchema>,
}

impl SessionsGroupsListResultSchema {
    pub fn validate(&self) -> Result<(), String> {
        for (i, g) in self.groups.iter().enumerate() {
            g.validate().map_err(|e| format!("groups[{}]: {}", i, e))?;
        }
        Ok(())
    }
}

// ---------- SessionsGroupsPutParamsSchema ----------

/// Replaces the ordered group catalog; creates listed names, keeps member categories untouched.
/// 对齐 TS: `Type.Object({ names: Type.Array(SessionLabelString, { maxItems: 200 }) }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionsGroupsPutParamsSchema {
    pub names: Vec<SessionLabelString>,
}

const SESSIONS_GROUPS_NAMES_MAX_ITEMS: usize = 200;

impl SessionsGroupsPutParamsSchema {
    pub fn validate(&self) -> Result<(), String> {
        if self.names.len() > SESSIONS_GROUPS_NAMES_MAX_ITEMS {
            return Err(format!(
                "names: expected length <= {}, got {}",
                SESSIONS_GROUPS_NAMES_MAX_ITEMS,
                self.names.len()
            ));
        }
        for (i, n) in self.names.iter().enumerate() {
            validate_non_empty_string(&format!("names[{}]", i), n)?;
        }
        Ok(())
    }
}

// ---------- SessionsGroupsRenameParamsSchema ----------

/// 对齐 TS: `Type.Object({ name: SessionLabelString, to: SessionLabelString }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionsGroupsRenameParamsSchema {
    pub name: SessionLabelString,
    pub to: SessionLabelString,
}

impl SessionsGroupsRenameParamsSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_non_empty_string("name", &self.name)?;
        validate_non_empty_string("to", &self.to)?;
        Ok(())
    }
}

// ---------- SessionsGroupsDeleteParamsSchema ----------

/// 对齐 TS: `Type.Object({ name: SessionLabelString }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionsGroupsDeleteParamsSchema {
    pub name: SessionLabelString,
}

impl SessionsGroupsDeleteParamsSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_non_empty_string("name", &self.name)?;
        Ok(())
    }
}

// ---------- SessionsGroupsMutationResultSchema ----------

/// Result for group catalog mutations, with member sessions updated where applicable.
/// 对齐 TS:
///   `Type.Object({
///      ok: Type.Literal(true),
///      groups: Type.Array(SessionGroupSchema),
///      updatedSessions: Type.Optional(Type.Integer({ minimum: 0 })),
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionsGroupsMutationResultSchema {
    pub ok: bool,
    pub groups: Vec<SessionGroupSchema>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub updated_sessions: Option<i64>,
}

impl SessionsGroupsMutationResultSchema {
    pub fn validate(&self) -> Result<(), String> {
        if !self.ok {
            return Err("ok: expected literal true".to_string());
        }
        for (i, g) in self.groups.iter().enumerate() {
            g.validate().map_err(|e| format!("groups[{}]: {}", i, e))?;
        }
        validate_optional_non_negative_integer("updatedSessions", self.updated_sessions)?;
        Ok(())
    }
}

// ---------- SessionsCompactParamsSchema ----------

/// Requests manual compaction for a session transcript.
/// 对齐 TS:
///   `Type.Object({
///      key:      NonEmptyString,
///      agentId:  Type.Optional(NonEmptyString),
///      maxLines: Type.Optional(Type.Integer({ minimum: 1 })),
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionsCompactParamsSchema {
    pub key: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub agent_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_lines: Option<i64>,
}

impl SessionsCompactParamsSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_non_empty_string("key", &self.key)?;
        validate_optional_non_empty_string("agentId", self.agent_id.as_deref())?;
        if let Some(n) = self.max_lines {
            validate_positive_integer("maxLines", n)?;
        }
        Ok(())
    }
}

// ---------- SessionsCompactionListParamsSchema ----------

/// Lists compaction checkpoints for one session.
/// 对齐 TS: `Type.Object({ key: NonEmptyString, agentId: Type.Optional(NonEmptyString) }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionsCompactionListParamsSchema {
    pub key: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub agent_id: Option<String>,
}

impl SessionsCompactionListParamsSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_non_empty_string("key", &self.key)?;
        validate_optional_non_empty_string("agentId", self.agent_id.as_deref())?;
        Ok(())
    }
}

// ---------- SessionsCompactionGetParamsSchema ----------

/// Reads one compaction checkpoint by id.
/// 对齐 TS:
///   `Type.Object({
///      key:          NonEmptyString,
///      agentId:      Type.Optional(NonEmptyString),
///      checkpointId: NonEmptyString,
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionsCompactionGetParamsSchema {
    pub key: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub agent_id: Option<String>,
    pub checkpoint_id: String,
}

impl SessionsCompactionGetParamsSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_non_empty_string("key", &self.key)?;
        validate_optional_non_empty_string("agentId", self.agent_id.as_deref())?;
        validate_non_empty_string("checkpointId", &self.checkpoint_id)?;
        Ok(())
    }
}

// ---------- SessionsCompactionBranchParamsSchema ----------

/// Creates a new branch from a compaction checkpoint.
/// 对齐 TS: 与 Get 一致 (key, agentId?, checkpointId).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionsCompactionBranchParamsSchema {
    pub key: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub agent_id: Option<String>,
    pub checkpoint_id: String,
}

impl SessionsCompactionBranchParamsSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_non_empty_string("key", &self.key)?;
        validate_optional_non_empty_string("agentId", self.agent_id.as_deref())?;
        validate_non_empty_string("checkpointId", &self.checkpoint_id)?;
        Ok(())
    }
}

// ---------- SessionsCompactionRestoreParamsSchema ----------

/// Restores an existing session to a compaction checkpoint.
/// 对齐 TS: 与 Get/Branch 一致 (key, agentId?, checkpointId).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionsCompactionRestoreParamsSchema {
    pub key: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub agent_id: Option<String>,
    pub checkpoint_id: String,
}

impl SessionsCompactionRestoreParamsSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_non_empty_string("key", &self.key)?;
        validate_optional_non_empty_string("agentId", self.agent_id.as_deref())?;
        validate_non_empty_string("checkpointId", &self.checkpoint_id)?;
        Ok(())
    }
}

// ---------- SessionsCompactionListResultSchema ----------

/// List response for session compaction checkpoints.
/// 对齐 TS:
///   `Type.Object({
///      ok: Type.Literal(true),
///      key: NonEmptyString,
///      checkpoints: Type.Array(SessionCompactionCheckpointSchema),
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionsCompactionListResultSchema {
    pub ok: bool,
    pub key: String,
    pub checkpoints: Vec<SessionCompactionCheckpointSchema>,
}

impl SessionsCompactionListResultSchema {
    pub fn validate(&self) -> Result<(), String> {
        if !self.ok {
            return Err("ok: expected literal true".to_string());
        }
        validate_non_empty_string("key", &self.key)?;
        for (i, cp) in self.checkpoints.iter().enumerate() {
            cp.validate()
                .map_err(|e| format!("checkpoints[{}]: {}", i, e))?;
        }
        Ok(())
    }
}

// ---------- SessionsCompactionGetResultSchema ----------

/// Get response for a single compaction checkpoint.
/// 对齐 TS:
///   `Type.Object({
///      ok: Type.Literal(true),
///      key: NonEmptyString,
///      checkpoint: SessionCompactionCheckpointSchema,
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionsCompactionGetResultSchema {
    pub ok: bool,
    pub key: String,
    pub checkpoint: SessionCompactionCheckpointSchema,
}

impl SessionsCompactionGetResultSchema {
    pub fn validate(&self) -> Result<(), String> {
        if !self.ok {
            return Err("ok: expected literal true".to_string());
        }
        validate_non_empty_string("key", &self.key)?;
        self.checkpoint
            .validate()
            .map_err(|e| format!("checkpoint: {}", e))?;
        Ok(())
    }
}

// ---------- CompactionEntryMetadata (shared nested type) ----------

/// Compact entry metadata shared by branch/restore result shapes.
/// 对齐 TS:
///   `Type.Object({
///      sessionId: NonEmptyString,
///      updatedAt: Type.Integer({ minimum: 0 }),
///   }, { additionalProperties: true })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompactionEntryMetadata {
    pub session_id: String,
    pub updated_at: i64,
}

impl CompactionEntryMetadata {
    pub fn validate(&self) -> Result<(), String> {
        validate_non_empty_string("sessionId", &self.session_id)?;
        validate_non_negative_integer("updatedAt", self.updated_at)?;
        Ok(())
    }
}

// ---------- SessionsCompactionBranchResultSchema ----------

/// Branch response with the newly created session key and entry metadata.
/// 对齐 TS:
///   `Type.Object({
///      ok: Type.Literal(true),
///      sourceKey: NonEmptyString,
///      key:       NonEmptyString,
///      sessionId: NonEmptyString,
///      checkpoint:SessionCompactionCheckpointSchema,
///      entry:     Type.Object({ sessionId, updatedAt }, { additionalProperties: true }),
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionsCompactionBranchResultSchema {
    pub ok: bool,
    pub source_key: String,
    pub key: String,
    pub session_id: String,
    pub checkpoint: SessionCompactionCheckpointSchema,
    pub entry: CompactionEntryMetadata,
}

impl SessionsCompactionBranchResultSchema {
    pub fn validate(&self) -> Result<(), String> {
        if !self.ok {
            return Err("ok: expected literal true".to_string());
        }
        validate_non_empty_string("sourceKey", &self.source_key)?;
        validate_non_empty_string("key", &self.key)?;
        validate_non_empty_string("sessionId", &self.session_id)?;
        self.checkpoint
            .validate()
            .map_err(|e| format!("checkpoint: {}", e))?;
        self.entry.validate().map_err(|e| format!("entry: {}", e))?;
        Ok(())
    }
}

// ---------- SessionsCompactionRestoreResultSchema ----------

/// Restore response with updated session entry metadata.
/// 对齐 TS:
///   `Type.Object({
///      ok: Type.Literal(true),
///      key:       NonEmptyString,
///      sessionId: NonEmptyString,
///      checkpoint:SessionCompactionCheckpointSchema,
///      entry:     Type.Object({ sessionId, updatedAt }, { additionalProperties: true }),
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionsCompactionRestoreResultSchema {
    pub ok: bool,
    pub key: String,
    pub session_id: String,
    pub checkpoint: SessionCompactionCheckpointSchema,
    pub entry: CompactionEntryMetadata,
}

impl SessionsCompactionRestoreResultSchema {
    pub fn validate(&self) -> Result<(), String> {
        if !self.ok {
            return Err("ok: expected literal true".to_string());
        }
        validate_non_empty_string("key", &self.key)?;
        validate_non_empty_string("sessionId", &self.session_id)?;
        self.checkpoint
            .validate()
            .map_err(|e| format!("checkpoint: {}", e))?;
        self.entry.validate().map_err(|e| format!("entry: {}", e))?;
        Ok(())
    }
}

// ---------- SessionsUsageParamsSchema ----------

/// Usage report query across one session, one agent, or all agent sessions.
/// 对齐 TS:
///   `Type.Object({
///      key:                   Type.Optional(NonEmptyString),
///      agentId:               Type.Optional(NonEmptyString),
///      agentScope:            Type.Optional(Type.Literal("all")),
///      startDate:             Type.Optional(Type.String({ pattern: "^\\d{4}-\\d{2}-\\d{2}$" })),
///      endDate:               Type.Optional(Type.String({ pattern: "^\\d{4}-\\d{2}-\\d{2}$" })),
///      mode:                  Type.Optional(Type.Union([Type.Literal("utc"),
///                                       Type.Literal("gateway"), Type.Literal("specific")])),
///      range:                 Type.Optional(Type.Union([Type.Literal("7d"), ...])),
///      groupBy:               Type.Optional(Type.Union([Type.Literal("instance"), Type.Literal("family")])),
///      includeHistorical:     Type.Optional(Type.Boolean()),
///      utcOffset:             Type.Optional(Type.String({ pattern: "^UTC[+-]\\d{1,2}(?::[0-5]\\d)?$" })),
///      timeZone:              Type.Optional(NonEmptyString),
///      limit:                 Type.Optional(Type.Integer({ minimum: 1 })),
///      includeContextWeight:  Type.Optional(Type.Boolean()),
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionsUsageParamsSchema {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub key: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub agent_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub agent_scope: Option<SessionsUsageAgentScope>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub start_date: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub end_date: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mode: Option<SessionsUsageMode>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub range: Option<SessionsUsageRange>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub group_by: Option<SessionsUsageGroupBy>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub include_historical: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub utc_offset: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub time_zone: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub limit: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub include_context_weight: Option<bool>,
}

impl SessionsUsageParamsSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_optional_non_empty_string("key", self.key.as_deref())?;
        validate_optional_non_empty_string("agentId", self.agent_id.as_deref())?;
        validate_optional_pattern("startDate", self.start_date.as_deref(), SESSIONS_USAGE_DATE_PATTERN)?;
        validate_optional_pattern("endDate", self.end_date.as_deref(), SESSIONS_USAGE_DATE_PATTERN)?;
        validate_optional_pattern("utcOffset", self.utc_offset.as_deref(), SESSIONS_USAGE_UTC_OFFSET_PATTERN)?;
        validate_optional_non_empty_string("timeZone", self.time_zone.as_deref())?;
        if let Some(limit) = self.limit {
            validate_positive_integer("limit", limit)?;
        }
        Ok(())
    }
}

// ============================================================================
// Wire types (对齐 TS `export type X = Static<typeof XSchema>;`)
// ============================================================================
//
// Wire types derive directly from local schema consts so public d.ts graphs never
// pull in the ProtocolSchemas registry.
// 对应 TS 末尾所有 `export type X = Static<typeof XSchema>` 一一映射.

pub type SessionsListParams = SessionsListParamsSchema;
pub type SessionsCleanupParams = SessionsCleanupParamsSchema;
pub type SessionsPreviewParams = SessionsPreviewParamsSchema;
pub type SessionsDescribeParams = SessionsDescribeParamsSchema;
pub type SessionsResolveParams = SessionsResolveParamsSchema;
pub type SessionsSearchParams = SessionsSearchParamsSchema;
pub type SessionsSearchHit = SessionsSearchHitSchema;
pub type SessionsSearchResult = SessionsSearchResultSchema;
pub type SessionCompactionCheckpoint = SessionCompactionCheckpointSchema;
pub type SessionOperationEvent = SessionOperationEventSchema;
pub type SessionsCompactionListParams = SessionsCompactionListParamsSchema;
pub type SessionsCompactionGetParams = SessionsCompactionGetParamsSchema;
pub type SessionsCompactionBranchParams = SessionsCompactionBranchParamsSchema;
pub type SessionsCompactionRestoreParams = SessionsCompactionRestoreParamsSchema;
pub type SessionsCompactionListResult = SessionsCompactionListResultSchema;
pub type SessionsCompactionGetResult = SessionsCompactionGetResultSchema;
pub type SessionsCompactionBranchResult = SessionsCompactionBranchResultSchema;
pub type SessionsCompactionRestoreResult = SessionsCompactionRestoreResultSchema;
pub type SessionWorktreeInfo = SessionWorktreeInfoSchema;
pub type SessionsCreateParams = SessionsCreateParamsSchema;
pub type SessionsCreateResult = SessionsCreateResultSchema;
pub type SessionsSendParams = SessionsSendParamsSchema;
pub type SessionsMessagesSubscribeParams = SessionsMessagesSubscribeParamsSchema;
pub type SessionsMessagesUnsubscribeParams = SessionsMessagesUnsubscribeParamsSchema;
pub type SessionsAbortParams = SessionsAbortParamsSchema;
pub type SessionsPluginPatchParams = SessionsPluginPatchParamsSchema;
pub type SessionsPluginPatchResult = SessionsPluginPatchResultSchema;
pub type SessionsResetParams = SessionsResetParamsSchema;
pub type SessionsDeleteParams = SessionsDeleteParamsSchema;
pub type SessionGroup = SessionGroupSchema;
pub type SessionsGroupsListParams = SessionsGroupsListParamsSchema;
pub type SessionsGroupsListResult = SessionsGroupsListResultSchema;
pub type SessionsGroupsPutParams = SessionsGroupsPutParamsSchema;
pub type SessionsGroupsRenameParams = SessionsGroupsRenameParamsSchema;
pub type SessionsGroupsDeleteParams = SessionsGroupsDeleteParamsSchema;
pub type SessionsGroupsMutationResult = SessionsGroupsMutationResultSchema;
pub type SessionsCompactParams = SessionsCompactParamsSchema;
pub type SessionsUsageParams = SessionsUsageParamsSchema;
pub type SessionFileKind = SessionFileKindSchema;
pub type SessionFileRelevance = SessionFileRelevanceSchema;
pub type SessionFileEntry = SessionFileEntrySchema;
pub type SessionFileBrowserEntry = SessionFileBrowserEntrySchema;
pub type SessionFileBrowserResult = SessionFileBrowserResultSchema;
pub type SessionsFilesListParams = SessionsFilesListParamsSchema;
pub type SessionsFilesListResult = SessionsFilesListResultSchema;
pub type SessionsFilesGetParams = SessionsFilesGetParamsSchema;
pub type SessionsFilesGetResult = SessionsFilesGetResultSchema;
pub type SessionsFilesSetParams = SessionsFilesSetParamsSchema;
pub type SessionsFilesSetResult = SessionsFilesSetResultSchema;
pub type SessionDiffFileStatus = SessionDiffFileStatusSchema;
pub type SessionDiffFile = SessionDiffFileSchema;
pub type SessionsDiffParams = SessionsDiffParamsSchema;
pub type SessionsDiffResult = SessionsDiffResultSchema;