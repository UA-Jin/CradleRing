// Gateway Protocol schema: agents-workspace.
// 翻译自 packages/gateway-protocol/src/schema/agents-workspace.ts
//
// Read-only agent workspace browsing schemas.
//
// These contracts back the workspace file browser in operator clients
// (mobile apps, Control UI). The surface is intentionally read-only:
// write/delete/upload stay out of this namespace until a separately
// reviewed mutation contract exists.
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

fn validate_optional_non_negative_integer(field: &str, n: Option<i64>) -> Result<(), String> {
    if let Some(v) = n {
        validate_non_negative_integer(field, v)?;
    }
    Ok(())
}

/// 对齐 TS: `Type.Integer({ minimum: 1 })`
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

fn validate_optional_positive_integer(field: &str, n: Option<i64>) -> Result<(), String> {
    if let Some(v) = n {
        validate_positive_integer(field, v)?;
    }
    Ok(())
}

// ---------- AgentsWorkspaceEntryKind enum ----------

/// Kind discriminator for one workspace directory entry.
/// 对齐 TS: `Type.Union([Type.Literal("file"), Type.Literal("directory")])`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AgentsWorkspaceEntryKind {
    File,
    Directory,
}

impl AgentsWorkspaceEntryKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::File => "file",
            Self::Directory => "directory",
        }
    }
}

pub fn is_valid_agents_workspace_entry_kind(s: &str) -> bool {
    matches!(s, "file" | "directory")
}

// ---------- AgentsWorkspaceEntrySchema ----------

/// One file or folder in an agent workspace directory listing.
/// 对齐 TS:
///   `Type.Object({
///      path: NonEmptyString,
///      name: NonEmptyString,
///      kind: Type.Union([Type.Literal("file"), Type.Literal("directory")]),
///      size: Type.Optional(Type.Integer({ minimum: 0 })),
///      updatedAtMs: Type.Optional(Type.Integer({ minimum: 0 })),
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentsWorkspaceEntrySchema {
    pub path: NonEmptyString,
    pub name: NonEmptyString,
    pub kind: AgentsWorkspaceEntryKind,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub size: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub updated_at_ms: Option<i64>,
}

impl AgentsWorkspaceEntrySchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_non_empty_string("path", &self.path)?;
        validate_non_empty_string("name", &self.name)?;
        validate_optional_non_negative_integer("size", self.size)?;
        validate_optional_non_negative_integer("updatedAtMs", self.updated_at_ms)?;
        Ok(())
    }
}

// ---------- AgentsWorkspaceListParamsSchema ----------

/// Lists one directory of an agent workspace.
/// 对齐 TS:
///   `Type.Object({
///      agentId: NonEmptyString,
///      path: Type.Optional(Type.String()),
///      offset: Type.Optional(Type.Integer({ minimum: 0 })),
///      limit: Type.Optional(Type.Integer({ minimum: 1 })),
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentsWorkspaceListParamsSchema {
    pub agent_id: NonEmptyString,
    /// 对齐 TS: `Type.String()` —— 可空、无最小长度约束（allow root "" path）。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub offset: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub limit: Option<i64>,
}

impl AgentsWorkspaceListParamsSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_non_empty_string("agentId", &self.agent_id)?;
        validate_optional_non_negative_integer("offset", self.offset)?;
        validate_optional_positive_integer("limit", self.limit)?;
        Ok(())
    }
}

// ---------- AgentsWorkspaceListResultSchema ----------

/// Paginated directory listing rooted at the agent workspace.
/// 对齐 TS:
///   `Type.Object({
///      agentId: NonEmptyString,
///      path: Type.String(),
///      parentPath: Type.Optional(Type.String()),
///      entries: Type.Array(AgentsWorkspaceEntrySchema),
///      totalEntries: Type.Integer({ minimum: 0 }),
///      offset: Type.Integer({ minimum: 0 }),
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentsWorkspaceListResultSchema {
    pub agent_id: NonEmptyString,
    pub path: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent_path: Option<String>,
    pub entries: Vec<AgentsWorkspaceEntrySchema>,
    pub total_entries: i64,
    pub offset: i64,
}

impl AgentsWorkspaceListResultSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_non_empty_string("agentId", &self.agent_id)?;
        validate_non_negative_integer("totalEntries", self.total_entries)?;
        validate_non_negative_integer("offset", self.offset)?;
        for (i, entry) in self.entries.iter().enumerate() {
            entry
                .validate()
                .map_err(|e| format!("entries[{}]: {}", i, e))?;
        }
        Ok(())
    }
}

