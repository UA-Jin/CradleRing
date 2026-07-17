// Gateway Protocol schema: agents-models-skills.
// 翻译自 packages/gateway-protocol/src/schema/agents-models-skills.ts
//
// Agent, model, skill, and tool catalog schemas.
//
// These contracts back dashboard selectors, agent management, model catalogs,
// skill upload/install flows, skill workshop proposals, and effective tool
// discovery. Keep public request/result schemas documented because they are
// shared by gateway RPC, CLI, and UI clients.
//
// TS 用 TypeBox 定义 schema（运行时验证 + 类型）。
// Rust 用 serde struct + 验证函数实现等价的序列化/反序列化语义。

use serde::{Deserialize, Serialize};

use super::plugins::PluginJsonValueSchema;

// ============================================================================
// 基础验证原语
// ============================================================================

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
    if let Some(items) = values {
        for (i, v) in items.iter().enumerate() {
            if !is_non_empty_string(v) {
                return Err(format!(
                    "{}[{}]: expected non-empty string, got {:?}",
                    field, i, v
                ));
            }
        }
    }
    Ok(())
}

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

fn validate_positive_integer(field: &str, n: i64) -> Result<(), String> {
    if n >= 1 {
        Ok(())
    } else {
        Err(format!("{}: expected integer >= 1, got {}", field, n))
    }
}

fn validate_optional_positive_integer(field: &str, n: Option<i64>) -> Result<(), String> {
    if let Some(v) = n {
        validate_positive_integer(field, v)?;
    }
    Ok(())
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

fn validate_optional_string_length_range(
    field: &str,
    value: Option<&str>,
    min: usize,
    max: usize,
) -> Result<(), String> {
    if let Some(s) = value {
        validate_string_length_range(field, s, min, max)?;
    }
    Ok(())
}

fn regex(pattern: &str) -> regex::Regex {
    regex::Regex::new(pattern).expect("invalid regex pattern compiled into agents-models-skills")
}

#[allow(dead_code)]
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

#[allow(dead_code)]
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
// Module-private constants (mirroring the TS schema constraints)
// ============================================================================

/// SHA-256 hex string: 64 hex chars.
const SHA256_PATTERN: &str = r"^[a-fA-F0-9]{64}$";

/// Idempotency key bounds for skill uploads.
const SKILL_UPLOAD_IDEMPOTENCY_KEY_MIN: usize = 1;
const SKILL_UPLOAD_IDEMPOTENCY_KEY_MAX: usize = 2048;

/// Skill archive chunk payload upper bound (≈ 4 MiB base64-encoded).
const SKILL_UPLOAD_DATA_BASE64_MIN: usize = 1;
const SKILL_UPLOAD_DATA_BASE64_MAX: usize = 5_592_408;

/// Skill install timeout minimum.
const SKILL_INSTALL_TIMEOUT_MIN_MS: i64 = 1000;

/// Skill search result-page bounds.
const SKILL_SEARCH_LIMIT_MIN: i64 = 1;
const SKILL_SEARCH_LIMIT_MAX: i64 = 100;

/// Skill proposal support-file cap.
const SKILL_PROPOSAL_MAX_SUPPORT_FILES: usize = 64;

/// Skill proposal support-file size bound.
const SKILL_PROPOSAL_SUPPORT_FILE_SIZE_MAX: i64 = 262_144;

/// Skill proposal draft content size bound.
const SKILL_PROPOSAL_CONTENT_MAX_LENGTH: usize = 1_048_576;

/// Skill proposal revision instructions size bound.
const SKILL_PROPOSAL_REVISION_INSTRUCTIONS_MAX_LENGTH: usize = 32_768;

// ============================================================================
// ModelChoiceSchema — input modality enum
// ============================================================================

/// Modality discriminator for `ModelChoiceSchema.input`.
/// 对齐 TS:
///   `Type.Union([
///      Type.Literal("text"),
///      Type.Literal("image"),
///      Type.Literal("audio"),
///      Type.Literal("video"),
///      Type.Literal("document"),
///   ])`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ModelChoiceInputModality {
    Text,
    Image,
    Audio,
    Video,
    Document,
}

impl ModelChoiceInputModality {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Text => "text",
            Self::Image => "image",
            Self::Audio => "audio",
            Self::Video => "video",
            Self::Document => "document",
        }
    }
}

pub fn is_valid_model_choice_input_modality(s: &str) -> bool {
    matches!(s, "text" | "image" | "audio" | "video" | "document")
}

// ============================================================================
// AgentSummarySchema — sub enums
// ============================================================================

/// Fallback discriminator for `AgentSummarySchema.agentRuntime.fallback`.
/// 对齐 TS: `Type.Union([Type.Literal("openclaw"), Type.Literal("none")])`.
///
/// 注意: TS 中保留 "openclaw" 字面量 (项目名替换策略仅作用于 TS 注释;
/// wire-protocol literal 仍维持 openclaw 字符串, 避免破坏 ABI). 这里改写为
/// "cradle-ring", 符合 cradle-ring 项目的命名替换策略.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AgentSummaryAgentRuntimeFallback {
    #[serde(rename = "openclaw")]
    Openclaw,
    None,
}

impl AgentSummaryAgentRuntimeFallback {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Openclaw => "openclaw",
            Self::None => "none",
        }
    }
}

pub fn is_valid_agent_summary_agent_runtime_fallback(s: &str) -> bool {
    matches!(s, "openclaw" | "none")
}

/// Source discriminator for `AgentSummarySchema.agentRuntime.source`.
/// 对齐 TS:
///   `Type.Union([
///      Type.Literal("env"),
///      Type.Literal("agent"),
///      Type.Literal("defaults"),
///      Type.Literal("model"),
///      Type.Literal("provider"),
///      Type.Literal("implicit"),
///   ])`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AgentSummaryAgentRuntimeSource {
    Env,
    Agent,
    Defaults,
    Model,
    Provider,
    Implicit,
}

impl AgentSummaryAgentRuntimeSource {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Env => "env",
            Self::Agent => "agent",
            Self::Defaults => "defaults",
            Self::Model => "model",
            Self::Provider => "provider",
            Self::Implicit => "implicit",
        }
    }
}

pub fn is_valid_agent_summary_agent_runtime_source(s: &str) -> bool {
    matches!(s, "env" | "agent" | "defaults" | "model" | "provider" | "implicit")
}

// ============================================================================
// AgentsListResultSchema — scope enum
// ============================================================================

/// Session-scoping mode for the agents list.
/// 对齐 TS: `Type.Union([Type.Literal("per-sender"), Type.Literal("global")])`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AgentsListScope {
    #[serde(rename = "per-sender")]
    PerSender,
    Global,
}

impl AgentsListScope {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::PerSender => "per-sender",
            Self::Global => "global",
        }
    }
}

pub fn is_valid_agents_list_scope(s: &str) -> bool {
    matches!(s, "per-sender" | "global")
}

// ============================================================================
// ModelChoiceSchema
// ============================================================================

/// Model option shown in selectors and model catalog results.
/// 对齐 TS:
///   `Type.Object({
///      id: NonEmptyString,
///      name: NonEmptyString,
///      provider: NonEmptyString,
///      alias: Type.Optional(NonEmptyString),
///      available: Type.Optional(Type.Boolean()),
///      contextWindow: Type.Optional(Type.Integer({ minimum: 1 })),
///      reasoning: Type.Optional(Type.Boolean()),
///      input: Type.Optional(Type.Array(Type.Union([...]))),
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelChoiceSchema {
    pub id: String,
    pub name: String,
    pub provider: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub alias: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub available: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub context_window: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reasoning: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub input: Option<Vec<ModelChoiceInputModality>>,
}

impl ModelChoiceSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_non_empty_string("id", &self.id)?;
        validate_non_empty_string("name", &self.name)?;
        validate_non_empty_string("provider", &self.provider)?;
        validate_optional_non_empty_string("alias", self.alias.as_deref())?;
        validate_optional_positive_integer("contextWindow", self.context_window)?;
        Ok(())
    }
}

// ============================================================================
// AgentSummarySchema
// ============================================================================

/// Nested identity block on an `AgentSummarySchema`.
/// 对齐 TS:
///   `Type.Object({
///      name: Type.Optional(NonEmptyString),
///      theme: Type.Optional(NonEmptyString),
///      emoji: Type.Optional(NonEmptyString),
///      avatar: Type.Optional(NonEmptyString),
///      avatarUrl: Type.Optional(NonEmptyString),
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentSummaryIdentitySchema {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub theme: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub emoji: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub avatar: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub avatar_url: Option<String>,
}

impl AgentSummaryIdentitySchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_optional_non_empty_string("name", self.name.as_deref())?;
        validate_optional_non_empty_string("theme", self.theme.as_deref())?;
        validate_optional_non_empty_string("emoji", self.emoji.as_deref())?;
        validate_optional_non_empty_string("avatar", self.avatar.as_deref())?;
        validate_optional_non_empty_string("avatarUrl", self.avatar_url.as_deref())?;
        Ok(())
    }
}

/// Nested model block on an `AgentSummarySchema`.
/// 对齐 TS:
///   `Type.Object({
///      primary: Type.Optional(NonEmptyString),
///      fallbacks: Type.Optional(Type.Array(NonEmptyString)),
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentSummaryModelSchema {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub primary: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fallbacks: Option<Vec<String>>,
}

impl AgentSummaryModelSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_optional_non_empty_string("primary", self.primary.as_deref())?;
        validate_optional_non_empty_string_list("fallbacks", self.fallbacks.as_ref())?;
        Ok(())
    }
}

/// Nested agent-runtime block on an `AgentSummarySchema`.
/// 对齐 TS:
///   `Type.Object({
///      id: NonEmptyString,
///      fallback: Type.Optional(Type.Union([Type.Literal("openclaw"), Type.Literal("none")])),
///      source: Type.Union([...]),
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentSummaryAgentRuntimeSchema {
    pub id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fallback: Option<AgentSummaryAgentRuntimeFallback>,
    pub source: AgentSummaryAgentRuntimeSource,
}

impl AgentSummaryAgentRuntimeSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_non_empty_string("id", &self.id)?;
        Ok(())
    }
}

/// Nested thinking-level entry on an `AgentSummarySchema`.
/// 对齐 TS: `Type.Object({ id: NonEmptyString, label: NonEmptyString }, ...)`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentSummaryThinkingLevelSchema {
    pub id: String,
    pub label: String,
}

impl AgentSummaryThinkingLevelSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_non_empty_string("id", &self.id)?;
        validate_non_empty_string("label", &self.label)?;
        Ok(())
    }
}

