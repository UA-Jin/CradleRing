// Gateway Protocol schema: config.
// 翻译自 packages/gateway-protocol/src/schema/config.ts
//
// Gateway config and update protocol schemas.
// These payloads carry raw config text plus optional delivery context so the
// gateway can report edits/restarts back to the originating channel.
//
// TS 用 TypeBox 定义 schema（运行时验证 + 类型）。
// Rust 用 serde struct + 验证函数实现等价的序列化/反序列化语义。

use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::primitives::NonEmptyString;

// ---------- 基础验证原语 (对齐 TypeBox: NonEmptyString) ----------

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

fn regex(pattern: &str) -> regex::Regex {
    regex::Regex::new(pattern).expect("invalid regex pattern compiled into config")
}

// ---------- 模块私有常量 ----------

/// 对齐 TS: `Type.String({ minLength: 1, maxLength: 1024, pattern: "^[A-Za-z0-9_./\\[\\]\\-*]+$" })`.
const CONFIG_SCHEMA_LOOKUP_PATH_PATTERN: &str = r"^[A-Za-z0-9_./\[\]\-*]+$";

/// Maximum length for a config schema lookup path string.
pub const CONFIG_SCHEMA_LOOKUP_PATH_MAX_LENGTH: usize = 1024;
pub const CONFIG_SCHEMA_LOOKUP_PATH_MIN_LENGTH: usize = 1;

/// Returns true when `path` matches the config schema lookup grammar.
pub fn is_valid_config_schema_lookup_path(path: &str) -> bool {
    let len = path.chars().count();
    if len < CONFIG_SCHEMA_LOOKUP_PATH_MIN_LENGTH || len > CONFIG_SCHEMA_LOOKUP_PATH_MAX_LENGTH {
        return false;
    }
    regex(CONFIG_SCHEMA_LOOKUP_PATH_PATTERN).is_match(path)
}

// ---------- ConfigDeliveryContextSchema ----------

/// Optional delivery context attached to config apply/patch/update payloads.
/// 对齐 TS:
///   `Type.Object({
///       channel:    Type.Optional(Type.String()),
///       to:         Type.Optional(Type.String()),
///       accountId:  Type.Optional(Type.String()),
///       threadId:   Type.Optional(Type.Union([Type.String(), Type.Number()])),
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfigDeliveryContextSchema {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub channel: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub to: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub account_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub thread_id: Option<ConfigThreadId>,
}

impl ConfigDeliveryContextSchema {
    pub fn validate(&self) -> Result<(), String> {
        Ok(())
    }
}

/// `threadId` field accepts either a string or number; modeled as a Rust
/// `untagged` enum to mirror `Type.Union([Type.String(), Type.Number()])`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ConfigThreadId {
    Text(String),
    Number(f64),
}

// ---------- ConfigGetParamsSchema ----------

/// Empty request payload for reading the current raw config.
/// 对齐 TS: `ConfigGetParamsSchema = Type.Object({}, { additionalProperties: false })`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ConfigGetParamsSchema {}

impl ConfigGetParamsSchema {
    pub fn validate(&self) -> Result<(), String> {
        Ok(())
    }
}

// ---------- ConfigSetParamsSchema ----------

/// Full raw config replacement request with optional base hash guard.
/// 对齐 TS:
///   `Type.Object({
///       raw:      NonEmptyString,
///       baseHash: Type.Optional(NonEmptyString),
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfigSetParamsSchema {
    pub raw: NonEmptyString,
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "baseHash")]
    pub base_hash: Option<NonEmptyString>,
}

impl ConfigSetParamsSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_non_empty_string("raw", &self.raw)?;
        if let Some(bh) = &self.base_hash {
            validate_non_empty_string("baseHash", bh)?;
        }
        Ok(())
    }
}

// ---------- ConfigApplyLikeParamsSchema (shared between Apply / Patch) ----------

