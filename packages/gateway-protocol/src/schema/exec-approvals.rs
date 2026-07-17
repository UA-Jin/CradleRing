// Gateway Protocol schema: exec_approvals.
// 翻译自 packages/gateway-protocol/src/schema/exec-approvals.ts
//
// Exec approval protocol schemas.
//
// These payloads cross the security-review boundary for command execution, so
// persisted policy, request snapshots, and resolve decisions stay explicit.
//
// TS 用 TypeBox 定义 schema（运行时验证 + 类型）。
// Rust 用 serde struct + 验证函数实现等价的序列化/反序列化语义。

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use super::primitives::NonEmptyString;

// ---------- 基础验证原语 (对齐 TypeBox: NonEmptyString / Integer{min} / Number{min}) ----------

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
    values: Option<&Vec<NonEmptyString>>,
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

/// 对齐 TS: `Type.Integer({ minimum: 1 })`
fn validate_positive_integer(field: &str, n: i64) -> Result<(), String> {
    if n >= 1 {
        Ok(())
    } else {
        Err(format!("{}: expected integer >= 1, got {}", field, n))
    }
}

/// 对齐 TS: `Type.Number({ minimum: 0 })` —— 非负有限数。
fn validate_non_negative_number(field: &str, n: f64) -> Result<(), String> {
    if n.is_finite() && n >= 0.0 {
        Ok(())
    } else {
        Err(format!(
            "{}: expected finite number >= 0, got {}",
            field, n
        ))
    }
}

fn validate_optional_non_negative_number(field: &str, value: Option<f64>) -> Result<(), String> {
    if let Some(n) = value {
        validate_non_negative_number(field, n)?;
    }
    Ok(())
}

// ---------- 闭合枚举常量 ----------

/// 对齐 TS: `ExecSecuritySchema = Type.Union([Type.Literal("deny"),
///                                                Type.Literal("allowlist"),
///                                                Type.Literal("full")])`
pub mod exec_security {
    pub const DENY: &str = "deny";
    pub const ALLOWLIST: &str = "allowlist";
    pub const FULL: &str = "full";

    pub fn all() -> &'static [&'static str] {
        &[DENY, ALLOWLIST, FULL]
    }

    pub fn from_str(s: &str) -> Option<&'static str> {
        match s {
            "deny" => Some(DENY),
            "allowlist" => Some(ALLOWLIST),
            "full" => Some(FULL),
            _ => None,
        }
    }
}

pub fn is_valid_exec_security(s: &str) -> bool {
    exec_security::from_str(s).is_some()
}

/// 对齐 TS: `ExecAskSchema = Type.Union([Type.Literal("off"),
///                                          Type.Literal("on-miss"),
///                                          Type.Literal("always")])`
pub mod exec_ask {
    pub const OFF: &str = "off";
    pub const ON_MISS: &str = "on-miss";
    pub const ALWAYS: &str = "always";

    pub fn all() -> &'static [&'static str] {
        &[OFF, ON_MISS, ALWAYS]
    }

    pub fn from_str(s: &str) -> Option<&'static str> {
        match s {
            "off" => Some(OFF),
            "on-miss" => Some(ON_MISS),
            "always" => Some(ALWAYS),
            _ => None,
        }
    }
}

pub fn is_valid_exec_ask(s: &str) -> bool {
    exec_ask::from_str(s).is_some()
}

/// 对齐 TS: `NativeExecApprovalActionSchema = Type.Union([
///              Type.Literal("allow"), Type.Literal("deny"), Type.Literal("prompt")])`
pub mod native_exec_approval_action {
    pub const ALLOW: &str = "allow";
    pub const DENY: &str = "deny";
    pub const PROMPT: &str = "prompt";

    pub fn all() -> &'static [&'static str] {
        &[ALLOW, DENY, PROMPT]
    }

    pub fn from_str(s: &str) -> Option<&'static str> {
        match s {
            "allow" => Some(ALLOW),
            "deny" => Some(DENY),
            "prompt" => Some(PROMPT),
            _ => None,
        }
    }
}

pub fn is_valid_native_exec_approval_action(s: &str) -> bool {
    native_exec_approval_action::from_str(s).is_some()
}

// ---------- ExecApprovalsAllowlistEntrySchema ----------