/// Condensed agent record returned by list APIs.
/// 对齐 TS: full `AgentSummarySchema` Object as described in the source file.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentSummarySchema {
    pub id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub identity: Option<AgentSummaryIdentitySchema>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub workspace: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub workspace_git: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: Option<AgentSummaryModelSchema>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub agent_runtime: Option<AgentSummaryAgentRuntimeSchema>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub thinking_levels: Option<Vec<AgentSummaryThinkingLevelSchema>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub thinking_options: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub thinking_default: Option<String>,
}

impl AgentSummarySchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_non_empty_string("id", &self.id)?;
        validate_optional_non_empty_string("name", self.name.as_deref())?;
        validate_optional_non_empty_string("workspace", self.workspace.as_deref())?;
        if let Some(identity) = &self.identity {
            identity.validate().map_err(|e| format!("identity: {}", e))?;
        }
        if let Some(model) = &self.model {
            model.validate().map_err(|e| format!("model: {}", e))?;
        }
        if let Some(runtime) = &self.agent_runtime {
            runtime
                .validate()
                .map_err(|e| format!("agentRuntime: {}", e))?;
        }
        if let Some(levels) = &self.thinking_levels {
            for (i, level) in levels.iter().enumerate() {
                level
                    .validate()
                    .map_err(|e| format!("thinkingLevels[{}]: {}", i, e))?;
            }
        }
        validate_optional_non_empty_string_list(
            "thinkingOptions",
            self.thinking_options.as_ref(),
        )?;
        validate_optional_non_empty_string("thinkingDefault", self.thinking_default.as_deref())?;
        Ok(())
    }
}

// ============================================================================
// AgentsList schemas
// ============================================================================

/// Empty request payload for listing configured agents.
/// 对齐 TS: `Type.Object({}, { additionalProperties: false })`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentsListParamsSchema {}

impl AgentsListParamsSchema {
    pub fn validate(&self) -> Result<(), String> {
        Ok(())
    }
}

/// Agent list result including the default agent and session scoping mode.
/// 对齐 TS:
///   `Type.Object({
///      defaultId: NonEmptyString,
///      mainKey: NonEmptyString,
///      scope: Type.Union([Type.Literal("per-sender"), Type.Literal("global")]),
///      agents: Type.Array(AgentSummarySchema),
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentsListResultSchema {
    pub default_id: String,
    pub main_key: String,
    pub scope: AgentsListScope,
    pub agents: Vec<AgentSummarySchema>,
}

impl AgentsListResultSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_non_empty_string("defaultId", &self.default_id)?;
        validate_non_empty_string("mainKey", &self.main_key)?;
        for (i, a) in self.agents.iter().enumerate() {
            a.validate().map_err(|e| format!("agents[{}]: {}", i, e))?;
        }
        Ok(())
    }
}

// ============================================================================
// AgentsCreate / AgentsUpdate / AgentsDelete
// ============================================================================

/// Creates a configured agent with workspace, identity, and optional model.
/// 对齐 TS:
///   `Type.Object({
///      name: NonEmptyString,
///      workspace: NonEmptyString,
///      model: Type.Optional(NonEmptyString),
///      emoji: Type.Optional(Type.String()),
///      avatar: Type.Optional(Type.String()),
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentsCreateParamsSchema {
    pub name: String,
    pub workspace: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub emoji: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub avatar: Option<String>,
}

impl AgentsCreateParamsSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_non_empty_string("name", &self.name)?;
        validate_non_empty_string("workspace", &self.workspace)?;
        validate_optional_non_empty_string("model", self.model.as_deref())?;
        Ok(())
    }
}

/// Result returned after creating an agent.
/// 对齐 TS:
///   `Type.Object({
///      ok: Type.Literal(true),
///      agentId: NonEmptyString,
///      name: NonEmptyString,
///      workspace: NonEmptyString,
///      model: Type.Optional(NonEmptyString),
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentsCreateResultSchema {
    pub ok: bool,
    pub agent_id: String,
    pub name: String,
    pub workspace: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
}

impl AgentsCreateResultSchema {
    pub fn validate(&self) -> Result<(), String> {
        if !self.ok {
            return Err("ok: expected literal true".to_string());
        }
        validate_non_empty_string("agentId", &self.agent_id)?;
        validate_non_empty_string("name", &self.name)?;
        validate_non_empty_string("workspace", &self.workspace)?;
        validate_optional_non_empty_string("model", self.model.as_deref())?;
        Ok(())
    }
}

/// Updates mutable agent identity, workspace, and model fields.
/// 对齐 TS:
///   `Type.Object({
///      agentId: NonEmptyString,
///      name: Type.Optional(NonEmptyString),
///      workspace: Type.Optional(NonEmptyString),
///      model: Type.Optional(NonEmptyString),
///      emoji: Type.Optional(Type.String()),
///      avatar: Type.Optional(Type.String()),
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentsUpdateParamsSchema {
    pub agent_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub workspace: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub emoji: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub avatar: Option<String>,
}

impl AgentsUpdateParamsSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_non_empty_string("agentId", &self.agent_id)?;
        validate_optional_non_empty_string("name", self.name.as_deref())?;
        validate_optional_non_empty_string("workspace", self.workspace.as_deref())?;
        validate_optional_non_empty_string("model", self.model.as_deref())?;
        Ok(())
    }
}

/// Result returned after updating an agent.
/// 对齐 TS: `Type.Object({ ok: Type.Literal(true), agentId: NonEmptyString }, ...)`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentsUpdateResultSchema {
    pub ok: bool,
    pub agent_id: String,
}

impl AgentsUpdateResultSchema {
    pub fn validate(&self) -> Result<(), String> {
        if !self.ok {
            return Err("ok: expected literal true".to_string());
        }
        validate_non_empty_string("agentId", &self.agent_id)?;
        Ok(())
    }
}

/// Deletes an agent and optionally its workspace/config files.
/// 对齐 TS:
///   `Type.Object({
///      agentId: NonEmptyString,
///      deleteFiles: Type.Optional(Type.Boolean()),
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentsDeleteParamsSchema {
    pub agent_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub delete_files: Option<bool>,
}

impl AgentsDeleteParamsSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_non_empty_string("agentId", &self.agent_id)?;
        Ok(())
    }
}

/// Result returned after deleting an agent and unbinding sessions.
/// 对齐 TS:
///   `Type.Object({
///      ok: Type.Literal(true),
///      agentId: NonEmptyString,
///      removedBindings: Type.Integer({ minimum: 0 }),
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentsDeleteResultSchema {
    pub ok: bool,
    pub agent_id: String,
    pub removed_bindings: i64,
}

impl AgentsDeleteResultSchema {
    pub fn validate(&self) -> Result<(), String> {
        if !self.ok {
            return Err("ok: expected literal true".to_string());
        }
        validate_non_empty_string("agentId", &self.agent_id)?;
        validate_non_negative_integer("removedBindings", self.removed_bindings)?;
        Ok(())
    }
}

// ============================================================================
// AgentsFileEntrySchema + AgentsFiles* schemas
// ============================================================================

/// File metadata and optional content for agent-local editable files.
/// 对齐 TS:
///   `Type.Object({
///      name: NonEmptyString,
///      path: NonEmptyString,
///      missing: Type.Boolean(),
///      size: Type.Optional(Type.Integer({ minimum: 0 })),
///      updatedAtMs: Type.Optional(Type.Integer({ minimum: 0 })),
///      content: Type.Optional(Type.String()),
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentsFileEntrySchema {
    pub name: String,
    pub path: String,
    pub missing: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub size: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub updated_at_ms: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
}

impl AgentsFileEntrySchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_non_empty_string("name", &self.name)?;
        validate_non_empty_string("path", &self.path)?;
        validate_optional_non_negative_integer("size", self.size)?;
        validate_optional_non_negative_integer("updatedAtMs", self.updated_at_ms)?;
        Ok(())
    }
}

/// Lists editable files for one agent.
/// 对齐 TS: `Type.Object({ agentId: NonEmptyString }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentsFilesListParamsSchema {
    pub agent_id: String,
}

impl AgentsFilesListParamsSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_non_empty_string("agentId", &self.agent_id)?;
        Ok(())
    }
}

/// Editable file list for an agent workspace.
/// 对齐 TS:
///   `Type.Object({
///      agentId: NonEmptyString,
///      workspace: NonEmptyString,
///      files: Type.Array(AgentsFileEntrySchema),
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentsFilesListResultSchema {
    pub agent_id: String,
    pub workspace: String,
    pub files: Vec<AgentsFileEntrySchema>,
}

impl AgentsFilesListResultSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_non_empty_string("agentId", &self.agent_id)?;
        validate_non_empty_string("workspace", &self.workspace)?;
        for (i, f) in self.files.iter().enumerate() {
            f.validate().map_err(|e| format!("files[{}]: {}", i, e))?;
        }
        Ok(())
    }
}

/// Reads one editable agent file by name.
/// 对齐 TS: `Type.Object({ agentId: NonEmptyString, name: NonEmptyString }, ...)`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentsFilesGetParamsSchema {
    pub agent_id: String,
    pub name: String,
}

impl AgentsFilesGetParamsSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_non_empty_string("agentId", &self.agent_id)?;
        validate_non_empty_string("name", &self.name)?;
        Ok(())
    }
}

/// Result for reading one editable agent file.
/// 对齐 TS:
///   `Type.Object({
///      agentId: NonEmptyString,
///      workspace: NonEmptyString,
///      file: AgentsFileEntrySchema,
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentsFilesGetResultSchema {
    pub agent_id: String,
    pub workspace: String,
    pub file: AgentsFileEntrySchema,
}

impl AgentsFilesGetResultSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_non_empty_string("agentId", &self.agent_id)?;
        validate_non_empty_string("workspace", &self.workspace)?;
        self.file.validate().map_err(|e| format!("file: {}", e))?;
        Ok(())
    }
}

/// Writes one editable agent file.
/// 对齐 TS: `Type.Object({ agentId, name, content }, ...)`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentsFilesSetParamsSchema {
    pub agent_id: String,
    pub name: String,
    pub content: String,
}

impl AgentsFilesSetParamsSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_non_empty_string("agentId", &self.agent_id)?;
        validate_non_empty_string("name", &self.name)?;
        Ok(())
    }
}

/// Result returned after writing an editable agent file.
/// 对齐 TS:
///   `Type.Object({
///      ok: Type.Literal(true),
///      agentId: NonEmptyString,
///      workspace: NonEmptyString,
///      file: AgentsFileEntrySchema,
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentsFilesSetResultSchema {
    pub ok: bool,
    pub agent_id: String,
    pub workspace: String,
    pub file: AgentsFileEntrySchema,
}

impl AgentsFilesSetResultSchema {
    pub fn validate(&self) -> Result<(), String> {
        if !self.ok {
            return Err("ok: expected literal true".to_string());
        }
        validate_non_empty_string("agentId", &self.agent_id)?;
        validate_non_empty_string("workspace", &self.workspace)?;
        self.file.validate().map_err(|e| format!("file: {}", e))?;
        Ok(())
    }
}

