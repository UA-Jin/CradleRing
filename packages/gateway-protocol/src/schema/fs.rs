// Gateway Protocol schema: fs.
// 翻译自 packages/gateway-protocol/src/schema/fs.ts
//
// TS 用 TypeBox 定义 schema（运行时验证 + 类型）。
// Rust 用 serde struct + 验证函数实现等价的序列化/反序列化语义。
//
// Host directory browsing for the new-session folder picker. Admin-only on the
// gateway; listing stays directories-only so the picker never leaks file names.

use serde::{Deserialize, Serialize};

// ---------- 基础验证原语 (对齐 TypeBox: NonEmptyString) ----------

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

// ---------- FsListDirParams ----------

/// Parameters for listing a directory on a connected node host.
/// 对齐 TS:
/// ```ts
/// export const FsListDirParamsSchema = Type.Object({
///   path: Type.Optional(NonEmptyString),
///   nodeId: Type.Optional(NonEmptyString),
/// }, { additionalProperties: false });
/// ```
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FsListDirParams {
    /// Absolute directory to list; omitted means the selected host's home directory.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    /// Connected node host to browse; omitted means the Gateway host.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub node_id: Option<String>,
}

impl FsListDirParams {
    pub fn validate(&self) -> Result<(), String> {
        validate_optional_non_empty_string(self.path.as_deref())?;
        validate_optional_non_empty_string(self.node_id.as_deref())?;
        Ok(())
    }
}

// ---------- FsDirEntry ----------

/// A single directory entry returned by `FsListDir`.
/// 对齐 TS:
/// ```ts
/// export const FsDirEntrySchema = Type.Object({
///   name: NonEmptyString,
///   path: NonEmptyString,
///   hidden: Type.Optional(Type.Boolean()),
/// }, { additionalProperties: false });
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FsDirEntry {
    pub name: String,
    pub path: String,
    /// Dot-prefixed directories; clients render them dimmed after visible ones.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hidden: Option<bool>,
}

impl FsDirEntry {
    pub fn validate(&self) -> Result<(), String> {
        validate_non_empty_string(&self.name)?;
        validate_non_empty_string(&self.path)?;
        Ok(())
    }
}

// ---------- FsListDirResult ----------

/// Result payload for a host directory listing.
/// 对齐 TS:
/// ```ts
/// export const FsListDirResultSchema = Type.Object({
///   path: NonEmptyString,
///   parent: Type.Optional(NonEmptyString),
///   home: NonEmptyString,
///   entries: Type.Array(FsDirEntrySchema),
/// }, { additionalProperties: false });
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FsListDirResult {
    /// Resolved absolute path that was listed.
    pub path: String,
    /// Absent at the filesystem root.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent: Option<String>,
    /// Selected host's home directory, for the picker's "home" shortcut.
    pub home: String,
    pub entries: Vec<FsDirEntry>,
}

impl FsListDirResult {
    pub fn validate(&self) -> Result<(), String> {
        validate_non_empty_string(&self.path)?;
        validate_optional_non_empty_string(self.parent.as_deref())?;
        validate_non_empty_string(&self.home)?;
        for (i, entry) in self.entries.iter().enumerate() {
            entry
                .validate()
                .map_err(|e| format!("entries[{}]: {}", i, e))?;
        }
        Ok(())
    }
}

// Wire types derive directly from local schema consts so public d.ts graphs never
// pull in the ProtocolSchemas registry.
// 对应 TS:
//   export type FsDirEntry = Static<typeof FsDirEntrySchema>;
//   export type FsListDirParams = Static<typeof FsListDirParamsSchema>;
//   export type FsListDirResult = Static<typeof FsListDirResultSchema>;
pub type FsDirEntryType = FsDirEntry;
pub type FsListDirParamsType = FsListDirParams;
pub type FsListDirResultType = FsListDirResult;