/// Shared apply/patch payload carrying raw config, optional base hash, and
/// optional delivery context. Patch adds `replacePaths`.
/// 对齐 TS:
///   `Type.Object({
///       raw:             NonEmptyString,
///       baseHash:        Type.Optional(NonEmptyString),
///       sessionKey:      Type.Optional(Type.String()),
///       deliveryContext: Type.Optional(ConfigDeliveryContextSchema),
///       note:            Type.Optional(Type.String()),
///       restartDelayMs:  Type.Optional(Type.Integer({ minimum: 0 })),
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfigApplyLikeParamsSchema {
    pub raw: NonEmptyString,
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "baseHash")]
    pub base_hash: Option<NonEmptyString>,
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "sessionKey")]
    pub session_key: Option<String>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "deliveryContext"
    )]
    pub delivery_context: Option<ConfigDeliveryContextSchema>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "restartDelayMs"
    )]
    pub restart_delay_ms: Option<i64>,
}

impl ConfigApplyLikeParamsSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_non_empty_string("raw", &self.raw)?;
        if let Some(bh) = &self.base_hash {
            validate_non_empty_string("baseHash", bh)?;
        }
        if let Some(delay) = self.restart_delay_ms {
            if delay < 0 {
                return Err(format!(
                    "restartDelayMs: expected >= 0, got {}",
                    delay
                ));
            }
        }
        if let Some(dc) = &self.delivery_context {
            dc.validate().map_err(|e| format!("deliveryContext: {}", e))?;
        }
        Ok(())
    }
}

// ---------- ConfigApplyParamsSchema ----------

/// Raw config apply request that may schedule a restart.
/// 对齐 TS: `ConfigApplyParamsSchema = ConfigApplyLikeParamsSchema`.
pub type ConfigApplyParamsSchema = ConfigApplyLikeParamsSchema;

// ---------- ConfigPatchParamsSchema ----------

/// Raw config patch request that may schedule a restart, optionally scoped to
/// `replacePaths`.
/// 对齐 TS:
///   `Type.Object({
///       ...ConfigApplyLikeParamProperties,
///       replacePaths: Type.Optional(Type.Array(NonEmptyString, { maxItems: 256 })),
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfigPatchParamsSchema {
    pub raw: NonEmptyString,
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "baseHash")]
    pub base_hash: Option<NonEmptyString>,
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "sessionKey")]
    pub session_key: Option<String>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "deliveryContext"
    )]
    pub delivery_context: Option<ConfigDeliveryContextSchema>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "restartDelayMs"
    )]
    pub restart_delay_ms: Option<i64>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "replacePaths"
    )]
    pub replace_paths: Option<Vec<NonEmptyString>>,
}

impl ConfigPatchParamsSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_non_empty_string("raw", &self.raw)?;
        if let Some(bh) = &self.base_hash {
            validate_non_empty_string("baseHash", bh)?;
        }
        if let Some(delay) = self.restart_delay_ms {
            if delay < 0 {
                return Err(format!(
                    "restartDelayMs: expected >= 0, got {}",
                    delay
                ));
            }
        }
        if let Some(dc) = &self.delivery_context {
            dc.validate().map_err(|e| format!("deliveryContext: {}", e))?;
        }
        if let Some(paths) = &self.replace_paths {
            if paths.len() > 256 {
                return Err(format!(
                    "replacePaths: maxItems 256, got {}",
                    paths.len()
                ));
            }
            for (i, p) in paths.iter().enumerate() {
                validate_non_empty_string(&format!("replacePaths[{}]", i), p)?;
            }
        }
        Ok(())
    }
}

// ---------- ConfigSchemaParamsSchema ----------

/// Empty request payload for fetching the generated config schema.
/// 对齐 TS: `ConfigSchemaParamsSchema = Type.Object({}, { additionalProperties: false })`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ConfigSchemaParamsSchema {}

impl ConfigSchemaParamsSchema {
    pub fn validate(&self) -> Result<(), String> {
        Ok(())
    }
}

// ---------- ConfigSchemaLookupParamsSchema ----------

/// Schema lookup request for one config path.
/// 对齐 TS:
///   `Type.Object({
///       path: ConfigSchemaLookupPathString,
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigSchemaLookupParamsSchema {
    pub path: String,
}

impl ConfigSchemaLookupParamsSchema {
    pub fn validate(&self) -> Result<(), String> {
        if !is_valid_config_schema_lookup_path(&self.path) {
            return Err(format!(
                "path: invalid config schema lookup path: {:?}",
                self.path
            ));
        }
        Ok(())
    }
}

// ---------- UpdateStatusParamsSchema ----------