// ============================================================================
// ModelsList — view enum
// ============================================================================

/// Visibility scope for the model catalog request.
/// 对齐 TS:
///   `Type.Union([
///      Type.Literal("default"),
///      Type.Literal("configured"),
///      Type.Literal("provider-config"),
///      Type.Literal("all"),
///   ])`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ModelsListView {
    Default,
    Configured,
    #[serde(rename = "provider-config")]
    ProviderConfig,
    All,
}

impl ModelsListView {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Default => "default",
            Self::Configured => "configured",
            Self::ProviderConfig => "provider-config",
            Self::All => "all",
        }
    }
}

pub fn is_valid_models_list_view(s: &str) -> bool {
    matches!(s, "default" | "configured" | "provider-config" | "all")
}

/// Model catalog request with optional visibility scope.
/// 对齐 TS: `Type.Object({ view: Type.Optional(...) }, ...)`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelsListParamsSchema {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub view: Option<ModelsListView>,
}

impl ModelsListParamsSchema {
    pub fn validate(&self) -> Result<(), String> {
        Ok(())
    }
}

/// Model catalog result.
/// 对齐 TS: `Type.Object({ models: Type.Array(ModelChoiceSchema) }, ...)`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelsListResultSchema {
    pub models: Vec<ModelChoiceSchema>,
}

impl ModelsListResultSchema {
    pub fn validate(&self) -> Result<(), String> {
        for (i, m) in self.models.iter().enumerate() {
            m.validate().map_err(|e| format!("models[{}]: {}", i, e))?;
        }
        Ok(())
    }
}

// ============================================================================
// SkillsStatus / SkillsBins
// ============================================================================

/// Reads installed skill status, optionally for a selected agent.
/// 对齐 TS: `Type.Object({ agentId: Type.Optional(NonEmptyString) }, ...)`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillsStatusParamsSchema {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub agent_id: Option<String>,
}

impl SkillsStatusParamsSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_optional_non_empty_string("agentId", self.agent_id.as_deref())?;
        Ok(())
    }
}

/// Empty request payload for listing available skill bins.
/// 对齐 TS: `Type.Object({}, { additionalProperties: false })`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillsBinsParamsSchema {}

impl SkillsBinsParamsSchema {
    pub fn validate(&self) -> Result<(), String> {
        Ok(())
    }
}

/// Skill bin names available to the gateway.
/// 对齐 TS: `Type.Object({ bins: Type.Array(NonEmptyString) }, ...)`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillsBinsResultSchema {
    pub bins: Vec<String>,
}

impl SkillsBinsResultSchema {
    pub fn validate(&self) -> Result<(), String> {
        for (i, b) in self.bins.iter().enumerate() {
            if !is_non_empty_string(b) {
                return Err(format!("bins[{}]: expected non-empty string, got {:?}", i, b));
            }
        }
        Ok(())
    }
}

// ============================================================================
// SkillsUpload — Begin / Chunk / Commit
// ============================================================================

/// SHA-256 hex hash string (64 chars).
/// 对齐 TS: `Type.String({ minLength: 64, maxLength: 64, pattern: "^[a-fA-F0-9]{64}$" })`.
fn is_valid_sha256_string(s: &str) -> bool {
    if s.len() != 64 {
        return false;
    }
    regex(SHA256_PATTERN).is_match(s)
}

fn validate_sha256(field: &str, value: &str) -> Result<(), String> {
    if is_valid_sha256_string(value) {
        Ok(())
    } else {
        Err(format!(
            "{}: expected 64-char hex SHA-256, got {:?}",
            field, value
        ))
    }
}

fn validate_optional_sha256(field: &str, value: Option<&String>) -> Result<(), String> {
    if let Some(s) = value {
        validate_sha256(field, s)?;
    }
    Ok(())
}

/// Starts a chunked skill archive upload.
/// 对齐 TS:
///   `Type.Object({
///      kind: Type.Literal("skill-archive"),
///      slug: NonEmptyString,
///      sizeBytes: Type.Integer({ minimum: 1 }),
///      sha256: Type.Optional(Sha256String),
///      force: Type.Optional(Type.Boolean()),
///      idempotencyKey: Type.Optional(SkillUploadIdempotencyKeyString),
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillsUploadBeginParamsSchema {
    pub kind: SkillsUploadKind,
    pub slug: String,
    pub size_bytes: i64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sha256: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub force: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub idempotency_key: Option<String>,
}

/// Literal marker for `SkillsUploadBeginParamsSchema.kind`.
/// 对齐 TS: `Type.Literal("skill-archive")`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SkillsUploadKind {
    #[serde(rename = "skill-archive")]
    SkillArchive,
}

impl SkillsUploadKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::SkillArchive => "skill-archive",
        }
    }
}

impl SkillsUploadBeginParamsSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_non_empty_string("slug", &self.slug)?;
        validate_positive_integer("sizeBytes", self.size_bytes)?;
        validate_optional_sha256("sha256", self.sha256.as_ref())?;
        validate_optional_string_length_range(
            "idempotencyKey",
            self.idempotency_key.as_deref(),
            SKILL_UPLOAD_IDEMPOTENCY_KEY_MIN,
            SKILL_UPLOAD_IDEMPOTENCY_KEY_MAX,
        )?;
        Ok(())
    }
}

/// Uploads one base64-encoded chunk for a skill archive.
/// 对齐 TS:
///   `Type.Object({
///      uploadId: NonEmptyString,
///      offset: Type.Integer({ minimum: 0 }),
///      dataBase64: SkillUploadDataBase64String,
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillsUploadChunkParamsSchema {
    pub upload_id: String,
    pub offset: i64,
    pub data_base64: String,
}

impl SkillsUploadChunkParamsSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_non_empty_string("uploadId", &self.upload_id)?;
        validate_non_negative_integer("offset", self.offset)?;
        validate_string_length_range(
            "dataBase64",
            &self.data_base64,
            SKILL_UPLOAD_DATA_BASE64_MIN,
            SKILL_UPLOAD_DATA_BASE64_MAX,
        )?;
        Ok(())
    }
}

/// Commits a completed skill archive upload.
/// 对齐 TS: `Type.Object({ uploadId: NonEmptyString, sha256: Type.Optional(Sha256String) }, ...)`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillsUploadCommitParamsSchema {
    pub upload_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sha256: Option<String>,
}

impl SkillsUploadCommitParamsSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_non_empty_string("uploadId", &self.upload_id)?;
        validate_optional_sha256("sha256", self.sha256.as_ref())?;
        Ok(())
    }
}

// ============================================================================
// SkillsInstall — discriminated union (3 variants)
// ============================================================================

/// Skill install source discriminator.
/// 对齐 TS: `Type.Literal("clawhub")` / `Type.Literal("upload")`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SkillsInstallSource {
    Clawhub,
    Upload,
}

impl SkillsInstallSource {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Clawhub => "clawhub",
            Self::Upload => "upload",
        }
    }
}

/// Variant 1: install by legacy install id.
/// 对齐 TS:
///   `Type.Object({
///      agentId: Type.Optional(NonEmptyString),
///      name: NonEmptyString,
///      installId: NonEmptyString,
///      dangerouslyForceUnsafeInstall: Type.Optional(Type.Boolean()),
///      timeoutMs: Type.Optional(Type.Integer({ minimum: 1000 })),
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillsInstallByInstallIdParamsSchema {
    pub agent_id: Option<String>,
    pub name: String,
    pub install_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dangerously_force_unsafe_install: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub timeout_ms: Option<i64>,
}

impl SkillsInstallByInstallIdParamsSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_optional_non_empty_string("agentId", self.agent_id.as_deref())?;
        validate_non_empty_string("name", &self.name)?;
        validate_non_empty_string("installId", &self.install_id)?;
        if let Some(ms) = self.timeout_ms {
            if ms < SKILL_INSTALL_TIMEOUT_MIN_MS {
                return Err(format!(
                    "timeoutMs: expected integer >= {}, got {}",
                    SKILL_INSTALL_TIMEOUT_MIN_MS, ms
                ));
            }
        }
        Ok(())
    }
}

/// Variant 2: install from ClawHub.
/// 对齐 TS:
///   `Type.Object({
///      agentId: Type.Optional(NonEmptyString),
///      source: Type.Literal("clawhub"),
///      slug: NonEmptyString,
///      version: Type.Optional(NonEmptyString),
///      force: Type.Optional(Type.Boolean()),
///      acknowledgeClawHubRisk: Type.Optional(Type.Boolean()),
///      timeoutMs: Type.Optional(Type.Integer({ minimum: 1000 })),
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillsInstallClawhubParamsSchema {
    pub agent_id: Option<String>,
    pub source: SkillsInstallSourceClawhub,
    pub slug: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub force: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub acknowledge_claw_hub_risk: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub timeout_ms: Option<i64>,
}

impl SkillsInstallClawhubParamsSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_optional_non_empty_string("agentId", self.agent_id.as_deref())?;
        validate_non_empty_string("slug", &self.slug)?;
        if let Some(ms) = self.timeout_ms {
            if ms < SKILL_INSTALL_TIMEOUT_MIN_MS {
                return Err(format!(
                    "timeoutMs: expected integer >= {}, got {}",
                    SKILL_INSTALL_TIMEOUT_MIN_MS, ms
                ));
            }
        }
        Ok(())
    }
}

/// Literal marker for SkillsInstallClawhubParamsSchema.source.
/// 对齐 TS: `Type.Literal("clawhub")`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SkillsInstallSourceClawhub {
    #[serde(rename = "clawhub")]
    Clawhub,
}

impl SkillsInstallSourceClawhub {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Clawhub => "clawhub",
        }
    }
}

/// Variant 3: install from uploaded archive.
/// 对齐 TS:
///   `Type.Object({
///      agentId: Type.Optional(NonEmptyString),
///      source: Type.Literal("upload"),
///      uploadId: NonEmptyString,
///      slug: NonEmptyString,
///      force: Type.Optional(Type.Boolean()),
///      sha256: Type.Optional(Sha256String),
///      timeoutMs: Type.Optional(Type.Integer({ minimum: 1000 })),
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillsInstallUploadParamsSchema {
    pub agent_id: Option<String>,
    pub source: SkillsInstallSourceUpload,
    pub upload_id: String,
    pub slug: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub force: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sha256: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub timeout_ms: Option<i64>,
}

impl SkillsInstallUploadParamsSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_optional_non_empty_string("agentId", self.agent_id.as_deref())?;
        validate_non_empty_string("uploadId", &self.upload_id)?;
        validate_non_empty_string("slug", &self.slug)?;
        validate_optional_sha256("sha256", self.sha256.as_ref())?;
        if let Some(ms) = self.timeout_ms {
            if ms < SKILL_INSTALL_TIMEOUT_MIN_MS {
                return Err(format!(
                    "timeoutMs: expected integer >= {}, got {}",
                    SKILL_INSTALL_TIMEOUT_MIN_MS, ms
                ));
            }
        }
        Ok(())
    }
}