/// One persisted allowlist entry for a command pattern or resolved executable.
/// 对齐 TS:
/// ```ts
/// export const ExecApprovalsAllowlistEntrySchema = Type.Object({
///   id:                 Type.Optional(NonEmptyString),
///   pattern:            Type.String(),
///   source:             Type.Optional(Type.Literal("allow-always")),
///   commandText:        Type.Optional(Type.String()),
///   argPattern:         Type.Optional(Type.String()),
///   lastUsedAt:         Type.Optional(Type.Number({ minimum: 0 })),
///   lastUsedCommand:    Type.Optional(Type.String()),
///   lastResolvedPath:   Type.Optional(Type.String()),
/// }, { additionalProperties: false });
/// ```
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecApprovalsAllowlistEntrySchema {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<NonEmptyString>,
    pub pattern: String,
    /// 对齐 TS: `Type.Optional(Type.Literal("allow-always"))`
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub command_text: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub arg_pattern: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_used_at: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_used_command: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_resolved_path: Option<String>,
}

impl ExecApprovalsAllowlistEntrySchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_optional_non_empty_string("id", self.id.as_deref())?;
        if self.pattern.is_empty() {
            return Err("pattern: expected non-empty string".to_string());
        }
        if let Some(src) = &self.source {
            if src != "allow-always" {
                return Err(format!(
                    "source: expected literal \"allow-always\", got {:?}",
                    src
                ));
            }
        }
        validate_optional_non_negative_number("lastUsedAt", self.last_used_at)?;
        Ok(())
    }
}

// ---------- ExecApprovalsPolicyFields (内部共用字段集) ----------

/// Common policy fields shared by defaults / agent overlays.
/// 对齐 TS:
/// ```ts
/// const ExecApprovalsPolicyFields = {
///   security:        Type.Optional(Type.String()),
///   ask:             Type.Optional(Type.String()),
///   askFallback:     Type.Optional(Type.String()),
///   autoAllowSkills: Type.Optional(Type.Boolean()),
/// };
/// ```
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecApprovalsPolicyFields {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub security: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ask: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ask_fallback: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub auto_allow_skills: Option<bool>,
}

impl ExecApprovalsPolicyFields {
    pub fn validate(&self) -> Result<(), String> {
        if let Some(s) = &self.security {
            if !is_valid_exec_security(s) {
                return Err(format!(
                    "security: expected one of {:?}, got {:?}",
                    exec_security::all(),
                    s
                ));
            }
        }
        if let Some(a) = &self.ask {
            if !is_valid_exec_ask(a) {
                return Err(format!(
                    "ask: expected one of {:?}, got {:?}",
                    exec_ask::all(),
                    a
                ));
            }
        }
        if let Some(s) = &self.ask_fallback {
            if !is_valid_exec_security(s) {
                return Err(format!(
                    "askFallback: expected one of {:?}, got {:?}",
                    exec_security::all(),
                    s
                ));
            }
        }
        Ok(())
    }
}

// ---------- ExecApprovalsResolvedDefaultsSchema ----------

/// Host-resolved default policy after applying persisted defaults and runtime fallbacks.
/// 对齐 TS:
/// ```ts
/// const ExecApprovalsResolvedDefaultsSchema = Type.Object({
///   security:        ExecSecuritySchema,
///   ask:             ExecAskSchema,
///   askFallback:     ExecSecuritySchema,
///   autoAllowSkills: Type.Boolean(),
/// }, { additionalProperties: false });
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecApprovalsResolvedDefaultsSchema {
    pub security: String,
    pub ask: String,
    pub ask_fallback: String,
    pub auto_allow_skills: bool,
}

impl ExecApprovalsResolvedDefaultsSchema {
    pub fn validate(&self) -> Result<(), String> {
        if !is_valid_exec_security(&self.security) {
            return Err(format!(
                "security: expected one of {:?}, got {:?}",
                exec_security::all(),
                self.security
            ));
        }
        if !is_valid_exec_ask(&self.ask) {
            return Err(format!(
                "ask: expected one of {:?}, got {:?}",
                exec_ask::all(),
                self.ask
            ));
        }
        if !is_valid_exec_security(&self.ask_fallback) {
            return Err(format!(
                "askFallback: expected one of {:?}, got {:?}",
                exec_security::all(),
                self.ask_fallback
            ));
        }
        Ok(())
    }
}

// ---------- ExecApprovalsDefaultsSchema ----------

/// Default exec approval policy shared by all agents unless overridden.
/// 对齐 TS:
/// ```ts
/// export const ExecApprovalsDefaultsSchema =
///   Type.Object(ExecApprovalsPolicyFields, { additionalProperties: false });
/// ```
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecApprovalsDefaultsSchema {
    #[serde(flatten)]
    pub policy: ExecApprovalsPolicyFields,
}

impl ExecApprovalsDefaultsSchema {
    pub fn validate(&self) -> Result<(), String> {
        self.policy.validate()
    }
}

// ---------- ExecApprovalsAgentSchema ----------