/// Empty request payload for checking update/restart status.
/// 对齐 TS: `UpdateStatusParamsSchema = Type.Object({}, { additionalProperties: false })`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UpdateStatusParamsSchema {}

impl UpdateStatusParamsSchema {
    pub fn validate(&self) -> Result<(), String> {
        Ok(())
    }
}

// ---------- UpdateRunParamsSchema ----------

/// Request payload for running an update/restart flow with optional channel
/// delivery context.
/// 对齐 TS:
///   `Type.Object({
///       sessionKey:         Type.Optional(Type.String()),
///       deliveryContext:    Type.Optional(ConfigDeliveryContextSchema),
///       note:               Type.Optional(Type.String()),
///       continuationMessage: Type.Optional(Type.String()),
///       restartDelayMs:     Type.Optional(Type.Integer({ minimum: 0 })),
///       timeoutMs:          Type.Optional(Type.Integer({ minimum: 1 })),
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateRunParamsSchema {
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "sessionKey")]
    pub session_key: Option<String>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "deliveryContext"
    )]
    pub delivery_context: Option<ConfigDeliveryContextSchema>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "continuationMessage"
    )]
    pub continuation_message: Option<String>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "restartDelayMs"
    )]
    pub restart_delay_ms: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "timeoutMs")]
    pub timeout_ms: Option<i64>,
}

impl UpdateRunParamsSchema {
    pub fn validate(&self) -> Result<(), String> {
        if let Some(delay) = self.restart_delay_ms {
            if delay < 0 {
                return Err(format!(
                    "restartDelayMs: expected >= 0, got {}",
                    delay
                ));
            }
        }
        if let Some(t) = self.timeout_ms {
            if t < 1 {
                return Err(format!("timeoutMs: expected >= 1, got {}", t));
            }
        }
        if let Some(dc) = &self.delivery_context {
            dc.validate().map_err(|e| format!("deliveryContext: {}", e))?;
        }
        Ok(())
    }
}

// ---------- ConfigUiHintSchema ----------

/// UI metadata attached to config schema paths.
/// 对齐 TS:
///   `Type.Object({
///       label:        Type.Optional(Type.String()),
///       help:         Type.Optional(Type.String()),
///       tags:         Type.Optional(Type.Array(Type.String())),
///       group:        Type.Optional(Type.String()),
///       order:        Type.Optional(Type.Integer()),
///       advanced:     Type.Optional(Type.Boolean()),
///       sensitive:    Type.Optional(Type.Boolean()),
///       placeholder:  Type.Optional(Type.String()),
///       itemTemplate: Type.Optional(Type.Unknown()),
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfigUiHintSchema {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub help: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub group: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub order: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub advanced: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sensitive: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub placeholder: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "itemTemplate")]
    pub item_template: Option<Value>,
}

impl ConfigUiHintSchema {
    pub fn validate(&self) -> Result<(), String> {
        Ok(())
    }
}

// ---------- ConfigSchemaResponseSchema ----------

/// Full generated config schema response.
/// 对齐 TS:
///   `Type.Object({
///       schema:      Type.Unknown(),
///       uiHints:     Type.Record(Type.String(), ConfigUiHintSchema),
///       version:     NonEmptyString,
///       generatedAt: NonEmptyString,
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfigSchemaResponseSchema {
    pub schema: Value,
    #[serde(rename = "uiHints")]
    pub ui_hints: std::collections::BTreeMap<String, ConfigUiHintSchema>,
    pub version: NonEmptyString,
    #[serde(rename = "generatedAt")]
    pub generated_at: NonEmptyString,
}

impl ConfigSchemaResponseSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_non_empty_string("version", &self.version)?;
        validate_non_empty_string("generatedAt", &self.generated_at)?;
        for (k, v) in &self.ui_hints {
            v.validate()
                .map_err(|e| format!("uiHints[{}]: {}", k, e))?;
        }
        Ok(())
    }
}

// ---------- ConfigSchemaLookupChildSchema ----------