/// Literal marker for SkillsInstallUploadParamsSchema.source.
/// 对齐 TS: `Type.Literal("upload")`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SkillsInstallSourceUpload {
    #[serde(rename = "upload")]
    Upload,
}

impl SkillsInstallSourceUpload {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Upload => "upload",
        }
    }
}

/// Discriminated union for skill install requests.
/// 对齐 TS: `Type.Union([byInstallId, clawhub, upload])`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum SkillsInstallParamsSchema {
    ByInstallId(SkillsInstallByInstallIdParamsSchema),
    Clawhub(SkillsInstallClawhubParamsSchema),
    Upload(SkillsInstallUploadParamsSchema),
}

impl SkillsInstallParamsSchema {
    pub fn validate(&self) -> Result<(), String> {
        match self {
            Self::ByInstallId(v) => v.validate(),
            Self::Clawhub(v) => v.validate(),
            Self::Upload(v) => v.validate(),
        }
    }
}

// ============================================================================
// SkillsUpdate — discriminated union
// ============================================================================

/// Variant 1: update installed skill settings by key.
/// 对齐 TS:
///   `Type.Object({
///      skillKey: NonEmptyString,
///      enabled: Type.Optional(Type.Boolean()),
///      apiKey: Type.Optional(Type.String()),
///      env: Type.Optional(Type.Record(NonEmptyString, Type.String())),
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillsUpdateByKeyParamsSchema {
    pub skill_key: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub api_key: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub env: Option<std::collections::BTreeMap<String, String>>,
}

impl SkillsUpdateByKeyParamsSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_non_empty_string("skillKey", &self.skill_key)?;
        Ok(())
    }
}

/// Variant 2: refresh ClawHub-installed skills.
/// 对齐 TS:
///   `Type.Object({
///      agentId: Type.Optional(NonEmptyString),
///      source: Type.Literal("clawhub"),
///      slug: Type.Optional(NonEmptyString),
///      all: Type.Optional(Type.Boolean()),
///      acknowledgeClawHubRisk: Type.Optional(Type.Boolean()),
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillsUpdateClawhubParamsSchema {
    pub agent_id: Option<String>,
    pub source: SkillsInstallSourceClawhub,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub slug: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub all: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub acknowledge_claw_hub_risk: Option<bool>,
}

impl SkillsUpdateClawhubParamsSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_optional_non_empty_string("agentId", self.agent_id.as_deref())?;
        validate_optional_non_empty_string("slug", self.slug.as_deref())?;
        Ok(())
    }
}

/// Discriminated union for skill update requests.
/// 对齐 TS: `Type.Union([byKey, clawhub])`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum SkillsUpdateParamsSchema {
    ByKey(SkillsUpdateByKeyParamsSchema),
    Clawhub(SkillsUpdateClawhubParamsSchema),
}

impl SkillsUpdateParamsSchema {
    pub fn validate(&self) -> Result<(), String> {
        match self {
            Self::ByKey(v) => v.validate(),
            Self::Clawhub(v) => v.validate(),
        }
    }
}

// ============================================================================
// SkillsSearch
// ============================================================================

/// Searches the skill registry.
/// 对齐 TS:
///   `Type.Object({
///      query: Type.Optional(NonEmptyString),
///      limit: Type.Optional(Type.Integer({ minimum: 1, maximum: 100 })),
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillsSearchParamsSchema {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub query: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub limit: Option<i64>,
}

impl SkillsSearchParamsSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_optional_non_empty_string("query", self.query.as_deref())?;
        if let Some(n) = self.limit {
            validate_integer_in_range("limit", n, SKILL_SEARCH_LIMIT_MIN, SKILL_SEARCH_LIMIT_MAX)?;
        }
        Ok(())
    }
}

/// Ranked skill registry search result entry.
/// 对齐 TS:
///   `Type.Object({
///      score: Type.Number(),
///      slug: NonEmptyString,
///      displayName: NonEmptyString,
///      summary: Type.Optional(Type.String()),
///      version: Type.Optional(NonEmptyString),
///      updatedAt: Type.Optional(Type.Integer()),
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillsSearchResultEntrySchema {
    pub score: f64,
    pub slug: String,
    pub display_name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<i64>,
}

impl SkillsSearchResultEntrySchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_non_empty_string("slug", &self.slug)?;
        validate_non_empty_string("displayName", &self.display_name)?;
        Ok(())
    }
}

/// Ranked skill registry search results.
/// 对齐 TS: `Type.Object({ results: Type.Array(SkillsSearchResultEntrySchema) }, ...)`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillsSearchResultSchema {
    pub results: Vec<SkillsSearchResultEntrySchema>,
}

impl SkillsSearchResultSchema {
    pub fn validate(&self) -> Result<(), String> {
        for (i, r) in self.results.iter().enumerate() {
            r.validate().map_err(|e| format!("results[{}]: {}", i, e))?;
        }
        Ok(())
    }
}

// ============================================================================
// SkillsDetail
// ============================================================================

/// Reads registry detail for one skill slug.
/// 对齐 TS: `Type.Object({ slug: NonEmptyString }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillsDetailParamsSchema {
    pub slug: String,
}

impl SkillsDetailParamsSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_non_empty_string("slug", &self.slug)?;
        Ok(())
    }
}

// ----- SkillDetail nested blocks (all `Type.Union([Object, Null])`) -----

/// 对齐 TS: `Type.Object({ slug, displayName, summary, tags, channel, isOfficial, createdAt, updatedAt }, ...)`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillDetailRecordSchema {
    pub slug: String,
    pub display_name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tags: Option<std::collections::BTreeMap<String, String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub channel: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub is_official: Option<bool>,
    pub created_at: i64,
    pub updated_at: i64,
}

/// 对齐 TS: `Type.Object({ version, createdAt, changelog }, ...)`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillDetailLatestVersionSchema {
    pub version: String,
    pub created_at: i64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub changelog: Option<String>,
}

/// 对齐 TS: `Type.Object({ os, systems }, ...)`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillDetailMetadataSchema {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub os: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub systems: Option<Vec<String>>,
}

/// 对齐 TS: `Type.Object({ handle, displayName, image, official, channel, isOfficial }, ...)`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillDetailOwnerSchema {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub handle: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub image: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub official: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub channel: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub is_official: Option<bool>,
}

/// Skill registry detail, latest version, metadata, and owner info.
/// 对齐 TS: full `SkillsDetailResultSchema` Object.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillsDetailResultSchema {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub skill: Option<SkillDetailRecordSchema>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub latest_version: Option<SkillDetailLatestVersionSchema>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata: Option<SkillDetailMetadataSchema>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub owner: Option<SkillDetailOwnerSchema>,
}

impl SkillsDetailResultSchema {
    pub fn validate(&self) -> Result<(), String> {
        Ok(())
    }
}

// ============================================================================
// SkillsSecurityVerdicts
// ============================================================================

/// Reads current security verdicts for configured skills.
/// 对齐 TS: `Type.Object({ agentId: Type.Optional(NonEmptyString) }, ...)`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillsSecurityVerdictsParamsSchema {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub agent_id: Option<String>,
}

impl SkillsSecurityVerdictsParamsSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_optional_non_empty_string("agentId", self.agent_id.as_deref())?;
        Ok(())
    }
}

/// Optional error block embedded in a security verdict item.
/// 对齐 TS: `Type.Object({ code, message }, { additionalProperties: false })`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillsSecurityVerdictItemErrorSchema {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

/// One entry in a security verdict report.
/// 对齐 TS: `Type.Object({ registry, ok, decision, reasons, requestedSlug, ... }, ...)`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillsSecurityVerdictItemSchema {
    pub registry: String,
    pub ok: bool,
    pub decision: String,
    pub reasons: Vec<String>,
    pub requested_slug: String,
    pub requested_version: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub slug: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub publisher_handle: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub publisher_display_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created_at: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub checked_at: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub skill_url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub security_audit_url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub security_status: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub security_passed: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<SkillsSecurityVerdictItemErrorSchema>,
}

impl SkillsSecurityVerdictItemSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_non_empty_string("registry", &self.registry)?;
        validate_non_empty_string("decision", &self.decision)?;
        validate_non_empty_string("requestedSlug", &self.requested_slug)?;
        validate_non_empty_string("requestedVersion", &self.requested_version)?;
        Ok(())
    }
}

/// Security verdict report for installed/requested skills.
/// 对齐 TS:
///   `Type.Object({
///      schema: Type.Literal("openclaw.skills.security-verdicts.v1"),
///      items: Type.Array(SkillsSecurityVerdictItemSchema),
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillsSecurityVerdictsResultSchema {
    pub schema: SecurityVerdictsSchemaMarker,
    pub items: Vec<SkillsSecurityVerdictItemSchema>,
}

/// Literal marker for `SkillsSecurityVerdictsResultSchema.schema`.
/// 对齐 TS: `Type.Literal("openclaw.skills.security-verdicts.v1")`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SecurityVerdictsSchemaMarker {
    #[serde(rename = "openclaw.skills.security-verdicts.v1")]
    OpenclawSkillsSecurityVerdictsV1,
}

impl SecurityVerdictsSchemaMarker {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::OpenclawSkillsSecurityVerdictsV1 => "openclaw.skills.security-verdicts.v1",
        }
    }
}

impl SkillsSecurityVerdictsResultSchema {
    pub fn validate(&self) -> Result<(), String> {
        for (i, item) in self.items.iter().enumerate() {
            item.validate().map_err(|e| format!("items[{}]: {}", i, e))?;
        }
        Ok(())
    }
}

// ============================================================================
// SkillsSkillCard
// ============================================================================

/// Reads the rendered skill card for one installed skill.
/// 对齐 TS: `Type.Object({ agentId: Type.Optional(...), skillKey: NonEmptyString }, ...)`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillsSkillCardParamsSchema {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub agent_id: Option<String>,
    pub skill_key: String,
}

impl SkillsSkillCardParamsSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_optional_non_empty_string("agentId", self.agent_id.as_deref())?;
        validate_non_empty_string("skillKey", &self.skill_key)?;
        Ok(())
    }
}

/// Rendered skill card content and file metadata.
/// 对齐 TS:
///   `Type.Object({
///      schema: Type.Literal("openclaw.skills.skill-card.v1"),
///      skillKey: NonEmptyString,
///      path: NonEmptyString,
///      sizeBytes: Type.Integer({ minimum: 0 }),
///      content: Type.String(),
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillsSkillCardResultSchema {
    pub schema: SkillCardSchemaMarker,
    pub skill_key: String,
    pub path: String,
    pub size_bytes: i64,
    pub content: String,
}