/// Agent-specific exec approval policy and allowlist.
/// 对齐 TS:
/// ```ts
/// export const ExecApprovalsAgentSchema = Type.Object({
///   ...ExecApprovalsPolicyFields,
///   allowlist: Type.Optional(Type.Array(ExecApprovalsAllowlistEntrySchema)),
/// }, { additionalProperties: false });
/// ```
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecApprovalsAgentSchema {
    #[serde(flatten)]
    pub policy: ExecApprovalsPolicyFields,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub allowlist: Option<Vec<ExecApprovalsAllowlistEntrySchema>>,
}

impl ExecApprovalsAgentSchema {
    pub fn validate(&self) -> Result<(), String> {
        self.policy.validate()?;
        if let Some(arr) = &self.allowlist {
            for (i, entry) in arr.iter().enumerate() {
                entry
                    .validate()
                    .map_err(|e| format!("allowlist[{}]: {}", i, e))?;
            }
        }
        Ok(())
    }
}

// ---------- ExecApprovalsFileSchema ----------

/// Versioned exec approvals config file edited through gateway APIs.
/// 对齐 TS:
/// ```ts
/// export const ExecApprovalsFileSchema = Type.Object({
///   version:  Type.Literal(1),
///   socket:   Type.Optional(Type.Object({
///     path:   Type.Optional(Type.String()),
///     token:  Type.Optional(Type.String()),
///   }, { additionalProperties: false })),
///   defaults: Type.Optional(ExecApprovalsDefaultsSchema),
///   agents:   Type.Optional(Type.Record(Type.String(), ExecApprovalsAgentSchema)),
/// }, { additionalProperties: false });
/// ```
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecApprovalsSocketSchema {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub token: Option<String>,
}

impl ExecApprovalsSocketSchema {
    pub fn validate(&self) -> Result<(), String> {
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecApprovalsFileSchema {
    /// 对齐 TS: `Type.Literal(1)` —— JSON 字面量 1。
    pub version: i64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub socket: Option<ExecApprovalsSocketSchema>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub defaults: Option<ExecApprovalsDefaultsSchema>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub agents: Option<BTreeMap<String, ExecApprovalsAgentSchema>>,
}

impl ExecApprovalsFileSchema {
    pub fn validate(&self) -> Result<(), String> {
        if self.version != 1 {
            return Err(format!("version: expected literal 1, got {}", self.version));
        }
        if let Some(s) = &self.socket {
            s.validate().map_err(|e| format!("socket: {}", e))?;
        }
        if let Some(d) = &self.defaults {
            d.validate().map_err(|e| format!("defaults: {}", e))?;
        }
        if let Some(agents) = &self.agents {
            for (k, v) in agents {
                v.validate().map_err(|e| format!("agents[{}]: {}", k, e))?;
            }
        }
        Ok(())
    }
}

// ---------- ExecApprovalsSnapshotSchema ----------

/// File-backed read snapshot with path/hash metadata for optimistic writes.
/// 对齐 TS:
/// ```ts
/// export const ExecApprovalsSnapshotSchema = Type.Object({
///   path:   NonEmptyString,
///   exists: Type.Boolean(),
///   hash:   NonEmptyString,
///   file:   ExecApprovalsFileSchema,
/// }, { additionalProperties: false });
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecApprovalsSnapshotSchema {
    pub path: NonEmptyString,
    pub exists: bool,
    pub hash: NonEmptyString,
    pub file: ExecApprovalsFileSchema,
}

impl ExecApprovalsSnapshotSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_non_empty_string("path", &self.path)?;
        validate_non_empty_string("hash", &self.hash)?;
        self.file.validate().map_err(|e| format!("file: {}", e))?;
        Ok(())
    }
}

// ---------- NativeExecApprovalRuleSchema / ConstraintsSchema ----------

/// One rule owned and enforced by a host-native exec policy implementation.
/// 对齐 TS:
/// ```ts
/// const NativeExecApprovalRuleSchema = Type.Object({
///   pattern:     NonEmptyString,
///   action:      NativeExecApprovalActionSchema,
///   shells:      Type.Optional(Type.Array(NonEmptyString)),
///   description: Type.Optional(Type.String()),
///   enabled:     Type.Optional(Type.Boolean()),
/// }, { additionalProperties: false });
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NativeExecApprovalRuleSchema {
    pub pattern: NonEmptyString,
    pub action: String, // 闭合字面量 ("allow" | "deny" | "prompt")
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub shells: Option<Vec<NonEmptyString>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,
}

impl NativeExecApprovalRuleSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_non_empty_string("pattern", &self.pattern)?;
        if !is_valid_native_exec_approval_action(&self.action) {
            return Err(format!(
                "action: expected one of {:?}, got {:?}",
                native_exec_approval_action::all(),
                self.action
            ));
        }
        validate_optional_non_empty_string_list("shells", self.shells.as_ref())?;
        Ok(())
    }
}