/// Child entry returned when looking up a config schema path.
/// 对齐 TS:
///   `Type.Object({
///       key:         NonEmptyString,
///       path:        NonEmptyString,
///       type:        Type.Optional(Type.Union([Type.String(), Type.Array(Type.String())])),
///       required:    Type.Boolean(),
///       hasChildren: Type.Boolean(),
///       reloadKind:  Type.Optional(Type.Union([Type.Literal("restart"),
///                                              Type.Literal("hot"),
///                                              Type.Literal("none")])),
///       hint:        Type.Optional(ConfigUiHintSchema),
///       hintPath:    Type.Optional(Type.String()),
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfigSchemaLookupChildSchema {
    pub key: NonEmptyString,
    pub path: NonEmptyString,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "type"
    )]
    pub schema_type: Option<ConfigSchemaChildType>,
    pub required: bool,
    #[serde(rename = "hasChildren")]
    pub has_children: bool,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "reloadKind"
    )]
    pub reload_kind: Option<ConfigReloadKind>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hint: Option<ConfigUiHintSchema>,
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "hintPath")]
    pub hint_path: Option<String>,
}

impl ConfigSchemaLookupChildSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_non_empty_string("key", &self.key)?;
        validate_non_empty_string("path", &self.path)?;
        if let Some(h) = &self.hint {
            h.validate().map_err(|e| format!("hint: {}", e))?;
        }
        Ok(())
    }
}

/// Child entry `type` accepts a single string or a list of strings.
/// 对齐 TS: `Type.Optional(Type.Union([Type.String(), Type.Array(Type.String())]))`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ConfigSchemaChildType {
    Single(String),
    Multi(Vec<String>),
}

/// Reload classification reported on schema lookup children / results.
/// 对齐 TS: `Type.Union([Type.Literal("restart"), Type.Literal("hot"), Type.Literal("none")])`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ConfigReloadKind {
    #[serde(rename = "restart")]
    Restart,
    #[serde(rename = "hot")]
    Hot,
    #[serde(rename = "none")]
    None,
}

impl ConfigReloadKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Restart => "restart",
            Self::Hot => "hot",
            Self::None => "none",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "restart" => Some(Self::Restart),
            "hot" => Some(Self::Hot),
            "none" => Some(Self::None),
            _ => None,
        }
    }
}

pub fn is_valid_config_reload_kind(s: &str) -> bool {
    ConfigReloadKind::from_str(s).is_some()
}

// ---------- ConfigSchemaLookupResultSchema ----------

/// Schema lookup response for one config path and its immediate children.
/// 对齐 TS:
///   `Type.Object({
///       path:       NonEmptyString,
///       schema:     Type.Unknown(),
///       reloadKind: Type.Optional(Type.Union([Type.Literal("restart"),
///                                             Type.Literal("hot"),
///                                             Type.Literal("none")])),
///       hint:       Type.Optional(ConfigUiHintSchema),
///       hintPath:   Type.Optional(Type.String()),
///       children:   Type.Array(ConfigSchemaLookupChildSchema),
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfigSchemaLookupResultSchema {
    pub path: NonEmptyString,
    pub schema: Value,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "reloadKind"
    )]
    pub reload_kind: Option<ConfigReloadKind>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hint: Option<ConfigUiHintSchema>,
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "hintPath")]
    pub hint_path: Option<String>,
    pub children: Vec<ConfigSchemaLookupChildSchema>,
}

impl ConfigSchemaLookupResultSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_non_empty_string("path", &self.path)?;
        if let Some(h) = &self.hint {
            h.validate().map_err(|e| format!("hint: {}", e))?;
        }
        for (i, child) in self.children.iter().enumerate() {
            child
                .validate()
                .map_err(|e| format!("children[{}]: {}", i, e))?;
        }
        Ok(())
    }
}

// Wire type aliases (对标 TS `type X = Static<typeof YSchema>`)
pub type ConfigGetParams = ConfigGetParamsSchema;
pub type ConfigSetParams = ConfigSetParamsSchema;
pub type ConfigApplyParams = ConfigApplyParamsSchema;
pub type ConfigPatchParams = ConfigPatchParamsSchema;
pub type ConfigSchemaParams = ConfigSchemaParamsSchema;
pub type ConfigSchemaLookupParams = ConfigSchemaLookupParamsSchema;
pub type ConfigSchemaResponse = ConfigSchemaResponseSchema;
pub type ConfigSchemaLookupResult = ConfigSchemaLookupResultSchema;
pub type UpdateStatusParams = UpdateStatusParamsSchema;
pub type UpdateRunParams = UpdateRunParamsSchema;