// ---------- AgentsWorkspaceFileEncoding enum ----------

/// Encoding discriminator for workspace file previews.
/// 对齐 TS: `Type.Union([Type.Literal("utf8"), Type.Literal("base64")])`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AgentsWorkspaceFileEncoding {
    Utf8,
    Base64,
}

impl AgentsWorkspaceFileEncoding {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Utf8 => "utf8",
            Self::Base64 => "base64",
        }
    }
}

pub fn is_valid_agents_workspace_file_encoding(s: &str) -> bool {
    matches!(s, "utf8" | "base64")
}

// ---------- AgentsWorkspaceFileSchema ----------

/// One workspace file preview payload (UTF-8 text or base64 image).
/// 对齐 TS:
///   `Type.Object({
///      path: NonEmptyString,
///      name: NonEmptyString,
///      size: Type.Integer({ minimum: 0 }),
///      updatedAtMs: Type.Integer({ minimum: 0 }),
///      mimeType: NonEmptyString,
///      encoding: Type.Union([Type.Literal("utf8"), Type.Literal("base64")]),
///      content: Type.String(),
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentsWorkspaceFileSchema {
    pub path: NonEmptyString,
    pub name: NonEmptyString,
    pub size: i64,
    pub updated_at_ms: i64,
    pub mime_type: NonEmptyString,
    pub encoding: AgentsWorkspaceFileEncoding,
    pub content: String,
}

impl AgentsWorkspaceFileSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_non_empty_string("path", &self.path)?;
        validate_non_empty_string("name", &self.name)?;
        validate_non_negative_integer("size", self.size)?;
        validate_non_negative_integer("updatedAtMs", self.updated_at_ms)?;
        validate_non_empty_string("mimeType", &self.mime_type)?;
        Ok(())
    }
}

// ---------- AgentsWorkspaceGetParamsSchema ----------

/// Reads one workspace file by workspace-relative path.
/// 对齐 TS:
///   `Type.Object({
///      agentId: NonEmptyString,
///      path: NonEmptyString,
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentsWorkspaceGetParamsSchema {
    pub agent_id: NonEmptyString,
    pub path: NonEmptyString,
}

impl AgentsWorkspaceGetParamsSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_non_empty_string("agentId", &self.agent_id)?;
        validate_non_empty_string("path", &self.path)?;
        Ok(())
    }
}

// ---------- AgentsWorkspaceGetResultSchema ----------

/// Result for reading one workspace file.
/// 对齐 TS:
///   `Type.Object({
///      agentId: NonEmptyString,
///      file: AgentsWorkspaceFileSchema,
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentsWorkspaceGetResultSchema {
    pub agent_id: NonEmptyString,
    pub file: AgentsWorkspaceFileSchema,
}

impl AgentsWorkspaceGetResultSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_non_empty_string("agentId", &self.agent_id)?;
        self.file.validate().map_err(|e| format!("file: {}", e))?;
        Ok(())
    }
}

// ============================================================
// Wire types
// 对应 TS:
//   export type AgentsWorkspaceEntry = Static<typeof AgentsWorkspaceEntrySchema>;
//   export type AgentsWorkspaceFile = Static<typeof AgentsWorkspaceFileSchema>;
//   export type AgentsWorkspaceListParams = Static<typeof AgentsWorkspaceListParamsSchema>;
//   export type AgentsWorkspaceListResult = Static<typeof AgentsWorkspaceListResultSchema>;
//   export type AgentsWorkspaceGetParams = Static<typeof AgentsWorkspaceGetParamsSchema>;
//   export type AgentsWorkspaceGetResult = Static<typeof AgentsWorkspaceGetResultSchema>;
// Wire types derive directly from local schema consts so public d.ts graphs never
// pull in the ProtocolSchemas registry.
// ============================================================

pub type AgentsWorkspaceEntry = AgentsWorkspaceEntrySchema;
pub type AgentsWorkspaceFile = AgentsWorkspaceFileSchema;
pub type AgentsWorkspaceListParams = AgentsWorkspaceListParamsSchema;
pub type AgentsWorkspaceListResult = AgentsWorkspaceListResultSchema;
pub type AgentsWorkspaceGetParams = AgentsWorkspaceGetParamsSchema;
pub type AgentsWorkspaceGetResult = AgentsWorkspaceGetResultSchema;