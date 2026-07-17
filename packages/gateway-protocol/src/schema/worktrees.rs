// Gateway Protocol schema: worktrees.
// 翻译自 packages/gateway-protocol/src/schema/worktrees.ts
//
// Worktree management protocol schemas.
//
// TS 用 TypeBox 定义 schema（运行时验证 + 类型）。
// Rust 用 serde struct + 验证函数实现等价的序列化/反序列化语义。

use serde::{Deserialize, Serialize};

use super::primitives::NonEmptyString;

// ---------- Module-private bounds ----------

/// Worktree name grammar: starts with `[a-z0-9]`, followed by 0-63 chars of
/// `[a-z0-9-]` (kebab-case lower-case identifier).
/// 对齐 TS: `WorktreeNameSchema = Type.String({ pattern: "^[a-z0-9][a-z0-9-]{0,63}$" })`.
const WORKTREE_NAME_PATTERN: &str = r"^[a-z0-9][a-z0-9-]{0,63}$";

/// Repo fingerprint grammar: 16 hex chars.
/// 对齐 TS: `Type.String({ pattern: "^[a-f0-9]{16}$" })`.
const WORKTREE_REPO_FINGERPRINT_PATTERN: &str = r"^[a-f0-9]{16}$";

/// Closed enumeration of allowed worktree owner kinds.
/// 对齐 TS: `Type.String({ enum: ["manual", "workboard", "session"] })`.
pub mod worktree_owner_kinds {
    pub const MANUAL: &str = "manual";
    pub const WORKBOARD: &str = "workboard";
    pub const SESSION: &str = "session";

    pub fn all() -> &'static [&'static str] {
        &[MANUAL, WORKBOARD, SESSION]
    }

    pub fn from_str(s: &str) -> Option<&'static str> {
        match s {
            "manual" => Some(MANUAL),
            "workboard" => Some(WORKBOARD),
            "session" => Some(SESSION),
            _ => None,
        }
    }
}

pub fn is_valid_worktree_owner_kind(kind: &str) -> bool {
    worktree_owner_kinds::from_str(kind).is_some()
}

// ---------- 基础验证原语 (对齐 TypeBox: NonEmptyString / String{pattern} / Integer{min:0}) ----------

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

fn validate_optional_non_empty_string_list(
    field: &str,
    values: Option<&Vec<String>>,
) -> Result<(), String> {
    if let Some(arr) = values {
        for (i, v) in arr.iter().enumerate() {
            validate_non_empty_string(&format!("{}[{}]", field, i), v)?;
        }
    }
    Ok(())
}

/// 对齐 TS: `Type.Integer({ minimum: 0 })`
fn validate_non_negative_integer(field: &str, n: i64) -> Result<(), String> {
    if n >= 0 {
        Ok(())
    } else {
        Err(format!("{}: expected integer >= 0, got {}", field, n))
    }
}

fn validate_optional_non_negative_integer(field: &str, value: Option<i64>) -> Result<(), String> {
    if let Some(n) = value {
        validate_non_negative_integer(field, n)?;
    }
    Ok(())
}

fn regex(pattern: &str) -> regex::Regex {
    regex::Regex::new(pattern).expect("invalid regex pattern compiled into worktrees")
}

/// 对齐 TS: `Type.String({ pattern: "^[a-z0-9][a-z0-9-]{0,63}$" })`.
fn validate_worktree_name(field: &str, value: &str) -> Result<(), String> {
    if !regex(WORKTREE_NAME_PATTERN).is_match(value) {
        return Err(format!(
            "{}: expected string matching {:?}, got {:?}",
            field, WORKTREE_NAME_PATTERN, value
        ));
    }
    Ok(())
}

fn validate_optional_worktree_name(field: &str, value: Option<&str>) -> Result<(), String> {
    if let Some(s) = value {
        validate_worktree_name(field, s)?;
    }
    Ok(())
}

/// 对齐 TS: `Type.String({ pattern: "^[a-f0-9]{16}$" })`.
fn validate_worktree_repo_fingerprint(field: &str, value: &str) -> Result<(), String> {
    if !regex(WORKTREE_REPO_FINGERPRINT_PATTERN).is_match(value) {
        return Err(format!(
            "{}: expected string matching {:?}, got {:?}",
            field, WORKTREE_REPO_FINGERPRINT_PATTERN, value
        ));
    }
    Ok(())
}