/// 对齐 TS:
/// ```ts
/// const NativeExecApprovalConstraintsSchema = Type.Object({
///   baseHashRequired:         Type.Optional(Type.Boolean()),
///   defaultAllowAllowed:      Type.Optional(Type.Boolean()),
///   broadAllowRulesAllowed:   Type.Optional(Type.Boolean()),
///   dangerousAllowRulesAllowed: Type.Optional(Type.Boolean()),
/// }, { additionalProperties: false });
/// ```
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NativeExecApprovalConstraintsSchema {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub base_hash_required: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_allow_allowed: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub broad_allow_rules_allowed: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dangerous_allow_rules_allowed: Option<bool>,
}

impl NativeExecApprovalConstraintsSchema {
    pub fn validate(&self) -> Result<(), String> {
        Ok(())
    }
}

// ---------- ExecApprovalsNodeSnapshotSchema ----------

/// Node read snapshot supporting file-backed and host-native approval owners.
/// 对齐 TS — 内含一组 `oneOf` 互斥形态：
///   1. 文件后端形态：path/exists/hash/file 必备，不能出现 enabled/baseHash/defaultAction/rules/constraints/message；
///   2. native 启用形态：enabled=true 且 hash 非空、defaultAction/rules 必备，不能出现 path/exists/file/resolvedDefaults/message；
///   3. native 禁用形态：只允许 enabled=false。
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecApprovalsNodeSnapshotSchema {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub exists: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hash: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file: Option<ExecApprovalsFileSchema>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resolved_defaults: Option<ExecApprovalsResolvedDefaultsSchema>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub base_hash: Option<NonEmptyString>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_action: Option<String>, // 闭合字面量 ("allow" | "deny" | "prompt")
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rules: Option<Vec<NativeExecApprovalRuleSchema>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub constraints: Option<NativeExecApprovalConstraintsSchema>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

impl ExecApprovalsNodeSnapshotSchema {
    pub fn validate(&self) -> Result<(), String> {
        // Step 1: validate inner shapes that carry their own constraints.
        if let Some(rd) = &self.resolved_defaults {
            rd.validate()
                .map_err(|e| format!("resolvedDefaults: {}", e))?;
        }
        if let Some(da) = &self.default_action {
            if !is_valid_native_exec_approval_action(da) {
                return Err(format!(
                    "defaultAction: expected one of {:?}, got {:?}",
                    native_exec_approval_action::all(),
                    da
                ));
            }
        }
        if let Some(rules) = &self.rules {
            for (i, r) in rules.iter().enumerate() {
                r.validate().map_err(|e| format!("rules[{}]: {}", i, e))?;
            }
        }
        if let Some(c) = &self.constraints {
            c.validate().map_err(|e| format!("constraints: {}", e))?;
        }

        // Step 2: enforce the `oneOf` shape selection.
        let file_form: bool = self.path.is_some()
            && self.exists.is_some()
            && self.hash.is_some()
            && self.file.is_some();
        let native_enabled_form: bool = matches!(self.enabled, Some(true))
            && self.hash.is_some()
            && self.default_action.is_some()
            && self.rules.is_some();
        let native_disabled_form: bool = matches!(self.enabled, Some(false));

        if file_form {
            // Disallow native fields.
            let native_extras = self.enabled.is_some()
                || self.base_hash.is_some()
                || self.default_action.is_some()
                || self.rules.is_some()
                || self.constraints.is_some()
                || self.message.is_some();
            if native_extras {
                return Err(
                    "nodeSnapshot: file-backed shape (path+exists+hash+file) cannot also carry native fields"
                        .to_string(),
                );
            }
        } else if native_enabled_form {
            // hash must be non-empty.
            if let Some(h) = &self.hash {
                if h.is_empty() {
                    return Err(
                        "nodeSnapshot: native enabled shape requires non-empty hash".to_string(),
                    );
                }
            }
            // Disallow file-backed fields.
            let file_extras = self.path.is_some()
                || self.exists.is_some()
                || self.file.is_some()
                || self.resolved_defaults.is_some()
                || self.message.is_some();
            if file_extras {
                return Err(
                    "nodeSnapshot: native enabled shape (enabled=true+hash+defaultAction+rules) cannot also carry file-backed fields"
                        .to_string(),
                );
            }
            if let Some(bh) = &self.base_hash {
                validate_non_empty_string("baseHash", bh)?;
            }
        } else if native_disabled_form {
            // Disallow everything else.
            let extras = self.path.is_some()
                || self.exists.is_some()
                || self.hash.is_some()
                || self.file.is_some()
                || self.resolved_defaults.is_some()
                || self.base_hash.is_some()
                || self.default_action.is_some()
                || self.rules.is_some()
                || self.constraints.is_some();
            if extras {
                return Err(
                    "nodeSnapshot: native disabled shape (enabled=false) must not carry any other fields"
                        .to_string(),
                );
            }
        } else {
            return Err(
                "nodeSnapshot: payload matches none of the supported shapes (file-backed, native-enabled, native-disabled)"
                    .to_string(),
            );
        }
        Ok(())
    }
}

// ---------- ExecApprovalsGetParamsSchema ----------

/// Empty request payload for reading local exec approval policy.
/// 对齐 TS: `Type.Object({}, { additionalProperties: false })`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecApprovalsGetParamsSchema {}

impl ExecApprovalsGetParamsSchema {
    /// 对齐 TS 的 `additionalProperties: false` —— 此结构体无字段，
    /// serde 默认拒绝未知键。
    pub fn validate(&self) -> Result<(), String> {
        Ok(())
    }
}

// ---------- ExecApprovalsSetParamsSchema ----------

/// Local exec approval policy write request with optional base hash guard.
/// 对齐 TS:
/// ```ts
/// export const ExecApprovalsSetParamsSchema = Type.Object({
///   file:     ExecApprovalsFileSchema,
///   baseHash: Type.Optional(NonEmptyString),
/// }, { additionalProperties: false });
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecApprovalsSetParamsSchema {
    pub file: ExecApprovalsFileSchema,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub base_hash: Option<NonEmptyString>,
}

impl ExecApprovalsSetParamsSchema {
    pub fn validate(&self) -> Result<(), String> {
        self.file.validate().map_err(|e| format!("file: {}", e))?;
        validate_optional_non_empty_string("baseHash", self.base_hash.as_deref())?;
        Ok(())
    }
}

// ---------- ExecApprovalsNodeGetParamsSchema ----------

/// Node-scoped request payload for reading exec approval policy.
/// 对齐 TS:
/// ```ts
/// export const ExecApprovalsNodeGetParamsSchema = Type.Object({
///   nodeId: NonEmptyString,
/// }, { additionalProperties: false });
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecApprovalsNodeGetParamsSchema {
    pub node_id: NonEmptyString,
}

impl ExecApprovalsNodeGetParamsSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_non_empty_string("nodeId", &self.node_id)?;
        Ok(())
    }
}

