// Gateway Protocol schema: sessions-catalog.
// 翻译自 packages/gateway-protocol/src/schema/sessions-catalog.ts
//
// Cross-host session catalog protocol. The session catalog namespace lets a
// single dashboard enumerate sessions that live on multiple hosts (the gateway
// itself plus attached node workers). Each catalog exposes one host set, per-
// host session lists, and an optional continue/archive bridge into the local
// sessions API.
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

// ============================================================================
// SessionCatalogCapabilitiesSchema
// ============================================================================

/// Catalog capability flags surfaced to clients.
/// 对齐 TS: `Type.Object({ continueSession: Type.Boolean(), archive: Type.Boolean() }, ...)`.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionCatalogCapabilitiesSchema {
    pub continue_session: bool,
    pub archive: bool,
}

impl SessionCatalogCapabilitiesSchema {
    pub fn validate(&self) -> Result<(), String> {
        Ok(())
    }
}

// ============================================================================
// SessionCatalogDescriptorSchema
// ============================================================================

/// Top-level catalog descriptor (id + label + capability flags).
/// 对齐 TS:
///   `Type.Object({
///      id: NonEmptyString,
///      label: NonEmptyString,
///      capabilities: SessionCatalogCapabilitiesSchema,
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionCatalogDescriptorSchema {
    pub id: String,
    pub label: String,
    pub capabilities: SessionCatalogCapabilitiesSchema,
}

impl SessionCatalogDescriptorSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_non_empty_string("id", &self.id)?;
        validate_non_empty_string("label", &self.label)?;
        self.capabilities
            .validate()
            .map_err(|e| format!("capabilities: {}", e))?;
        Ok(())
    }
}

// ============================================================================
// SessionCatalogSessionSchema
// ============================================================================

/// One session row inside a host bucket.
/// 对齐 TS:
///   `Type.Object({
///      threadId: NonEmptyString,
///      name: Type.Optional(Type.String()),
///      cwd: Type.Optional(Type.String()),
///      status: NonEmptyString,
///      createdAt: Type.Optional(Type.Number()),
///      updatedAt: Type.Optional(Type.Number()),
///      recencyAt: Type.Optional(Type.Number()),
///      source: Type.Optional(Type.String()),
///      modelProvider: Type.Optional(Type.String()),
///      cliVersion: Type.Optional(Type.String()),
///      gitBranch: Type.Optional(Type.String()),
///      archived: Type.Boolean(),
///      openClawSessionKey: Type.Optional(NonEmptyString),
///      canContinue: Type.Boolean(),
///      canArchive: Type.Boolean(),
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionCatalogSessionSchema {
    pub thread_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cwd: Option<String>,
    pub status: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created_at: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub recency_at: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model_provider: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cli_version: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub git_branch: Option<String>,
    pub archived: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub open_claw_session_key: Option<String>,
    pub can_continue: bool,
    pub can_archive: bool,
}

impl SessionCatalogSessionSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_non_empty_string("threadId", &self.thread_id)?;
        validate_non_empty_string("status", &self.status)?;
        validate_optional_non_empty_string(
            "openClawSessionKey",
            self.open_claw_session_key.as_deref(),
        )?;
        Ok(())
    }
}

// ============================================================================
// SessionCatalogHostKind enum
// ============================================================================

/// Host kind discriminator for `SessionCatalogHostSchema`.
/// 对齐 TS: `Type.Union([Type.Literal("gateway"), Type.Literal("node")])`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SessionCatalogHostKind {
    Gateway,
    Node,
}

impl SessionCatalogHostKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Gateway => "gateway",
            Self::Node => "node",
        }
    }
}

pub fn is_valid_session_catalog_host_kind(s: &str) -> bool {
    matches!(s, "gateway" | "node")
}

// ============================================================================
// SessionCatalogErrorSchema
// ============================================================================

/// Error payload attached to a catalog host or top-level catalog.
/// 对齐 TS: `Type.Object({ code: NonEmptyString, message: NonEmptyString }, ...)`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionCatalogErrorSchema {
    pub code: String,
    pub message: String,
}

impl SessionCatalogErrorSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_non_empty_string("code", &self.code)?;
        validate_non_empty_string("message", &self.message)?;
        Ok(())
    }
}