// ---------- WorktreeOwnerKindSchema ----------

/// Closed enumeration of allowed worktree owner kinds.
/// 对齐 TS: `Type.String({ enum: ["manual", "workboard", "session"] })`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum WorktreeOwnerKind {
    #[serde(rename = "manual")]
    Manual,
    #[serde(rename = "workboard")]
    Workboard,
    #[serde(rename = "session")]
    Session,
}

impl WorktreeOwnerKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Manual => "manual",
            Self::Workboard => "workboard",
            Self::Session => "session",
        }
    }
}

// ---------- WorktreeRecordSchema ----------

/// One worktree record returned by `worktrees.list`.
/// 对齐 TS:
///   `Type.Object({
///      id:             NonEmptyString,
///      name:           WorktreeNameSchema,
///      repoFingerprint:Type.String({ pattern: "^[a-f0-9]{16}$" }),
///      repoRoot:       NonEmptyString,
///      path:           NonEmptyString,
///      branch:         NonEmptyString,
///      baseRef:        NonEmptyString,
///      ownerKind:      Type.String({ enum: ["manual", "workboard", "session"] }),
///      ownerId:        Type.Optional(NonEmptyString),
///      snapshotRef:    Type.Optional(NonEmptyString),
///      createdAt:      Type.Integer({ minimum: 0 }),
///      lastActiveAt:   Type.Integer({ minimum: 0 }),
///      removedAt:      Type.Optional(Type.Integer({ minimum: 0 })),
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorktreeRecord {
    pub id: NonEmptyString,
    pub name: String,
    pub repo_fingerprint: String,
    pub repo_root: NonEmptyString,
    pub path: NonEmptyString,
    pub branch: NonEmptyString,
    pub base_ref: NonEmptyString,
    pub owner_kind: WorktreeOwnerKind,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub owner_id: Option<NonEmptyString>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub snapshot_ref: Option<NonEmptyString>,
    pub created_at: i64,
    pub last_active_at: i64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub removed_at: Option<i64>,
}

impl WorktreeRecord {
    pub fn validate(&self) -> Result<(), String> {
        validate_non_empty_string("id", &self.id)?;
        validate_worktree_name("name", &self.name)?;
        validate_worktree_repo_fingerprint("repoFingerprint", &self.repo_fingerprint)?;
        validate_non_empty_string("repoRoot", &self.repo_root)?;
        validate_non_empty_string("path", &self.path)?;
        validate_non_empty_string("branch", &self.branch)?;
        validate_non_empty_string("baseRef", &self.base_ref)?;
        validate_optional_non_empty_string("ownerId", self.owner_id.as_deref())?;
        validate_optional_non_empty_string("snapshotRef", self.snapshot_ref.as_deref())?;
        validate_non_negative_integer("createdAt", self.created_at)?;
        validate_non_negative_integer("lastActiveAt", self.last_active_at)?;
        validate_optional_non_negative_integer("removedAt", self.removed_at)?;
        Ok(())
    }
}

// ---------- WorktreesListParamsSchema ----------

/// Empty request payload for listing worktrees.
/// 对齐 TS: `Type.Object({}, { additionalProperties: false })`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct WorktreesListParams {}

impl WorktreesListParams {
    pub fn validate(&self) -> Result<(), String> {
        // No required/constrained fields; the empty schema always validates.
        Ok(())
    }
}

// ---------- WorktreesListResultSchema ----------

/// Bounded list of worktrees returned by `worktrees.list`.
/// 对齐 TS:
///   `Type.Object({ worktrees: Type.Array(WorktreeRecordSchema) },
///                { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorktreesListResult {
    pub worktrees: Vec<WorktreeRecord>,
}

impl WorktreesListResult {
    pub fn validate(&self) -> Result<(), String> {
        for (i, w) in self.worktrees.iter().enumerate() {
            w.validate().map_err(|e| format!("worktrees[{}]: {}", i, e))?;
        }
        Ok(())
    }
}

// ---------- WorktreesCreateParamsSchema ----------