// ---------- NativeExecApprovalPolicySchema ----------

/// Writable host-native policy fields; the node remains the validation authority.
/// 对齐 TS:
/// ```ts
/// const NativeExecApprovalPolicySchema = Type.Object({
///   defaultAction: Type.Optional(NativeExecApprovalActionSchema),
///   rules:         Type.Array(NativeExecApprovalRuleSchema),
/// }, { additionalProperties: false });
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NativeExecApprovalPolicySchema {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_action: Option<String>, // 闭合字面量 ("allow" | "deny" | "prompt")
    /// Windows treats set as full replacement; omission would silently clear the rule list.
    pub rules: Vec<NativeExecApprovalRuleSchema>,
}

impl NativeExecApprovalPolicySchema {
    pub fn validate(&self) -> Result<(), String> {
        if let Some(da) = &self.default_action {
            if !is_valid_native_exec_approval_action(da) {
                return Err(format!(
                    "defaultAction: expected one of {:?}, got {:?}",
                    native_exec_approval_action::all(),
                    da
                ));
            }
        }
        for (i, r) in self.rules.iter().enumerate() {
            r.validate().map_err(|e| format!("rules[{}]: {}", i, e))?;
        }
        Ok(())
    }
}

// ---------- ExecApprovalsNodeSetParamsSchema ----------

/// Node-scoped write for exactly one file-backed or host-native approval owner.
/// 对齐 TS — `oneOf` 互斥：
///   形态 1：仅 `file`（不准 `native`）；形态 2：`native` + `baseHash`（不准 `file`）。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecApprovalsNodeSetParamsSchema {
    pub node_id: NonEmptyString,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file: Option<ExecApprovalsFileSchema>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub native: Option<NativeExecApprovalPolicySchema>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub base_hash: Option<NonEmptyString>,
}

impl ExecApprovalsNodeSetParamsSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_non_empty_string("nodeId", &self.node_id)?;
        // oneOf enforcement
        match (&self.file, (&self.native, &self.base_hash)) {
            (Some(_), (None, _)) => {
                if let Some(f) = &self.file {
                    f.validate().map_err(|e| format!("file: {}", e))?;
                }
            }
            (None, (Some(_), Some(_))) => {
                if let Some(n) = &self.native {
                    n.validate().map_err(|e| format!("native: {}", e))?;
                }
                if let Some(bh) = &self.base_hash {
                    validate_non_empty_string("baseHash", bh)?;
                }
            }
            _ => {
                return Err(
                    "ExecApprovalsNodeSetParamsSchema: must carry exactly one of {file} or {native,baseHash}"
                        .to_string(),
                );
            }
        }
        Ok(())
    }
}