// ============================================================================
// SessionCatalogHostSchema
// ============================================================================

/// One host bucket inside a catalog (gateway itself or an attached node).
/// 对齐 TS:
///   `Type.Object({
///      hostId: NonEmptyString,
///      label: NonEmptyString,
///      kind: Type.Union([Type.Literal("gateway"), Type.Literal("node")]),
///      connected: Type.Boolean(),
///      nodeId: Type.Optional(NonEmptyString),
///      sessions: Type.Array(SessionCatalogSessionSchema),
///      nextCursor: Type.Optional(Type.String()),
///      error: Type.Optional(SessionCatalogErrorSchema),
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionCatalogHostSchema {
    pub host_id: String,
    pub label: String,
    pub kind: SessionCatalogHostKind,
    pub connected: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub node_id: Option<String>,
    pub sessions: Vec<SessionCatalogSessionSchema>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub next_cursor: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<SessionCatalogErrorSchema>,
}

impl SessionCatalogHostSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_non_empty_string("hostId", &self.host_id)?;
        validate_non_empty_string("label", &self.label)?;
        validate_optional_non_empty_string("nodeId", self.node_id.as_deref())?;
        for (i, s) in self.sessions.iter().enumerate() {
            s.validate().map_err(|e| format!("sessions[{}]: {}", i, e))?;
        }
        if let Some(e) = &self.error {
            e.validate().map_err(|e| format!("error: {}", e))?;
        }
        Ok(())
    }
}

// ============================================================================
// SessionCatalogSchema
// ============================================================================

/// Top-level catalog aggregating one or more hosts.
/// 对齐 TS:
///   `Type.Object({
///      id: NonEmptyString,
///      label: NonEmptyString,
///      capabilities: SessionCatalogCapabilitiesSchema,
///      hosts: Type.Array(SessionCatalogHostSchema),
///      error: Type.Optional(SessionCatalogErrorSchema),
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionCatalogSchema {
    pub id: String,
    pub label: String,
    pub capabilities: SessionCatalogCapabilitiesSchema,
    pub hosts: Vec<SessionCatalogHostSchema>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<SessionCatalogErrorSchema>,
}

impl SessionCatalogSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_non_empty_string("id", &self.id)?;
        validate_non_empty_string("label", &self.label)?;
        self.capabilities
            .validate()
            .map_err(|e| format!("capabilities: {}", e))?;
        for (i, h) in self.hosts.iter().enumerate() {
            h.validate().map_err(|e| format!("hosts[{}]: {}", i, e))?;
        }
        if let Some(e) = &self.error {
            e.validate().map_err(|e| format!("error: {}", e))?;
        }
        Ok(())
    }
}

// ============================================================================
// SessionsCatalogListParamsSchema — discriminated union over cursors
// ============================================================================

/// First variant — discovery without explicit cursors.
/// 对齐 TS:
///   `Type.Object({
///      catalogId: Type.Optional(NonEmptyString),
///      search: Type.Optional(Type.String()),
///      limitPerHost: Type.Optional(Type.Integer({ minimum: 1 })),
///      hostIds: Type.Optional(Type.Array(NonEmptyString)),
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionsCatalogListDiscoveryParamsSchema {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub catalog_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub search: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub limit_per_host: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub host_ids: Option<Vec<String>>,
}

impl SessionsCatalogListDiscoveryParamsSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_optional_non_empty_string("catalogId", self.catalog_id.as_deref())?;
        validate_optional_positive_integer("limitPerHost", self.limit_per_host)?;
        if let Some(ids) = &self.host_ids {
            validate_non_empty_string_list("hostIds", ids)?;
        }
        Ok(())
    }
}

/// Second variant — pagination with explicit per-host cursors.
/// 对齐 TS:
///   `Type.Object({
///      catalogId: NonEmptyString,
///      cursors: Type.Record(NonEmptyString, Type.String()),
///      search: Type.Optional(Type.String()),
///      limitPerHost: Type.Optional(Type.Integer({ minimum: 1 })),
///      hostIds: Type.Optional(Type.Array(NonEmptyString)),
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionsCatalogListPagedParamsSchema {
    pub catalog_id: String,
    /// 对齐 TS: `Type.Record(NonEmptyString, Type.String())`.
    pub cursors: std::collections::BTreeMap<String, String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub search: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub limit_per_host: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub host_ids: Option<Vec<String>>,
}

impl SessionsCatalogListPagedParamsSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_non_empty_string("catalogId", &self.catalog_id)?;
        for (k, v) in &self.cursors {
            if !is_non_empty_string(k) {
                return Err(format!(
                    "cursors: expected non-empty string key, got {:?}",
                    k
                ));
            }
            if !is_non_empty_string(v) {
                return Err(format!(
                    "cursors[{:?}]: expected non-empty string value, got {:?}",
                    k, v
                ));
            }
        }
        validate_optional_positive_integer("limitPerHost", self.limit_per_host)?;
        if let Some(ids) = &self.host_ids {
            validate_non_empty_string_list("hostIds", ids)?;
        }
        Ok(())
    }
}

/// Discriminated union: list catalogs, with or without per-host cursors.
/// 对齐 TS:
///   `Type.Union([
///      Type.Object({ catalogId?: NonEmptyString, ... }),
///      Type.Object({ catalogId: NonEmptyString, cursors: ..., ... }),
///   ])`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum SessionsCatalogListParamsSchema {
    Discovery(SessionsCatalogListDiscoveryParamsSchema),
    Paged(SessionsCatalogListPagedParamsSchema),
}

impl SessionsCatalogListParamsSchema {
    pub fn validate(&self) -> Result<(), String> {
        match self {
            Self::Discovery(d) => d.validate(),
            Self::Paged(p) => p.validate(),
        }
    }
}

// ============================================================================
// SessionsCatalogListResultSchema
// ============================================================================

/// List response — one or more catalogs keyed by id.
/// 对齐 TS: `Type.Object({ catalogs: Type.Array(SessionCatalogSchema) }, ...)`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionsCatalogListResultSchema {
    pub catalogs: Vec<SessionCatalogSchema>,
}

impl SessionsCatalogListResultSchema {
    pub fn validate(&self) -> Result<(), String> {
        for (i, c) in self.catalogs.iter().enumerate() {
            c.validate().map_err(|e| format!("catalogs[{}]: {}", i, e))?;
        }
        Ok(())
    }
}

// ============================================================================
// SessionCatalogTranscriptItemType enum
// ============================================================================

/// Item type discriminator for `SessionCatalogTranscriptItemSchema`.
/// 对齐 TS:
///   `Type.Union([
///      Type.Literal("userMessage"),
///      Type.Literal("agentMessage"),
///      Type.Literal("reasoning"),
///      Type.Literal("toolCall"),
///      Type.Literal("toolResult"),
///      Type.Literal("other"),
///   ])`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SessionCatalogTranscriptItemType {
    UserMessage,
    AgentMessage,
    Reasoning,
    ToolCall,
    ToolResult,
    Other,
}

impl SessionCatalogTranscriptItemType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::UserMessage => "userMessage",
            Self::AgentMessage => "agentMessage",
            Self::Reasoning => "reasoning",
            Self::ToolCall => "toolCall",
            Self::ToolResult => "toolResult",
            Self::Other => "other",
        }
    }
}

pub fn is_valid_session_catalog_transcript_item_type(s: &str) -> bool {
    matches!(
        s,
        "userMessage" | "agentMessage" | "reasoning" | "toolCall" | "toolResult" | "other"
    )
}

// ============================================================================
// SessionCatalogTranscriptItemSchema
// ============================================================================

/// One normalized transcript item surfaced by the catalog read endpoint.
/// 对齐 TS:
///   `Type.Object({
///      id: Type.Optional(Type.String()),
///      type: Type.Union([...]),
///      text: Type.Optional(Type.String()),
///      timestamp: Type.Optional(Type.String()),
///      model: Type.Optional(Type.String()),
///      truncated: Type.Optional(Type.Boolean()),
///      raw: Type.Optional(PluginJsonValueSchema),
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionCatalogTranscriptItemSchema {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(rename = "type")]
    pub item_type: SessionCatalogTranscriptItemType,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub truncated: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub raw: Option<PluginJsonValueSchema>,
}

impl SessionCatalogTranscriptItemSchema {
    pub fn validate(&self) -> Result<(), String> {
        Ok(())
    }
}