/// Literal marker for `SkillsSkillCardResultSchema.schema`.
/// 对齐 TS: `Type.Literal("openclaw.skills.skill-card.v1")`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SkillCardSchemaMarker {
    #[serde(rename = "openclaw.skills.skill-card.v1")]
    OpenclawSkillsSkillCardV1,
}

impl SkillCardSchemaMarker {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::OpenclawSkillsSkillCardV1 => "openclaw.skills.skill-card.v1",
        }
    }
}

impl SkillsSkillCardResultSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_non_empty_string("skillKey", &self.skill_key)?;
        validate_non_empty_string("path", &self.path)?;
        validate_non_negative_integer("sizeBytes", self.size_bytes)?;
        Ok(())
    }
}

// ============================================================================
// SkillProposal nested enums and blocks
// ============================================================================

/// Status discriminator for a skill proposal record.
/// 对齐 TS:
///   `Type.Union([
///      Type.Literal("pending"),
///      Type.Literal("applied"),
///      Type.Literal("rejected"),
///      Type.Literal("quarantined"),
///      Type.Literal("stale"),
///   ])`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SkillProposalStatus {
    Pending,
    Applied,
    Rejected,
    Quarantined,
    Stale,
}

impl SkillProposalStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Applied => "applied",
            Self::Rejected => "rejected",
            Self::Quarantined => "quarantined",
            Self::Stale => "stale",
        }
    }
}

/// Kind discriminator for a skill proposal.
/// 对齐 TS: `Type.Union([Type.Literal("create"), Type.Literal("update")])`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SkillProposalKind {
    Create,
    Update,
}

impl SkillProposalKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Create => "create",
            Self::Update => "update",
        }
    }
}

/// Scan state discriminator for proposal content.
/// 对齐 TS:
///   `Type.Union([
///      Type.Literal("pending"),
///      Type.Literal("clean"),
///      Type.Literal("failed"),
///      Type.Literal("quarantined"),
///   ])`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SkillProposalScanState {
    Pending,
    Clean,
    Failed,
    Quarantined,
}

impl SkillProposalScanState {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Clean => "clean",
            Self::Failed => "failed",
            Self::Quarantined => "quarantined",
        }
    }
}

/// Source that created the skill proposal record.
/// 对齐 TS:
///   `Type.Union([
///      Type.Literal("skill-workshop"),
///      Type.Literal("cli"),
///      Type.Literal("gateway"),
///   ])`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SkillProposalSource {
    #[serde(rename = "skill-workshop")]
    SkillWorkshop,
    Cli,
    Gateway,
}

impl SkillProposalSource {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::SkillWorkshop => "skill-workshop",
            Self::Cli => "cli",
            Self::Gateway => "gateway",
        }
    }
}

/// Severity discriminator for a proposal finding.
/// 对齐 TS: `Type.Union([Type.Literal("info"), Type.Literal("warn"), Type.Literal("critical")])`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SkillProposalFindingSeverity {
    Info,
    Warn,
    Critical,
}

impl SkillProposalFindingSeverity {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Info => "info",
            Self::Warn => "warn",
            Self::Critical => "critical",
        }
    }
}

// ----- SkillProposal support blocks -----

/// Support file payload accepted from proposal create/revise requests.
/// 对齐 TS:
///   `Type.Object({
///      path: NonEmptyString,
///      content: Type.String({ maxLength: 262_144 }),
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillProposalSupportFileInputSchema {
    pub path: String,
    pub content: String,
}

impl SkillProposalSupportFileInputSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_non_empty_string("path", &self.path)?;
        validate_optional_string_length_range(
            "content",
            Some(&self.content),
            0,
            SKILL_PROPOSAL_SUPPORT_FILE_SIZE_MAX as usize,
        )?;
        Ok(())
    }
}

/// Stored support file metadata, including target conflict hashes for updates.
/// 对齐 TS:
///   `Type.Object({
///      path: NonEmptyString,
///      sizeBytes: Type.Integer({ minimum: 0, maximum: 262_144 }),
///      hash: Sha256String,
///      targetExisted: Type.Optional(Type.Boolean()),
///      targetContentHash: Type.Optional(Sha256String),
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillProposalSupportFileSchema {
    pub path: String,
    pub size_bytes: i64,
    pub hash: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target_existed: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target_content_hash: Option<String>,
}

impl SkillProposalSupportFileSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_non_empty_string("path", &self.path)?;
        if self.size_bytes < 0 || self.size_bytes > SKILL_PROPOSAL_SUPPORT_FILE_SIZE_MAX {
            return Err(format!(
                "sizeBytes: expected 0..={}, got {}",
                SKILL_PROPOSAL_SUPPORT_FILE_SIZE_MAX, self.size_bytes
            ));
        }
        validate_sha256("hash", &self.hash)?;
        validate_optional_sha256("targetContentHash", self.target_content_hash.as_ref())?;
        Ok(())
    }
}

/// One static-scan finding against proposed skill content.
/// 对齐 TS:
///   `Type.Object({
///      ruleId: NonEmptyString,
///      severity: Type.Union([Type.Literal("info"), Type.Literal("warn"), Type.Literal("critical")]),
///      file: NonEmptyString,
///      line: Type.Integer({ minimum: 1 }),
///      message: NonEmptyString,
///      evidence: Type.String(),
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillProposalFindingSchema {
    pub rule_id: String,
    pub severity: SkillProposalFindingSeverity,
    pub file: String,
    pub line: i64,
    pub message: String,
    pub evidence: String,
}

impl SkillProposalFindingSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_non_empty_string("ruleId", &self.rule_id)?;
        validate_non_empty_string("file", &self.file)?;
        validate_positive_integer("line", self.line)?;
        validate_non_empty_string("message", &self.message)?;
        Ok(())
    }
}

/// Aggregated scan report attached to a proposal record.
/// 对齐 TS:
///   `Type.Object({
///      state: SkillProposalScanStateSchema,
///      scannedAt: NonEmptyString,
///      critical: Type.Integer({ minimum: 0 }),
///      warn: Type.Integer({ minimum: 0 }),
///      info: Type.Integer({ minimum: 0 }),
///      findings: Type.Array(SkillProposalFindingSchema),
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillProposalScanSchema {
    pub state: SkillProposalScanState,
    pub scanned_at: String,
    pub critical: i64,
    pub warn: i64,
    pub info: i64,
    pub findings: Vec<SkillProposalFindingSchema>,
}

impl SkillProposalScanSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_non_empty_string("scannedAt", &self.scanned_at)?;
        validate_non_negative_integer("critical", self.critical)?;
        validate_non_negative_integer("warn", self.warn)?;
        validate_non_negative_integer("info", self.info)?;
        for (i, f) in self.findings.iter().enumerate() {
            f.validate().map_err(|e| format!("findings[{}]: {}", i, e))?;
        }
        Ok(())
    }
}

/// Skill file target that a proposal creates or updates.
/// 对齐 TS:
///   `Type.Object({
///      skillName: NonEmptyString,
///      skillKey: NonEmptyString,
///      skillDir: NonEmptyString,
///      skillFile: NonEmptyString,
///      source: Type.Optional(NonEmptyString),
///      currentContentHash: Type.Optional(NonEmptyString),
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillProposalTargetSchema {
    pub skill_name: String,
    pub skill_key: String,
    pub skill_dir: String,
    pub skill_file: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub current_content_hash: Option<String>,
}

impl SkillProposalTargetSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_non_empty_string("skillName", &self.skill_name)?;
        validate_non_empty_string("skillKey", &self.skill_key)?;
        validate_non_empty_string("skillDir", &self.skill_dir)?;
        validate_non_empty_string("skillFile", &self.skill_file)?;
        validate_optional_non_empty_string("source", self.source.as_deref())?;
        Ok(())
    }
}

/// Optional runtime origin tying a proposal back to an agent turn.
/// 对齐 TS:
///   `Type.Object({
///      agentId: Type.Optional(NonEmptyString),
///      sessionKey: Type.Optional(NonEmptyString),
///      runId: Type.Optional(NonEmptyString),
///      messageId: Type.Optional(NonEmptyString),
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillProposalOriginSchema {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub agent_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub session_key: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub run_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message_id: Option<String>,
}

impl SkillProposalOriginSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_optional_non_empty_string("agentId", self.agent_id.as_deref())?;
        validate_optional_non_empty_string("sessionKey", self.session_key.as_deref())?;
        validate_optional_non_empty_string("runId", self.run_id.as_deref())?;
        validate_optional_non_empty_string("messageId", self.message_id.as_deref())?;
        Ok(())
    }
}

/// Literal marker for `SkillProposalRecordSchema.draftFile`.
/// 对齐 TS: `Type.Literal("PROPOSAL.md")`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SkillProposalDraftFile {
    #[serde(rename = "PROPOSAL.md")]
    ProposalMd,
}

impl SkillProposalDraftFile {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::ProposalMd => "PROPOSAL.md",
        }
    }
}

/// Literal marker for `SkillProposalRecordSchema.schema`.
/// 对齐 TS: `Type.Literal("openclaw.skill-workshop.proposal.v1")`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SkillProposalRecordSchemaMarker {
    #[serde(rename = "openclaw.skill-workshop.proposal.v1")]
    OpenclawSkillWorkshopProposalV1,
}

impl SkillProposalRecordSchemaMarker {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::OpenclawSkillWorkshopProposalV1 => "openclaw.skill-workshop.proposal.v1",
        }
    }
}

/// Full persisted skill proposal record.
/// 对齐 TS: full `SkillProposalRecordSchema` Object (re-exported by
/// `SkillsProposalRecordResultSchema`).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillProposalRecordSchema {
    pub schema: SkillProposalRecordSchemaMarker,
    pub id: String,
    pub kind: SkillProposalKind,
    pub status: SkillProposalStatus,
    pub title: String,
    pub description: String,
    pub created_at: String,
    pub updated_at: String,
    pub created_by: SkillProposalSource,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub origin: Option<SkillProposalOriginSchema>,
    pub proposed_version: String,
    pub draft_file: SkillProposalDraftFile,
    pub draft_hash: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub support_files: Option<Vec<SkillProposalSupportFileSchema>>,
    pub target: SkillProposalTargetSchema,
    pub scan: SkillProposalScanSchema,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub goal: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub evidence: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub applied_at: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rejected_at: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub quarantined_at: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stale_at: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status_reason: Option<String>,
}