// ---------- ExecApprovalGetParamsSchema ----------

/// Lookup request for one pending exec approval by id.
/// 对齐 TS:
/// ```ts
/// export const ExecApprovalGetParamsSchema = Type.Object({
///   id: NonEmptyString,
/// }, { additionalProperties: false });
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecApprovalGetParamsSchema {
    pub id: NonEmptyString,
}

impl ExecApprovalGetParamsSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_non_empty_string("id", &self.id)?;
        Ok(())
    }
}

// ---------- ExecApprovalPolicySnapshotSchema ----------

/// 对齐 TS:
/// ```ts
/// const ExecApprovalPolicySnapshotSchema = Type.Object({
///   security:        ExecApprovalPolicySecuritySchema, // (deny | allowlist | full)
///   ask:             Union(off | on-miss | always),
///   askFallback:     ExecApprovalPolicySecuritySchema,
///   autoAllowSkills: Type.Boolean(),
///   allowlistRules:  Type.Array(Type.Object({
///     pattern:    Type.String(),
///     argPattern: Type.Optional(Type.String()),
///     source:     Type.Optional(Type.Literal("allow-always")),
///   }, { additionalProperties: false })),
/// }, { additionalProperties: false });
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecApprovalAllowlistRuleSchema {
    pub pattern: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub arg_pattern: Option<String>,
    /// 对齐 TS: `Type.Optional(Type.Literal("allow-always"))`
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
}