// ============================================================================
// SessionsCatalogRead schemas
// ============================================================================

/// Read transcript pages for one catalog thread.
/// 对齐 TS:
///   `Type.Object({
///      catalogId: NonEmptyString,
///      hostId: NonEmptyString,
///      threadId: NonEmptyString,
///      limit: Type.Optional(Type.Integer({ minimum: 1 })),
///      cursor: Type.Optional(Type.String()),
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionsCatalogReadParamsSchema {
    pub catalog_id: String,
    pub host_id: String,
    pub thread_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub limit: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,
}

impl SessionsCatalogReadParamsSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_non_empty_string("catalogId", &self.catalog_id)?;
        validate_non_empty_string("hostId", &self.host_id)?;
        validate_non_empty_string("threadId", &self.thread_id)?;
        validate_optional_positive_integer("limit", self.limit)?;
        Ok(())
    }
}

/// Read result: items + optional pagination cursor.
/// 对齐 TS:
///   `Type.Object({
///      hostId: NonEmptyString,
///      label: Type.Optional(Type.String()),
///      threadId: NonEmptyString,
///      items: Type.Array(SessionCatalogTranscriptItemSchema),
///      nextCursor: Type.Optional(Type.String()),
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionsCatalogReadResultSchema {
    pub host_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    pub thread_id: String,
    pub items: Vec<SessionCatalogTranscriptItemSchema>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub next_cursor: Option<String>,
}

impl SessionsCatalogReadResultSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_non_empty_string("hostId", &self.host_id)?;
        validate_non_empty_string("threadId", &self.thread_id)?;
        for (i, item) in self.items.iter().enumerate() {
            item.validate().map_err(|e| format!("items[{}]: {}", i, e))?;
        }
        Ok(())
    }
}

// ============================================================================
// SessionsCatalogContinue schemas
// ============================================================================

/// Continue params — identify a remote catalog thread to resume locally.
/// 对齐 TS:
///   `Type.Object({
///      catalogId: NonEmptyString,
///      hostId: NonEmptyString,
///      threadId: NonEmptyString,
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionsCatalogContinueParamsSchema {
    pub catalog_id: String,
    pub host_id: String,
    pub thread_id: String,
}

impl SessionsCatalogContinueParamsSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_non_empty_string("catalogId", &self.catalog_id)?;
        validate_non_empty_string("hostId", &self.host_id)?;
        validate_non_empty_string("threadId", &self.thread_id)?;
        Ok(())
    }
}

/// Continue result — the local session key opened for this thread.
/// 对齐 TS: `Type.Object({ sessionKey: NonEmptyString }, ...)`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionsCatalogContinueResultSchema {
    pub session_key: String,
}

impl SessionsCatalogContinueResultSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_non_empty_string("sessionKey", &self.session_key)?;
        Ok(())
    }
}

// ============================================================================
// SessionsCatalogArchive schemas
// ============================================================================

/// Archive params — requires explicit safety ack.
/// 对齐 TS:
///   `Type.Object({
///      catalogId: NonEmptyString,
///      hostId: NonEmptyString,
///      threadId: NonEmptyString,
///      confirmNoOtherRunner: Type.Literal(true),
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionsCatalogArchiveParamsSchema {
    pub catalog_id: String,
    pub host_id: String,
    pub thread_id: String,
    pub confirm_no_other_runner: bool,
}

impl SessionsCatalogArchiveParamsSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_non_empty_string("catalogId", &self.catalog_id)?;
        validate_non_empty_string("hostId", &self.host_id)?;
        validate_non_empty_string("threadId", &self.thread_id)?;
        if !self.confirm_no_other_runner {
            return Err("confirmNoOtherRunner: expected literal true".to_string());
        }
        Ok(())
    }
}

/// Archive result — fixed `ok: true`.
/// 对齐 TS: `Type.Object({ ok: Type.Literal(true) }, ...)`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionsCatalogArchiveResultSchema {
    pub ok: bool,
}

impl SessionsCatalogArchiveResultSchema {
    pub fn validate(&self) -> Result<(), String> {
        if !self.ok {
            return Err("ok: expected literal true".to_string());
        }
        Ok(())
    }
}

// (No module-private constants needed at this layer.)