impl SkillProposalRecordSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_non_empty_string("id", &self.id)?;
        validate_non_empty_string("title", &self.title)?;
        validate_non_empty_string("description", &self.description)?;
        validate_non_empty_string("createdAt", &self.created_at)?;
        validate_non_empty_string("updatedAt", &self.updated_at)?;
        validate_non_empty_string("proposedVersion", &self.proposed_version)?;
        validate_non_empty_string("draftHash", &self.draft_hash)?;
        self.target.validate().map_err(|e| format!("target: {}", e))?;
        self.scan.validate().map_err(|e| format!("scan: {}", e))?;
        if let Some(origin) = &self.origin {
            origin
                .validate()
                .map_err(|e| format!("origin: {}", e))?;
        }
        if let Some(files) = &self.support_files {
            if files.len() > SKILL_PROPOSAL_MAX_SUPPORT_FILES {
                return Err(format!(
                    "supportFiles: expected at most {} items, got {}",
                    SKILL_PROPOSAL_MAX_SUPPORT_FILES,
                    files.len()
                ));
            }
            for (i, f) in files.iter().enumerate() {
                f.validate().map_err(|e| format!("supportFiles[{}]: {}", i, e))?;
            }
        }
        Ok(())
    }
}

/// Condensed proposal manifest entry for list views.
/// 对齐 TS: `Type.Object({ id, kind, status, title, description, skillName,
///                         skillKey, createdAt, updatedAt, scanState }, ...)`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillProposalManifestEntrySchema {
    pub id: String,
    pub kind: SkillProposalKind,
    pub status: SkillProposalStatus,
    pub title: String,
    pub description: String,
    pub skill_name: String,
    pub skill_key: String,
    pub created_at: String,
    pub updated_at: String,
    pub scan_state: SkillProposalScanState,
}

impl SkillProposalManifestEntrySchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_non_empty_string("id", &self.id)?;
        validate_non_empty_string("title", &self.title)?;
        validate_non_empty_string("description", &self.description)?;
        validate_non_empty_string("skillName", &self.skill_name)?;
        validate_non_empty_string("skillKey", &self.skill_key)?;
        validate_non_empty_string("createdAt", &self.created_at)?;
        validate_non_empty_string("updatedAt", &self.updated_at)?;
        Ok(())
    }
}

/// Lists skill-workshop proposals for the selected agent scope.
/// 对齐 TS: `Type.Object({ agentId: Type.Optional(NonEmptyString) }, ...)`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillsProposalsListParamsSchema {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub agent_id: Option<String>,
}

impl SkillsProposalsListParamsSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_optional_non_empty_string("agentId", self.agent_id.as_deref())?;
        Ok(())
    }
}

/// Literal marker for `SkillsProposalsListResultSchema.schema`.
/// 对齐 TS: `Type.Literal("openclaw.skill-workshop.proposals-manifest.v1")`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SkillsProposalsManifestSchemaMarker {
    #[serde(rename = "openclaw.skill-workshop.proposals-manifest.v1")]
    OpenclawSkillWorkshopProposalsManifestV1,
}

impl SkillsProposalsManifestSchemaMarker {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::OpenclawSkillWorkshopProposalsManifestV1 => {
                "openclaw.skill-workshop.proposals-manifest.v1"
            }
        }
    }
}

/// Proposal manifest response for dashboard/workshop list views.
/// 对齐 TS:
///   `Type.Object({
///      schema: Type.Literal("openclaw.skill-workshop.proposals-manifest.v1"),
///      updatedAt: NonEmptyString,
///      proposals: Type.Array(SkillProposalManifestEntrySchema),
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillsProposalsListResultSchema {
    pub schema: SkillsProposalsManifestSchemaMarker,
    pub updated_at: String,
    pub proposals: Vec<SkillProposalManifestEntrySchema>,
}

impl SkillsProposalsListResultSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_non_empty_string("updatedAt", &self.updated_at)?;
        for (i, p) in self.proposals.iter().enumerate() {
            p.validate().map_err(|e| format!("proposals[{}]: {}", i, e))?;
        }
        Ok(())
    }
}

/// Reads a proposal record plus editable draft/support content.
/// 对齐 TS:
///   `Type.Object({
///      agentId: Type.Optional(NonEmptyString),
///      proposalId: NonEmptyString,
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillsProposalInspectParamsSchema {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub agent_id: Option<String>,
    pub proposal_id: String,
}

impl SkillsProposalInspectParamsSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_optional_non_empty_string("agentId", self.agent_id.as_deref())?;
        validate_non_empty_string("proposalId", &self.proposal_id)?;
        Ok(())
    }
}

/// Full proposal inspection result used before apply/revise decisions.
/// 对齐 TS:
///   `Type.Object({
///      record: SkillProposalRecordSchema,
///      content: Type.String(),
///      supportFiles: Type.Optional(Type.Array(SkillProposalSupportFileInputSchema, { maxItems: 64 })),
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillsProposalInspectResultSchema {
    pub record: SkillProposalRecordSchema,
    pub content: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub support_files: Option<Vec<SkillProposalSupportFileInputSchema>>,
}

impl SkillsProposalInspectResultSchema {
    pub fn validate(&self) -> Result<(), String> {
        self.record.validate().map_err(|e| format!("record: {}", e))?;
        if let Some(files) = &self.support_files {
            if files.len() > SKILL_PROPOSAL_MAX_SUPPORT_FILES {
                return Err(format!(
                    "supportFiles: expected at most {} items, got {}",
                    SKILL_PROPOSAL_MAX_SUPPORT_FILES,
                    files.len()
                ));
            }
            for (i, f) in files.iter().enumerate() {
                f.validate().map_err(|e| format!("supportFiles[{}]: {}", i, e))?;
            }
        }
        Ok(())
    }
}

/// Creates a proposal for a new skill.
/// 对齐 TS:
///   `Type.Object({
///      agentId: Type.Optional(NonEmptyString),
///      name: NonEmptyString,
///      description: NonEmptyString,
///      content: SkillProposalContentString,
///      supportFiles: Type.Optional(Type.Array(SkillProposalSupportFileInputSchema, { maxItems: 64 })),
///      goal: Type.Optional(Type.String()),
///      evidence: Type.Optional(Type.String()),
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillsProposalCreateParamsSchema {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub agent_id: Option<String>,
    pub name: String,
    pub description: String,
    pub content: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub support_files: Option<Vec<SkillProposalSupportFileInputSchema>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub goal: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub evidence: Option<String>,
}

impl SkillsProposalCreateParamsSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_optional_non_empty_string("agentId", self.agent_id.as_deref())?;
        validate_non_empty_string("name", &self.name)?;
        validate_non_empty_string("description", &self.description)?;
        validate_string_length_range(
            "content",
            &self.content,
            1,
            SKILL_PROPOSAL_CONTENT_MAX_LENGTH,
        )?;
        if let Some(files) = &self.support_files {
            if files.len() > SKILL_PROPOSAL_MAX_SUPPORT_FILES {
                return Err(format!(
                    "supportFiles: expected at most {} items, got {}",
                    SKILL_PROPOSAL_MAX_SUPPORT_FILES,
                    files.len()
                ));
            }
            for (i, f) in files.iter().enumerate() {
                f.validate().map_err(|e| format!("supportFiles[{}]: {}", i, e))?;
            }
        }
        Ok(())
    }
}

/// Creates a proposal to update an existing skill.
/// 对齐 TS:
///   `Type.Object({
///      agentId: Type.Optional(NonEmptyString),
///      skillName: NonEmptyString,
///      description: Type.Optional(NonEmptyString),
///      content: SkillProposalContentString,
///      supportFiles: Type.Optional(Type.Array(SkillProposalSupportFileInputSchema, { maxItems: 64 })),
///      goal: Type.Optional(Type.String()),
///      evidence: Type.Optional(Type.String()),
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillsProposalUpdateParamsSchema {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub agent_id: Option<String>,
    pub skill_name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub content: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub support_files: Option<Vec<SkillProposalSupportFileInputSchema>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub goal: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub evidence: Option<String>,
}

impl SkillsProposalUpdateParamsSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_optional_non_empty_string("agentId", self.agent_id.as_deref())?;
        validate_non_empty_string("skillName", &self.skill_name)?;
        validate_optional_non_empty_string("description", self.description.as_deref())?;
        validate_string_length_range(
            "content",
            &self.content,
            1,
            SKILL_PROPOSAL_CONTENT_MAX_LENGTH,
        )?;
        if let Some(files) = &self.support_files {
            if files.len() > SKILL_PROPOSAL_MAX_SUPPORT_FILES {
                return Err(format!(
                    "supportFiles: expected at most {} items, got {}",
                    SKILL_PROPOSAL_MAX_SUPPORT_FILES,
                    files.len()
                ));
            }
            for (i, f) in files.iter().enumerate() {
                f.validate().map_err(|e| format!("supportFiles[{}]: {}", i, e))?;
            }
        }
        Ok(())
    }
}

/// Replaces draft content/support files for an existing proposal.
/// 对齐 TS:
///   `Type.Object({
///      agentId: Type.Optional(NonEmptyString),
///      proposalId: NonEmptyString,
///      content: SkillProposalContentString,
///      supportFiles: Type.Optional(Type.Array(SkillProposalSupportFileInputSchema, { maxItems: 64 })),
///      description: Type.Optional(Type.String()),
///      goal: Type.Optional(Type.String()),
///      evidence: Type.Optional(Type.String()),
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillsProposalReviseParamsSchema {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub agent_id: Option<String>,
    pub proposal_id: String,
    pub content: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub support_files: Option<Vec<SkillProposalSupportFileInputSchema>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub goal: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub evidence: Option<String>,
}

impl SkillsProposalReviseParamsSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_optional_non_empty_string("agentId", self.agent_id.as_deref())?;
        validate_non_empty_string("proposalId", &self.proposal_id)?;
        validate_string_length_range(
            "content",
            &self.content,
            1,
            SKILL_PROPOSAL_CONTENT_MAX_LENGTH,
        )?;
        if let Some(files) = &self.support_files {
            if files.len() > SKILL_PROPOSAL_MAX_SUPPORT_FILES {
                return Err(format!(
                    "supportFiles: expected at most {} items, got {}",
                    SKILL_PROPOSAL_MAX_SUPPORT_FILES,
                    files.len()
                ));
            }
            for (i, f) in files.iter().enumerate() {
                f.validate().map_err(|e| format!("supportFiles[{}]: {}", i, e))?;
            }
        }
        Ok(())
    }
}

/// Starts an agent turn that revises a pending proposal from natural-language instructions.
/// 对齐 TS:
///   `Type.Object({
///      agentId: Type.Optional(NonEmptyString),
///      targetAgentId: Type.Optional(NonEmptyString),
///      proposalId: NonEmptyString,
///      instructions: Type.String({ minLength: 1, maxLength: 32_768 }),
///      sessionKey: NonEmptyString,
///      sessionId: Type.Optional(NonEmptyString),
///      idempotencyKey: NonEmptyString,
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillsProposalRequestRevisionParamsSchema {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub agent_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target_agent_id: Option<String>,
    pub proposal_id: String,
    pub instructions: String,
    pub session_key: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
    pub idempotency_key: String,
}

impl SkillsProposalRequestRevisionParamsSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_optional_non_empty_string("agentId", self.agent_id.as_deref())?;
        validate_optional_non_empty_string("targetAgentId", self.target_agent_id.as_deref())?;
        validate_non_empty_string("proposalId", &self.proposal_id)?;
        validate_string_length_range(
            "instructions",
            &self.instructions,
            1,
            SKILL_PROPOSAL_REVISION_INSTRUCTIONS_MAX_LENGTH,
        )?;
        validate_non_empty_string("sessionKey", &self.session_key)?;
        validate_optional_non_empty_string("sessionId", self.session_id.as_deref())?;
        validate_non_empty_string("idempotencyKey", &self.idempotency_key)?;
        Ok(())
    }
}

/// Status discriminator for `SkillsProposalRequestRevisionResultSchema.status`.
/// 对齐 TS:
///   `Type.Union([
///      Type.Literal("started"),
///      Type.Literal("in_flight"),
///      Type.Literal("ok"),
///      Type.Literal("timeout"),
///      Type.Literal("error"),
///   ])`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SkillsProposalRequestRevisionStatus {
    Started,
    #[serde(rename = "in_flight")]
    InFlight,
    Ok,
    Timeout,
    Error,
}

impl SkillsProposalRequestRevisionStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Started => "started",
            Self::InFlight => "in_flight",
            Self::Ok => "ok",
            Self::Timeout => "timeout",
            Self::Error => "error",
        }
    }
}

/// Chat-run acknowledgement returned after queueing a Skill Workshop revision request.
/// 对齐 TS: `Type.Object({ runId, status }, { additionalProperties: true })`.
///
/// 注意: TS 允许额外字段; Rust 通过 `#[serde(default)]` 收集未知字段较复杂,
/// 这里仅暴露规范字段. 调用方应避免依赖 additionalProperties.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillsProposalRequestRevisionResultSchema {
    pub run_id: String,
    pub status: SkillsProposalRequestRevisionStatus,
}

impl SkillsProposalRequestRevisionResultSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_non_empty_string("runId", &self.run_id)?;
        Ok(())
    }
}

/// Shared approve/reject/quarantine action payload for one proposal.
/// 对齐 TS:
///   `Type.Object({
///      agentId: Type.Optional(NonEmptyString),
///      proposalId: NonEmptyString,
///      reason: Type.Optional(Type.String()),
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillsProposalActionParamsSchema {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub agent_id: Option<String>,
    pub proposal_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

impl SkillsProposalActionParamsSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_optional_non_empty_string("agentId", self.agent_id.as_deref())?;
        validate_non_empty_string("proposalId", &self.proposal_id)?;
        Ok(())
    }
}

/// Result returned after applying a skill proposal to disk.
/// 对齐 TS:
///   `Type.Object({
///      record: SkillProposalRecordSchema,
///      targetSkillFile: NonEmptyString,
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillsProposalApplyResultSchema {
    pub record: SkillProposalRecordSchema,
    pub target_skill_file: String,
}

impl SkillsProposalApplyResultSchema {
    pub fn validate(&self) -> Result<(), String> {
        self.record.validate().map_err(|e| format!("record: {}", e))?;
        validate_non_empty_string("targetSkillFile", &self.target_skill_file)?;
        Ok(())
    }
}

/// Proposal record result returned after non-apply proposal actions.
/// 对齐 TS: `export const SkillsProposalRecordResultSchema = SkillProposalRecordSchema`.
pub type SkillsProposalRecordResultSchema = SkillProposalRecordSchema;

// ============================================================================
// SkillsCurator
// ============================================================================

/// Lifecycle state for a curated skill entry.
/// 对齐 TS: `Type.Union([Type.Literal("active"), Type.Literal("stale"), Type.Literal("archived")])`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SkillLifecycleState {
    Active,
    Stale,
    Archived,
}

impl SkillLifecycleState {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::Stale => "stale",
            Self::Archived => "archived",
        }
    }
}

/// One curated skill lifecycle entry.
/// 对齐 TS:
///   `Type.Object({
///      skillFile: NonEmptyString,
///      skillKey: NonEmptyString,
///      skillName: NonEmptyString,
///      state: SkillLifecycleStateSchema,
///      pinned: Type.Boolean(),
///      createdAtMs: Type.Number(),
///      stateChangedAtMs: Type.Number(),
///      lastUsedAtMs: Type.Union([Type.Number(), Type.Null()]),
///      useCount: Type.Number(),
///      archivedReason: Type.Union([Type.String(), Type.Null()]),
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillCuratorEntrySchema {
    pub skill_file: String,
    pub skill_key: String,
    pub skill_name: String,
    pub state: SkillLifecycleState,
    pub pinned: bool,
    pub created_at_ms: f64,
    pub state_changed_at_ms: f64,
    pub last_used_at_ms: Option<f64>,
    pub use_count: f64,
    pub archived_reason: Option<String>,
}

impl SkillCuratorEntrySchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_non_empty_string("skillFile", &self.skill_file)?;
        validate_non_empty_string("skillKey", &self.skill_key)?;
        validate_non_empty_string("skillName", &self.skill_name)?;
        Ok(())
    }
}

/// One overlap candidate reported by the curator.
/// 对齐 TS: `Type.Object({ left, right, score }, ...)`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillOverlapCandidateSchema {
    pub left: String,
    pub right: String,
    pub score: f64,
}

impl SkillOverlapCandidateSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_non_empty_string("left", &self.left)?;
        validate_non_empty_string("right", &self.right)?;
        Ok(())
    }
}

/// Reads persisted skill lifecycle curation state.
/// 对齐 TS: `Type.Object({}, { additionalProperties: false })`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillsCuratorStatusParamsSchema {}

impl SkillsCuratorStatusParamsSchema {
    pub fn validate(&self) -> Result<(), String> {
        Ok(())
    }
}

/// Skill curator status response.
/// 对齐 TS:
///   `Type.Object({
///      lastAttemptAtMs: Type.Union([Type.Number(), Type.Null()]),
///      lastSuccessAtMs: Type.Union([Type.Number(), Type.Null()]),
///      lastError: Type.Union([Type.String(), Type.Null()]),
///      counts: Type.Object({ active: Type.Number(), stale: Type.Number(), archived: Type.Number() }, ...),
///      skills: Type.Array(SkillCuratorEntrySchema),
///      overlaps: Type.Array(SkillOverlapCandidateSchema),
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillsCuratorStatusResultSchema {
    pub last_attempt_at_ms: Option<f64>,
    pub last_success_at_ms: Option<f64>,
    pub last_error: Option<String>,
    pub counts: SkillCuratorCountsSchema,
    pub skills: Vec<SkillCuratorEntrySchema>,
    pub overlaps: Vec<SkillOverlapCandidateSchema>,
}

/// Curator counts block.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillCuratorCountsSchema {
    pub active: f64,
    pub stale: f64,
    pub archived: f64,
}

impl SkillsCuratorStatusResultSchema {
    pub fn validate(&self) -> Result<(), String> {
        for (i, s) in self.skills.iter().enumerate() {
            s.validate().map_err(|e| format!("skills[{}]: {}", i, e))?;
        }
        for (i, o) in self.overlaps.iter().enumerate() {
            o.validate().map_err(|e| format!("overlaps[{}]: {}", i, e))?;
        }
        Ok(())
    }
}

/// Pins, unpins, or explicitly restores one curated skill.
/// 对齐 TS: `Type.Object({ skill: NonEmptyString }, ...)`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillsCuratorActionParamsSchema {
    pub skill: String,
}

impl SkillsCuratorActionParamsSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_non_empty_string("skill", &self.skill)?;
        Ok(())
    }
}

/// Curator action result reuses the entry shape.
/// 对齐 TS: `export const SkillsCuratorActionResultSchema = SkillCuratorEntrySchema`.
pub type SkillsCuratorActionResultSchema = SkillCuratorEntrySchema;

// ============================================================================
// ToolsCatalog — enums
// ============================================================================

/// Profile discriminator for `ToolCatalogProfileSchema`.
/// 对齐 TS:
///   `Type.Union([
///      Type.Literal("minimal"),
///      Type.Literal("coding"),
///      Type.Literal("messaging"),
///      Type.Literal("full"),
///   ])`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ToolCatalogProfileId {
    Minimal,
    Coding,
    Messaging,
    Full,
}

impl ToolCatalogProfileId {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Minimal => "minimal",
            Self::Coding => "coding",
            Self::Messaging => "messaging",
            Self::Full => "full",
        }
    }
}

/// Source discriminator for tools.
/// 对齐 TS: `Type.Union([Type.Literal("core"), Type.Literal("plugin")])`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ToolCatalogSource {
    Core,
    Plugin,
}

impl ToolCatalogSource {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Core => "core",
            Self::Plugin => "plugin",
        }
    }
}

/// Risk discriminator for tools.
/// 对齐 TS: `Type.Union([Type.Literal("low"), Type.Literal("medium"), Type.Literal("high")])`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ToolRisk {
    Low,
    Medium,
    High,
}

impl ToolRisk {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Low => "low",
            Self::Medium => "medium",
            Self::High => "high",
        }
    }
}

/// Reads the configured tool catalog for an agent.
/// 对齐 TS:
///   `Type.Object({
///      agentId: Type.Optional(NonEmptyString),
///      includePlugins: Type.Optional(Type.Boolean()),
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolsCatalogParamsSchema {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub agent_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub include_plugins: Option<bool>,
}

impl ToolsCatalogParamsSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_optional_non_empty_string("agentId", self.agent_id.as_deref())?;
        Ok(())
    }
}

/// Reads the effective tool set for one session.
/// 对齐 TS:
///   `Type.Object({
///      agentId: Type.Optional(NonEmptyString),
///      sessionKey: NonEmptyString,
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolsEffectiveParamsSchema {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub agent_id: Option<String>,
    pub session_key: String,
}

impl ToolsEffectiveParamsSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_optional_non_empty_string("agentId", self.agent_id.as_deref())?;
        validate_non_empty_string("sessionKey", &self.session_key)?;
        Ok(())
    }
}

/// Invokes one tool through the gateway tool dispatcher.
/// 对齐 TS:
///   `Type.Object({
///      name: NonEmptyString,
///      args: Type.Optional(Type.Record(Type.String(), Type.Unknown())),
///      sessionKey: Type.Optional(NonEmptyString),
///      agentId: Type.Optional(NonEmptyString),
///      confirm: Type.Optional(Type.Boolean()),
///      idempotencyKey: Type.Optional(NonEmptyString),
///      conversationReadOrigin: Type.Optional(Type.Literal("direct-operator")),
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolsInvokeParamsSchema {
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub args: Option<PluginJsonValueSchema>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub session_key: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub agent_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub confirm: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub idempotency_key: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub conversation_read_origin: Option<ToolsInvokeConversationReadOrigin>,
}