/// Parameters for creating a new worktree.
/// 对齐 TS:
///   `Type.Object({
///      repoRoot: NonEmptyString,
///      name:     Type.Optional(WorktreeNameSchema),
///      baseRef:  Type.Optional(NonEmptyString),
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorktreesCreateParams {
    pub repo_root: NonEmptyString,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub base_ref: Option<NonEmptyString>,
}

impl WorktreesCreateParams {
    pub fn validate(&self) -> Result<(), String> {
        validate_non_empty_string("repoRoot", &self.repo_root)?;
        validate_optional_worktree_name("name", self.name.as_deref())?;
        validate_optional_non_empty_string("baseRef", self.base_ref.as_deref())?;
        Ok(())
    }
}

// ---------- WorktreesRemoveParamsSchema ----------

/// Parameters for removing a worktree.
/// 对齐 TS:
///   `Type.Object({ id: NonEmptyString, force: Type.Optional(Type.Boolean()) },
///                { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorktreesRemoveParams {
    pub id: NonEmptyString,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub force: Option<bool>,
}

impl WorktreesRemoveParams {
    pub fn validate(&self) -> Result<(), String> {
        validate_non_empty_string("id", &self.id)?;
        Ok(())
    }
}

// ---------- WorktreesRemoveResultSchema ----------

/// Result of a successful worktree removal.
/// 对齐 TS:
///   `Type.Object({
///      removed:       Type.Boolean(),
///      snapshotRef:   Type.Optional(NonEmptyString),
///      // Why the pre-removal snapshot failed; present only on forced removals
///      // that continued without one.
///      snapshotError: Type.Optional(NonEmptyString),
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorktreesRemoveResult {
    pub removed: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub snapshot_ref: Option<NonEmptyString>,
    /// Why the pre-removal snapshot failed; present only on forced removals that
    /// continued without one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub snapshot_error: Option<NonEmptyString>,
}

impl WorktreesRemoveResult {
    pub fn validate(&self) -> Result<(), String> {
        validate_optional_non_empty_string("snapshotRef", self.snapshot_ref.as_deref())?;
        validate_optional_non_empty_string("snapshotError", self.snapshot_error.as_deref())?;
        Ok(())
    }
}

// ---------- WorktreesBranchesParamsSchema ----------

/// Parameters for listing branches of a repo.
/// 对齐 TS: `Type.Object({ repoRoot: NonEmptyString }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorktreesBranchesParams {
    pub repo_root: NonEmptyString,
}

impl WorktreesBranchesParams {
    pub fn validate(&self) -> Result<(), String> {
        validate_non_empty_string("repoRoot", &self.repo_root)?;
        Ok(())
    }
}

// ---------- WorktreeBranchKind (closed enum) ----------

/// Closed enumeration of branch source kinds.
/// 对齐 TS: `Type.Union([Type.Literal("local"), Type.Literal("remote")])`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum WorktreeBranchKind {
    #[serde(rename = "local")]
    Local,
    #[serde(rename = "remote")]
    Remote,
}

impl WorktreeBranchKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Local => "local",
            Self::Remote => "remote",
        }
    }
}

// ---------- WorktreeBranchSchema ----------

/// One branch entry returned by `worktrees.branches`.
/// 对齐 TS:
///   `Type.Object({
///      name: NonEmptyString,
///      kind: Type.Union([Type.Literal("local"), Type.Literal("remote")]),
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorktreeBranch {
    pub name: NonEmptyString,
    pub kind: WorktreeBranchKind,
}

impl WorktreeBranch {
    pub fn validate(&self) -> Result<(), String> {
        validate_non_empty_string("name", &self.name)?;
        Ok(())
    }
}

// ---------- WorktreesBranchesResultSchema ----------

/// Bounded branch list response.
/// 对齐 TS:
///   `Type.Object({
///      branches:      Type.Array(WorktreeBranchSchema),
///      defaultBranch: Type.Optional(NonEmptyString),
///      headBranch:    Type.Optional(NonEmptyString),
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorktreesBranchesResult {
    pub branches: Vec<WorktreeBranch>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_branch: Option<NonEmptyString>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub head_branch: Option<NonEmptyString>,
}