impl ExecApprovalAllowlistRuleSchema {
    pub fn validate(&self) -> Result<(), String> {
        if self.pattern.is_empty() {
            return Err("pattern: expected non-empty string".to_string());
        }
        if let Some(src) = &self.source {
            if src != "allow-always" {
                return Err(format!(
                    "source: expected literal \"allow-always\", got {:?}",
                    src
                ));
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecApprovalPolicySnapshotSchema {
    pub security: String,
    pub ask: String,
    pub ask_fallback: String,
    pub auto_allow_skills: bool,
    pub allowlist_rules: Vec<ExecApprovalAllowlistRuleSchema>,
}

impl ExecApprovalPolicySnapshotSchema {
    pub fn validate(&self) -> Result<(), String> {
        if !is_valid_exec_security(&self.security) {
            return Err(format!(
                "security: expected one of {:?}, got {:?}",
                exec_security::all(),
                self.security
            ));
        }
        if !is_valid_exec_ask(&self.ask) {
            return Err(format!(
                "ask: expected one of {:?}, got {:?}",
                exec_ask::all(),
                self.ask
            ));
        }
        if !is_valid_exec_security(&self.ask_fallback) {
            return Err(format!(
                "askFallback: expected one of {:?}, got {:?}",
                exec_security::all(),
                self.ask_fallback
            ));
        }
        for (i, r) in self.allowlist_rules.iter().enumerate() {
            r.validate()
                .map_err(|e| format!("allowlistRules[{}]: {}", i, e))?;
        }
        Ok(())
    }
}

// ---------- ExecApprovalRequestParamsSchema ----------

/// Pending command execution approval request shown to reviewers.
/// 对齐 TS — 详见原文件第 277–360 行。
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecSystemRunPlanSchema {
    pub argv: Vec<String>,
    /// 对齐 TS: `Type.Union([Type.String(), Type.Null()])`
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cwd: Option<String>,
    pub command_text: String,
    /// 对齐 TS: `Type.Optional(Type.Union([Type.String(), Type.Null()]))`
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub command_preview: Option<String>,
    /// 对齐 TS: `Type.Union([Type.String(), Type.Null()])`
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub agent_id: Option<String>,
    /// 对齐 TS: `Type.Union([Type.String(), Type.Null()])`
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub session_key: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub policy_snapshot: Option<ExecApprovalPolicySnapshotSchema>,
    /// 对齐 TS: `Type.Optional(Union(Object | Null))`
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mutable_file_operand: Option<ExecMutableFileOperandSchema>,
}

impl ExecSystemRunPlanSchema {
    pub fn validate(&self) -> Result<(), String> {
        for (i, a) in self.argv.iter().enumerate() {
            if a.is_empty() {
                return Err(format!("argv[{}]: expected non-empty string", i));
            }
        }
        if self.command_text.is_empty() {
            return Err("commandText: expected non-empty string".to_string());
        }
        if let Some(ps) = &self.policy_snapshot {
            ps.validate()
                .map_err(|e| format!("policySnapshot: {}", e))?;
        }
        if let Some(mfo) = &self.mutable_file_operand {
            mfo.validate()
                .map_err(|e| format!("mutableFileOperand: {}", e))?;
        }
        Ok(())
    }
}

/// 对齐 TS:
/// `Type.Object({ argvIndex, path, sha256 }, { additionalProperties: false }) | null`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecMutableFileOperandSchema {
    /// 对齐 TS: `Type.Integer({ minimum: 0 })`
    pub argv_index: i64,
    pub path: String,
    pub sha256: String,
}

impl ExecMutableFileOperandSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_non_negative_integer("argvIndex", self.argv_index)?;
        if self.path.is_empty() {
            return Err("path: expected non-empty string".to_string());
        }
        if self.sha256.is_empty() {
            return Err("sha256: expected non-empty string".to_string());
        }
        Ok(())
    }
}

/// Command span indices into the `command` string (UTF-16 code units).
/// 对齐 TS:
/// ```ts
/// Type.Object({
///   startIndex: Type.Integer({ minimum: 0, description: "Inclusive UTF-16 code unit offset into command." }),
///   endIndex:   Type.Integer({ minimum: 1, description: "Exclusive UTF-16 code unit offset..." }),
/// }, { additionalProperties: false });
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecCommandSpanSchema {
    /// Inclusive UTF-16 code unit offset into command.
    pub start_index: i64,
    /// Exclusive UTF-16 code unit offset into command; must be greater than
    /// startIndex and no greater than command.length.
    pub end_index: i64,
}

impl ExecCommandSpanSchema {
    pub fn validate(&self, command_len_utf16: usize) -> Result<(), String> {
        validate_non_negative_integer("startIndex", self.start_index)?;
        validate_positive_integer("endIndex", self.end_index)?;
        if (self.end_index as usize) <= (self.start_index as usize) {
            return Err(format!(
                "commandSpan: endIndex ({}) must be greater than startIndex ({})",
                self.end_index, self.start_index
            ));
        }
        if (self.end_index as usize) > command_len_utf16 {
            return Err(format!(
                "commandSpan: endIndex ({}) exceeds command length ({})",
                self.end_index, command_len_utf16
            ));
        }
        Ok(())
    }
}

/// Pending command execution approval request shown to reviewers.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecApprovalRequestParamsSchema {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<NonEmptyString>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub command: Option<NonEmptyString>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub command_argv: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub system_run_plan: Option<ExecSystemRunPlanSchema>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub env: Option<BTreeMap<NonEmptyString, String>>,
    /// 对齐 TS: `Type.Optional(Type.Union([Type.String(), Type.Null()]))`
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cwd: Option<String>,
    /// 对齐 TS: `Type.Optional(Type.Union([NonEmptyString, Type.Null()]))`
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub node_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub host: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub security: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ask: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub warning_text: Option<String>,
    /// 对齐 TS: `Type.Array(String({enum: ["allow-always"]}), { minItems: 1, maxItems: 1 })`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub unavailable_decisions: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub command_spans: Option<Vec<ExecCommandSpanSchema>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub agent_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resolved_path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub session_key: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub turn_source_channel: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub turn_source_to: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub turn_source_account_id: Option<String>,
    /// 对齐 TS: `Type.Optional(Type.Union([Type.String(), Type.Number(), Type.Null()]))`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub turn_source_thread_id: Option<ExecTurnSourceThreadId>,
    /// Trusted approval-runtime metadata naming operator devices that may review
    /// this approval; ordinary Gateway clients may send the field, but the
    /// Gateway only binds it for internal approval-runtime requests.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub approval_reviewer_device_ids: Option<Vec<NonEmptyString>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub require_delivery_route: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub suppress_delivery: Option<bool>,
    /// 对齐 TS: `Type.Optional(Type.Integer({ minimum: 1 }))`
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub timeout_ms: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub two_phase: Option<bool>,
}

/// Either a string id, a numeric id, or a null value used for the originating thread.
/// 对齐 TS: `Type.Union([Type.String(), Type.Number(), Type.Null()])`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ExecTurnSourceThreadId {
    String(String),
    Number(f64),
    Null,
}

impl ExecTurnSourceThreadId {
    pub fn validate(&self) -> Result<(), String> {
        match self {
            ExecTurnSourceThreadId::String(_) => Ok(()),
            ExecTurnSourceThreadId::Number(n) => {
                if n.is_finite() {
                    Ok(())
                } else {
                    Err(format!(
                        "turnSourceThreadId: expected finite number, got {}",
                        n
                    ))
                }
            }
            ExecTurnSourceThreadId::Null => Ok(()),
        }
    }
}

impl ExecApprovalRequestParamsSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_optional_non_empty_string("id", self.id.as_deref())?;
        validate_optional_non_empty_string("command", self.command.as_deref())?;
        if let Some(arr) = &self.command_argv {
            for (i, s) in arr.iter().enumerate() {
                if s.is_empty() {
                    return Err(format!("commandArgv[{}]: expected non-empty string", i));
                }
            }
        }
        if let Some(srp) = &self.system_run_plan {
            srp.validate()
                .map_err(|e| format!("systemRunPlan: {}", e))?;
        }
        if let Some(env) = &self.env {
            for (k, _) in env {
                validate_non_empty_string("env key", k)?;
            }
        }
        if let Some(ud) = &self.unavailable_decisions {
            if ud.len() < 1 || ud.len() > 1 {
                return Err(format!(
                    "unavailableDecisions: expected length [1, 1], got {}",
                    ud.len()
                ));
            }
            for (i, v) in ud.iter().enumerate() {
                if v != "allow-always" {
                    return Err(format!(
                        "unavailableDecisions[{}]: expected literal \"allow-always\", got {:?}",
                        i, v
                    ));
                }
            }
        }
        if let Some(spans) = &self.command_spans {
            let cmd_len = self.command.as_deref().map(|s| s.len()).unwrap_or(0);
            for (i, s) in spans.iter().enumerate() {
                s.validate(cmd_len)
                    .map_err(|e| format!("commandSpans[{}]: {}", i, e))?;
            }
        }
        validate_optional_non_empty_string_list(
            "approvalReviewerDeviceIds",
            self.approval_reviewer_device_ids.as_ref(),
        )?;
        if let Some(n) = self.timeout_ms {
            validate_positive_integer("timeoutMs", n)?;
        }
        if let Some(tid) = &self.turn_source_thread_id {
            tid.validate()
                .map_err(|e| format!("turnSourceThreadId: {}", e))?;
        }
        Ok(())
    }
}

// ---------- ExecApprovalResolveParamsSchema ----------

/// Reviewer decision payload for one pending exec approval.
/// 对齐 TS:
/// ```ts
/// export const ExecApprovalResolveParamsSchema = Type.Object({
///   id:       NonEmptyString,
///   decision: NonEmptyString,
/// }, { additionalProperties: false });
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecApprovalResolveParamsSchema {
    pub id: NonEmptyString,
    pub decision: NonEmptyString,
}

impl ExecApprovalResolveParamsSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_non_empty_string("id", &self.id)?;
        validate_non_empty_string("decision", &self.decision)?;
        Ok(())
    }
}

// Owner-local wire types derived directly from local schema consts so the
// public plugin-sdk declaration graph never pulls in the ProtocolSchemas registry.
// 对应 TS:
//   export type ExecApprovalsGetParams     = Static<typeof ExecApprovalsGetParamsSchema>;
//   export type ExecApprovalsSetParams     = Static<typeof ExecApprovalsSetParamsSchema>;
//   export type ExecApprovalsNodeGetParams = Static<typeof ExecApprovalsNodeGetParamsSchema>;
//   export type ExecApprovalsNodeSnapshot  = Static<typeof ExecApprovalsNodeSnapshotSchema>;
//   export type ExecApprovalsNodeSetParams = Static<typeof ExecApprovalsNodeSetParamsSchema>;
//   export type ExecApprovalsSnapshot      = Static<typeof ExecApprovalsSnapshotSchema>;
//   export type ExecApprovalGetParams      = Static<typeof ExecApprovalGetParamsSchema>;
//   export type ExecApprovalRequestParams  = Static<typeof ExecApprovalRequestParamsSchema>;
//   export type ExecApprovalResolveParams  = Static<typeof ExecApprovalResolveParamsSchema>;
pub type ExecApprovalsGetParams = ExecApprovalsGetParamsSchema;
pub type ExecApprovalsSetParams = ExecApprovalsSetParamsSchema;
pub type ExecApprovalsNodeGetParams = ExecApprovalsNodeGetParamsSchema;
pub type ExecApprovalsNodeSnapshot = ExecApprovalsNodeSnapshotSchema;
pub type ExecApprovalsNodeSetParams = ExecApprovalsNodeSetParamsSchema;
pub type ExecApprovalsSnapshot = ExecApprovalsSnapshotSchema;
pub type ExecApprovalGetParams = ExecApprovalGetParamsSchema;
pub type ExecApprovalRequestParams = ExecApprovalRequestParamsSchema;
pub type ExecApprovalResolveParams = ExecApprovalResolveParamsSchema;