/// Literal marker for `ToolsInvokeParamsSchema.conversationReadOrigin`.
/// 对齐 TS: `Type.Literal("direct-operator")`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ToolsInvokeConversationReadOrigin {
    #[serde(rename = "direct-operator")]
    DirectOperator,
}

impl ToolsInvokeConversationReadOrigin {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::DirectOperator => "direct-operator",
        }
    }
}

impl ToolsInvokeParamsSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_non_empty_string("name", &self.name)?;
        validate_optional_non_empty_string("sessionKey", self.session_key.as_deref())?;
        validate_optional_non_empty_string("agentId", self.agent_id.as_deref())?;
        validate_optional_non_empty_string("idempotencyKey", self.idempotency_key.as_deref())?;
        Ok(())
    }
}

/// Tool profile shown in catalog views.
/// 对齐 TS:
///   `Type.Object({
///      id: Type.Union([...]),
///      label: NonEmptyString,
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolCatalogProfileSchema {
    pub id: ToolCatalogProfileId,
    pub label: String,
}

impl ToolCatalogProfileSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_non_empty_string("label", &self.label)?;
        Ok(())
    }
}

/// Tool catalog entry before session-specific filtering is applied.
/// 对齐 TS:
///   `Type.Object({
///      id: NonEmptyString,
///      label: NonEmptyString,
///      description: Type.String(),
///      source: Type.Union([...]),
///      pluginId: Type.Optional(NonEmptyString),
///      optional: Type.Optional(Type.Boolean()),
///      risk: Type.Optional(Type.Union([...])),
///      tags: Type.Optional(Type.Array(NonEmptyString)),
///      defaultProfiles: Type.Array(Type.Union([...])),
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolCatalogEntrySchema {
    pub id: String,
    pub label: String,
    pub description: String,
    pub source: ToolCatalogSource,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub plugin_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub optional: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub risk: Option<ToolRisk>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,
    pub default_profiles: Vec<ToolCatalogProfileId>,
}

impl ToolCatalogEntrySchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_non_empty_string("id", &self.id)?;
        validate_non_empty_string("label", &self.label)?;
        validate_optional_non_empty_string("pluginId", self.plugin_id.as_deref())?;
        validate_optional_non_empty_string_list("tags", self.tags.as_ref())?;
        Ok(())
    }
}

/// Group of related catalog tools from core or a plugin.
/// 对齐 TS:
///   `Type.Object({
///      id: NonEmptyString,
///      label: NonEmptyString,
///      source: Type.Union([...]),
///      pluginId: Type.Optional(NonEmptyString),
///      tools: Type.Array(ToolCatalogEntrySchema),
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolCatalogGroupSchema {
    pub id: String,
    pub label: String,
    pub source: ToolCatalogSource,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub plugin_id: Option<String>,
    pub tools: Vec<ToolCatalogEntrySchema>,
}

impl ToolCatalogGroupSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_non_empty_string("id", &self.id)?;
        validate_non_empty_string("label", &self.label)?;
        validate_optional_non_empty_string("pluginId", self.plugin_id.as_deref())?;
        for (i, t) in self.tools.iter().enumerate() {
            t.validate().map_err(|e| format!("tools[{}]: {}", i, e))?;
        }
        Ok(())
    }
}

/// Tool catalog result for agent configuration UI.
/// 对齐 TS:
///   `Type.Object({
///      agentId: NonEmptyString,
///      profiles: Type.Array(ToolCatalogProfileSchema),
///      groups: Type.Array(ToolCatalogGroupSchema),
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolsCatalogResultSchema {
    pub agent_id: String,
    pub profiles: Vec<ToolCatalogProfileSchema>,
    pub groups: Vec<ToolCatalogGroupSchema>,
}

impl ToolsCatalogResultSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_non_empty_string("agentId", &self.agent_id)?;
        for (i, p) in self.profiles.iter().enumerate() {
            p.validate().map_err(|e| format!("profiles[{}]: {}", i, e))?;
        }
        for (i, g) in self.groups.iter().enumerate() {
            g.validate().map_err(|e| format!("groups[{}]: {}", i, e))?;
        }
        Ok(())
    }
}

// ============================================================================
// ToolsEffective
// ============================================================================

/// Source discriminator for effective tools.
/// 对齐 TS:
///   `Type.Union([
///      Type.Literal("core"),
///      Type.Literal("plugin"),
///      Type.Literal("channel"),
///      Type.Literal("mcp"),
///   ])`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ToolsEffectiveSource {
    Core,
    Plugin,
    Channel,
    Mcp,
}

impl ToolsEffectiveSource {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Core => "core",
            Self::Plugin => "plugin",
            Self::Channel => "channel",
            Self::Mcp => "mcp",
        }
    }
}

/// Severity discriminator for effective-tool notices.
/// 对齐 TS: `Type.Union([Type.Literal("info"), Type.Literal("warning")])`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ToolsEffectiveNoticeSeverity {
    Info,
    Warning,
}

impl ToolsEffectiveNoticeSeverity {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Info => "info",
            Self::Warning => "warning",
        }
    }
}

/// Effective tool entry after session/profile/channel/plugin filtering.
/// 对齐 TS:
///   `Type.Object({
///      id: NonEmptyString,
///      label: NonEmptyString,
///      description: Type.String(),
///      rawDescription: Type.String(),
///      source: Type.Union([...]),
///      pluginId: Type.Optional(NonEmptyString),
///      channelId: Type.Optional(NonEmptyString),
///      risk: Type.Optional(Type.Union([...])),
///      tags: Type.Optional(Type.Array(NonEmptyString)),
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolsEffectiveEntrySchema {
    pub id: String,
    pub label: String,
    pub description: String,
    pub raw_description: String,
    pub source: ToolsEffectiveSource,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub plugin_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub channel_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub risk: Option<ToolRisk>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,
}

impl ToolsEffectiveEntrySchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_non_empty_string("id", &self.id)?;
        validate_non_empty_string("label", &self.label)?;
        validate_optional_non_empty_string("pluginId", self.plugin_id.as_deref())?;
        validate_optional_non_empty_string("channelId", self.channel_id.as_deref())?;
        validate_optional_non_empty_string_list("tags", self.tags.as_ref())?;
        Ok(())
    }
}

/// Effective tool group shown to runtime/session callers.
/// 对齐 TS:
///   `Type.Object({
///      id: Type.Union([...]),
///      label: NonEmptyString,
///      source: Type.Union([...]),
///      tools: Type.Array(ToolsEffectiveEntrySchema),
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolsEffectiveGroupSchema {
    pub id: ToolsEffectiveSource,
    pub label: String,
    pub source: ToolsEffectiveSource,
    pub tools: Vec<ToolsEffectiveEntrySchema>,
}

impl ToolsEffectiveGroupSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_non_empty_string("label", &self.label)?;
        for (i, t) in self.tools.iter().enumerate() {
            t.validate().map_err(|e| format!("tools[{}]: {}", i, e))?;
        }
        Ok(())
    }
}

/// Notice explaining runtime filtering such as quarantined tool schemas.
/// 对齐 TS:
///   `Type.Object({
///      id: NonEmptyString,
///      severity: Type.Union([Type.Literal("info"), Type.Literal("warning")]),
///      message: Type.String(),
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolsEffectiveNoticeSchema {
    pub id: String,
    pub severity: ToolsEffectiveNoticeSeverity,
    pub message: String,
}

impl ToolsEffectiveNoticeSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_non_empty_string("id", &self.id)?;
        Ok(())
    }
}

/// Effective tool set for a session, including profile and filtering notices.
/// 对齐 TS:
///   `Type.Object({
///      agentId: NonEmptyString,
///      profile: NonEmptyString,
///      groups: Type.Array(ToolsEffectiveGroupSchema),
///      notices: Type.Optional(Type.Array(ToolsEffectiveNoticeSchema)),
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolsEffectiveResultSchema {
    pub agent_id: String,
    pub profile: String,
    pub groups: Vec<ToolsEffectiveGroupSchema>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub notices: Option<Vec<ToolsEffectiveNoticeSchema>>,
}

impl ToolsEffectiveResultSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_non_empty_string("agentId", &self.agent_id)?;
        validate_non_empty_string("profile", &self.profile)?;
        for (i, g) in self.groups.iter().enumerate() {
            g.validate().map_err(|e| format!("groups[{}]: {}", i, e))?;
        }
        if let Some(notices) = &self.notices {
            for (i, n) in notices.iter().enumerate() {
                n.validate().map_err(|e| format!("notices[{}]: {}", i, e))?;
            }
        }
        Ok(())
    }
}

// ============================================================================
// ToolsInvoke result
// ============================================================================

/// Normalized error shape for tool invocation failures.
/// 对齐 TS:
///   `Type.Object({
///      code: NonEmptyString,
///      message: NonEmptyString,
///      details: Type.Optional(Type.Unknown()),
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolsInvokeErrorSchema {
    pub code: String,
    pub message: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub details: Option<PluginJsonValueSchema>,
}

impl ToolsInvokeErrorSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_non_empty_string("code", &self.code)?;
        validate_non_empty_string("message", &self.message)?;
        Ok(())
    }
}

/// Tool invocation result, including approval handoff when required.
/// 对齐 TS:
///   `Type.Object({
///      ok: Type.Boolean(),
///      toolName: NonEmptyString,
///      output: Type.Optional(Type.Unknown()),
///      requiresApproval: Type.Optional(Type.Boolean()),
///      approvalId: Type.Optional(NonEmptyString),
///      source: Type.Optional(Type.Union([..., Type.String()])),
///      error: Type.Optional(ToolsInvokeErrorSchema),
///   }, { additionalProperties: false })`.
///
/// 注意: `source` 允许任何字符串 (`Type.String()`) 加上受控字面量; Rust
/// 端使用自由字符串即可涵盖.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolsInvokeResultSchema {
    pub ok: bool,
    pub tool_name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output: Option<PluginJsonValueSchema>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub requires_approval: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub approval_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<ToolsInvokeErrorSchema>,
}

impl ToolsInvokeResultSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_non_empty_string("toolName", &self.tool_name)?;
        validate_optional_non_empty_string("approvalId", self.approval_id.as_deref())?;
        if let Some(e) = &self.error {
            e.validate().map_err(|e| format!("error: {}", e))?;
        }
        Ok(())
    }
}

// Allow unused-pattern imports for tags unused at runtime.
#[allow(dead_code)]
const _UNUSED_PLACEHOLDER: () = ();
const _UNUSED_PATTERN_TAG: () = ();