impl WorktreesBranchesResult {
    pub fn validate(&self) -> Result<(), String> {
        for (i, b) in self.branches.iter().enumerate() {
            b.validate().map_err(|e| format!("branches[{}]: {}", i, e))?;
        }
        validate_optional_non_empty_string("defaultBranch", self.default_branch.as_deref())?;
        validate_optional_non_empty_string("headBranch", self.head_branch.as_deref())?;
        Ok(())
    }
}

// ---------- WorktreesRestoreParamsSchema ----------

/// Parameters for restoring a removed worktree.
/// 对齐 TS: `Type.Object({ id: NonEmptyString }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorktreesRestoreParams {
    pub id: NonEmptyString,
}

impl WorktreesRestoreParams {
    pub fn validate(&self) -> Result<(), String> {
        validate_non_empty_string("id", &self.id)?;
        Ok(())
    }
}

// ---------- WorktreesGcParamsSchema ----------

/// Empty request payload for `worktrees.gc`.
/// 对齐 TS: `Type.Object({}, { additionalProperties: false })`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct WorktreesGcParams {}

impl WorktreesGcParams {
    pub fn validate(&self) -> Result<(), String> {
        // No required/constrained fields; the empty schema always validates.
        Ok(())
    }
}

// ---------- WorktreesGcResultSchema ----------

/// Result of a `worktrees.gc` sweep.
/// 对齐 TS:
///   `Type.Object({
///      removed:         Type.Array(NonEmptyString),
///      orphansDeleted:  Type.Integer({ minimum: 0 }),
///      snapshotsPruned: Type.Integer({ minimum: 0 }),
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorktreesGcResult {
    pub removed: Vec<NonEmptyString>,
    pub orphans_deleted: i64,
    pub snapshots_pruned: i64,
}

impl WorktreesGcResult {
    pub fn validate(&self) -> Result<(), String> {
        validate_optional_non_empty_string_list("removed", Some(&self.removed))?;
        validate_non_negative_integer("orphansDeleted", self.orphans_deleted)?;
        validate_non_negative_integer("snapshotsPruned", self.snapshots_pruned)?;
        Ok(())
    }
}

// Wire types derive directly from local schema consts so public d.ts graphs never
// pull in the ProtocolSchemas registry.
// 对应 TS:
//   export type WorktreeRecord           = Static<typeof WorktreeRecordSchema>;
//   export type WorktreesListParams      = Static<typeof WorktreesListParamsSchema>;
//   export type WorktreesListResult      = Static<typeof WorktreesListResultSchema>;
//   export type WorktreesCreateParams    = Static<typeof WorktreesCreateParamsSchema>;
//   export type WorktreesRemoveParams    = Static<typeof WorktreesRemoveParamsSchema>;
//   export type WorktreesRemoveResult    = Static<typeof WorktreesRemoveResultSchema>;
//   export type WorktreesRestoreParams   = Static<typeof WorktreesRestoreParamsSchema>;
//   export type WorktreesGcParams        = Static<typeof WorktreesGcParamsSchema>;
//   export type WorktreesGcResult        = Static<typeof WorktreesGcResultSchema>;
//   export type WorktreeBranch           = Static<typeof WorktreeBranchSchema>;
//   export type WorktreesBranchesParams  = Static<typeof WorktreesBranchesParamsSchema>;
//   export type WorktreesBranchesResult  = Static<typeof WorktreesBranchesResultSchema>;
pub type WorktreeRecordType = WorktreeRecord;
pub type WorktreesListParamsType = WorktreesListParams;
pub type WorktreesListResultType = WorktreesListResult;
pub type WorktreesCreateParamsType = WorktreesCreateParams;
pub type WorktreesRemoveParamsType = WorktreesRemoveParams;
pub type WorktreesRemoveResultType = WorktreesRemoveResult;
pub type WorktreesRestoreParamsType = WorktreesRestoreParams;
pub type WorktreesGcParamsType = WorktreesGcParams;
pub type WorktreesGcResultType = WorktreesGcResult;
pub type WorktreeBranchType = WorktreeBranch;
pub type WorktreesBranchesParamsType = WorktreesBranchesParams;
pub type WorktreesBranchesResultType = WorktreesBranchesResult;