// Gateway Protocol schema: plugins.
// 翻译自 packages/gateway-protocol/src/schema/plugins.ts
//
// Plugin control-surface protocol schemas.
//
// These payloads let the gateway expose plugin-provided UI actions without
// baking plugin-specific payload shapes into the core protocol.
//
// TS 用 TypeBox 定义 schema（运行时验证 + 类型）。
// Rust 用 serde struct + 验证函数实现等价的序列化/反序列化语义。

use serde::{Deserialize, Serialize};

// ============================================================================
// PluginJsonValueSchema (project-wide opaque JSON placeholder)
// ============================================================================

/// Arbitrary plugin-owned JSON payload carried opaquely through the gateway.
/// 对齐 TS: `PluginJsonValueSchema = Type.Unknown()`.
pub type PluginJsonValueSchema = serde_json::Value;

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

#[allow(dead_code)]
fn validate_positive_integer(field: &str, n: i64) -> Result<(), String> {
    if n >= 1 {
        Ok(())
    } else {
        Err(format!("{}: expected integer >= 1, got {}", field, n))
    }
}

fn validate_non_negative_number(field: &str, n: f64) -> Result<(), String> {
    if n >= 0.0 {
        Ok(())
    } else {
        Err(format!("{}: expected number >= 0, got {}", field, n))
    }
}

fn validate_optional_non_negative_number(field: &str, n: Option<f64>) -> Result<(), String> {
    if let Some(v) = n {
        validate_non_negative_number(field, v)?;
    }
    Ok(())
}

#[allow(dead_code)]
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

fn validate_optional_non_empty_string_list(field: &str, values: Option<&Vec<String>>) -> Result<(), String> {
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

// ============================================================================
// PluginControlUiDescriptorSchema — surface enum
// ============================================================================

/// Surface discriminator for `PluginControlUiDescriptorSchema`.
/// 对齐 TS: `Type.Union([Type.Literal("session"), Type.Literal("tool"), Type.Literal("run"), Type.Literal("settings")])`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PluginControlUiDescriptorSurface {
    Session,
    Tool,
    Run,
    Settings,
}

impl PluginControlUiDescriptorSurface {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Session => "session",
            Self::Tool => "tool",
            Self::Run => "run",
            Self::Settings => "settings",
        }
    }
}

pub fn is_valid_plugin_control_ui_descriptor_surface(s: &str) -> bool {
    matches!(s, "session" | "tool" | "run" | "settings")
}

// ============================================================================
// PluginCatalogInstallActionSchema — variant enums
// ============================================================================

/// Source discriminator for `PluginCatalogInstallActionSchema`.
/// 对齐 TS: `Type.Literal("clawhub")` / `Type.Literal("official")`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PluginCatalogInstallSource {
    Clawhub,
    Official,
}

impl PluginCatalogInstallSource {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Clawhub => "clawhub",
            Self::Official => "official",
        }
    }
}

pub fn is_valid_plugin_catalog_install_source(s: &str) -> bool {
    matches!(s, "clawhub" | "official")
}

/// Family discriminator for `PluginSearchPackageSchema`.
/// 对齐 TS: `Type.Union([Type.Literal("code-plugin"), Type.Literal("bundle-plugin")])`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum PluginSearchPackageFamily {
    #[serde(rename = "code-plugin")]
    CodePlugin,
    #[serde(rename = "bundle-plugin")]
    BundlePlugin,
}

impl PluginSearchPackageFamily {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::CodePlugin => "code-plugin",
            Self::BundlePlugin => "bundle-plugin",
        }
    }
}

pub fn is_valid_plugin_search_package_family(s: &str) -> bool {
    matches!(s, "code-plugin" | "bundle-plugin")
}

/// Channel discriminator for `PluginSearchPackageSchema`.
/// 对齐 TS: `Type.Union([Type.Literal("official"), Type.Literal("community"), Type.Literal("private")])`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PluginSearchPackageChannel {
    Official,
    Community,
    Private,
}

impl PluginSearchPackageChannel {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Official => "official",
            Self::Community => "community",
            Self::Private => "private",
        }
    }
}

pub fn is_valid_plugin_search_package_channel(s: &str) -> bool {
    matches!(s, "official" | "community" | "private")
}

/// State discriminator for `PluginCatalogEntrySchema`.
/// 对齐 TS: `Type.Union([Type.Literal("enabled"), Type.Literal("disabled"), Type.Literal("not-installed"), Type.Literal("error")])`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum PluginCatalogEntryState {
    Enabled,
    Disabled,
    #[serde(rename = "not-installed")]
    NotInstalled,
    Error,
}

impl PluginCatalogEntryState {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Enabled => "enabled",
            Self::Disabled => "disabled",
            Self::NotInstalled => "not-installed",
            Self::Error => "error",
        }
    }
}

pub fn is_valid_plugin_catalog_entry_state(s: &str) -> bool {
    matches!(s, "enabled" | "disabled" | "not-installed" | "error")
}

// ============================================================================
// Module-private constants
// ============================================================================

/// Search limit bounds for `PluginsSearchParamsSchema`.
const PLUGINS_SEARCH_LIMIT_MIN: i64 = 1;
const PLUGINS_SEARCH_LIMIT_MAX: i64 = 100;

// ============================================================================
// PluginControlUiDescriptorSchema
// ============================================================================

/// Descriptor for one plugin-provided control UI action or surface.
/// 对齐 TS:
///   `Type.Object({
///      id: NonEmptyString,
///      pluginId: NonEmptyString,
///      pluginName: Type.Optional(NonEmptyString),
///      surface: Type.Union([Type.Literal("session"), Type.Literal("tool"),
///                            Type.Literal("run"), Type.Literal("settings")]),
///      label: NonEmptyString,
///      description: Type.Optional(Type.String()),
///      placement: Type.Optional(Type.String()),
///      schema: Type.Optional(PluginJsonValueSchema),
///      requiredScopes: Type.Optional(Type.Array(NonEmptyString)),
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginControlUiDescriptorSchema {
    pub id: String,
    pub plugin_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub plugin_name: Option<String>,
    pub surface: PluginControlUiDescriptorSurface,
    pub label: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub placement: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub schema: Option<PluginJsonValueSchema>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub required_scopes: Option<Vec<String>>,
}

impl PluginControlUiDescriptorSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_non_empty_string("id", &self.id)?;
        validate_non_empty_string("pluginId", &self.plugin_id)?;
        validate_optional_non_empty_string("pluginName", self.plugin_name.as_deref())?;
        validate_non_empty_string("label", &self.label)?;
        validate_optional_non_empty_string_list("requiredScopes", self.required_scopes.as_ref())?;
        Ok(())
    }
}

// ============================================================================
// PluginsUiDescriptors schemas
// ============================================================================

/// Empty request payload for listing plugin UI descriptors.
/// 对齐 TS: `Type.Object({}, { additionalProperties: false })`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginsUiDescriptorsParamsSchema {}

impl PluginsUiDescriptorsParamsSchema {
    pub fn validate(&self) -> Result<(), String> {
        Ok(())
    }
}

/// Response payload containing all plugin UI descriptors visible to the client.
/// 对齐 TS:
///   `Type.Object({
///      ok: Type.Literal(true),
///      descriptors: Type.Array(PluginControlUiDescriptorSchema),
///   }, { additionalProperties: false })`.
///
/// `ok` is constrained to the literal `true`; runtime validate checks the value.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginsUiDescriptorsResultSchema {
    pub ok: bool,
    pub descriptors: Vec<PluginControlUiDescriptorSchema>,
}

impl PluginsUiDescriptorsResultSchema {
    pub fn validate(&self) -> Result<(), String> {
        if !self.ok {
            return Err("ok: expected literal true".to_string());
        }
        for (i, d) in self.descriptors.iter().enumerate() {
            d.validate().map_err(|e| format!("descriptors[{}]: {}", i, e))?;
        }
        Ok(())
    }
}

// ============================================================================
// PluginsSessionAction schemas
// ============================================================================

/// Request payload for invoking one plugin-owned session action.
/// 对齐 TS:
///   `Type.Object({
///      pluginId: NonEmptyString,
///      actionId: NonEmptyString,
///      sessionKey: Type.Optional(NonEmptyString),
///      payload: Type.Optional(PluginJsonValueSchema),
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginsSessionActionParamsSchema {
    pub plugin_id: String,
    pub action_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub session_key: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub payload: Option<PluginJsonValueSchema>,
}

impl PluginsSessionActionParamsSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_non_empty_string("pluginId", &self.plugin_id)?;
        validate_non_empty_string("actionId", &self.action_id)?;
        validate_optional_non_empty_string("sessionKey", self.session_key.as_deref())?;
        Ok(())
    }
}

/// Successful plugin action result, optionally continuing the agent turn.
/// 对齐 TS:
///   `Type.Object({
///      ok: Type.Literal(true),
///      result: Type.Optional(PluginJsonValueSchema),
///      continueAgent: Type.Optional(Type.Boolean()),
///      reply: Type.Optional(PluginJsonValueSchema),
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginsSessionActionSuccessResultSchema {
    pub ok: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub result: Option<PluginJsonValueSchema>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub continue_agent: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reply: Option<PluginJsonValueSchema>,
}

impl PluginsSessionActionSuccessResultSchema {
    pub fn validate(&self) -> Result<(), String> {
        if !self.ok {
            return Err("ok: expected literal true".to_string());
        }
        Ok(())
    }
}

/// Failed plugin action result with plugin-owned detail payload.
/// 对齐 TS:
///   `Type.Object({
///      ok: Type.Literal(false),
///      error: Type.String(),
///      code: Type.Optional(Type.String()),
///      details: Type.Optional(PluginJsonValueSchema),
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginsSessionActionFailureResultSchema {
    pub ok: bool,
    pub error: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub details: Option<PluginJsonValueSchema>,
}

impl PluginsSessionActionFailureResultSchema {
    pub fn validate(&self) -> Result<(), String> {
        if self.ok {
            return Err("ok: expected literal false".to_string());
        }
        Ok(())
    }
}

/// Discriminated plugin action result returned to gateway clients.
/// 对齐 TS: `Type.Union([PluginsSessionActionSuccessResultSchema, PluginsSessionActionFailureResultSchema])`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum PluginsSessionActionResultSchema {
    Success(PluginsSessionActionSuccessResultSchema),
    Failure(PluginsSessionActionFailureResultSchema),
}

impl PluginsSessionActionResultSchema {
    pub fn validate(&self) -> Result<(), String> {
        match self {
            Self::Success(s) => s.validate(),
            Self::Failure(f) => f.validate(),
        }
    }
}

// ============================================================================
// PluginCatalogInstallAction — discriminated union
// ============================================================================

/// ClawHub-backed install action for one catalog entry.
/// 对齐 TS: `Type.Object({ source: Type.Literal("clawhub"), packageName: NonEmptyString }, ...)`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginCatalogClawHubInstallSchema {
    pub source: PluginCatalogInstallSourceClawhub,
    pub package_name: String,
}

impl PluginCatalogClawHubInstallSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_non_empty_string("packageName", &self.package_name)?;
        Ok(())
    }
}

// ClawHub literal marker for `source`.
/// 对齐 TS: `Type.Literal("clawhub")`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PluginCatalogInstallSourceClawhub {
    #[serde(rename = "clawhub")]
    Clawhub,
}

impl PluginCatalogInstallSourceClawhub {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Clawhub => "clawhub",
        }
    }
}

/// Official-catalog install action for one catalog entry.
/// 对齐 TS: `Type.Object({ source: Type.Literal("official"), pluginId: NonEmptyString }, ...)`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginCatalogOfficialInstallSchema {
    pub source: PluginCatalogInstallSourceOfficial,
    pub plugin_id: String,
}

impl PluginCatalogOfficialInstallSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_non_empty_string("pluginId", &self.plugin_id)?;
        Ok(())
    }
}

// Official literal marker for `source`.
/// 对齐 TS: `Type.Literal("official")`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PluginCatalogInstallSourceOfficial {
    #[serde(rename = "official")]
    Official,
}

impl PluginCatalogInstallSourceOfficial {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Official => "official",
        }
    }
}

/// Discriminated union of catalog install actions (clawhub / official).
/// 对齐 TS: `Type.Union([PluginCatalogClawHubInstallSchema, PluginCatalogOfficialInstallSchema])`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum PluginCatalogInstallActionSchema {
    Clawhub(PluginCatalogClawHubInstallSchema),
    Official(PluginCatalogOfficialInstallSchema),
}

impl PluginCatalogInstallActionSchema {
    pub fn validate(&self) -> Result<(), String> {
        match self {
            Self::Clawhub(c) => c.validate(),
            Self::Official(o) => o.validate(),
        }
    }
}

// ============================================================================
// PluginCatalogEntrySchema
// ============================================================================

/// Cold control-plane representation of an installed or available plugin.
/// 对齐 TS:
///   `Type.Object({
///      id: NonEmptyString,
///      name: NonEmptyString,
///      packageName: Type.Optional(NonEmptyString),
///      description: Type.Optional(Type.String()),
///      version: Type.Optional(NonEmptyString),
///      kind: Type.Optional(Type.Array(NonEmptyString)),
///      origin: Type.Optional(NonEmptyString),
///      installed: Type.Boolean(),
///      enabled: Type.Boolean(),
///      state: Type.Union([Type.Literal("enabled"), Type.Literal("disabled"),
///                          Type.Literal("not-installed"), Type.Literal("error")]),
///      featured: Type.Optional(Type.Boolean()),
///      order: Type.Optional(Type.Number()),
///      install: Type.Optional(PluginCatalogInstallActionSchema),
///      error: Type.Optional(Type.String()),
///      category: Type.Optional(NonEmptyString),
///      removable: Type.Optional(Type.Boolean()),
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginCatalogEntrySchema {
    pub id: String,
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub package_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub kind: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub origin: Option<String>,
    pub installed: bool,
    pub enabled: bool,
    pub state: PluginCatalogEntryState,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub featured: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub order: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub install: Option<PluginCatalogInstallActionSchema>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    /// Coarse manifest-derived grouping (channel, provider, memory, ...) for catalog UIs.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,
    /// True when the plugin has an install record and can be removed via plugins.uninstall.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub removable: Option<bool>,
}

impl PluginCatalogEntrySchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_non_empty_string("id", &self.id)?;
        validate_non_empty_string("name", &self.name)?;
        validate_optional_non_empty_string("packageName", self.package_name.as_deref())?;
        validate_optional_non_empty_string_list("kind", self.kind.as_ref())?;
        validate_optional_non_empty_string("origin", self.origin.as_deref())?;
        validate_optional_non_empty_string("category", self.category.as_deref())?;
        if let Some(install) = &self.install {
            install
                .validate()
                .map_err(|e| format!("install: {}", e))?;
        }
        Ok(())
    }
}

// ============================================================================
// PluginsList schemas
// ============================================================================

/// Empty request payload for the cold plugin catalog.
/// 对齐 TS: `Type.Object({}, { additionalProperties: false })`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginsListParamsSchema {}

impl PluginsListParamsSchema {
    pub fn validate(&self) -> Result<(), String> {
        Ok(())
    }
}

/// Installed and curated plugin catalog visible to the current gateway client.
/// 对齐 TS:
///   `Type.Object({
///      plugins: Type.Array(PluginCatalogEntrySchema),
///      diagnostics: Type.Array(Type.Unknown()),
///      mutationAllowed: Type.Boolean(),
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginsListResultSchema {
    pub plugins: Vec<PluginCatalogEntrySchema>,
    pub diagnostics: Vec<PluginJsonValueSchema>,
    pub mutation_allowed: bool,
}

impl PluginsListResultSchema {
    pub fn validate(&self) -> Result<(), String> {
        for (i, p) in self.plugins.iter().enumerate() {
            p.validate().map_err(|e| format!("plugins[{}]: {}", i, e))?;
        }
        Ok(())
    }
}

// ============================================================================
// PluginsSearch schemas
// ============================================================================

/// Request payload for searching installable ClawHub plugin families.
/// 对齐 TS:
///   `Type.Object({
///      query: NonEmptyString,
///      limit: Type.Optional(Type.Integer({ minimum: 1, maximum: 100 })),
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginsSearchParamsSchema {
    pub query: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub limit: Option<i64>,
}

impl PluginsSearchParamsSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_non_empty_string("query", &self.query)?;
        if let Some(limit) = self.limit {
            validate_integer_in_range(
                "limit",
                limit,
                PLUGINS_SEARCH_LIMIT_MIN,
                PLUGINS_SEARCH_LIMIT_MAX,
            )?;
        }
        Ok(())
    }
}

/// ClawHub package fields exposed by plugin search.
/// 对齐 TS:
///   `Type.Object({
///      name: NonEmptyString,
///      displayName: NonEmptyString,
///      family: Type.Union([Type.Literal("code-plugin"), Type.Literal("bundle-plugin")]),
///      channel: Type.Union([Type.Literal("official"), Type.Literal("community"),
///                            Type.Literal("private")]),
///      isOfficial: Type.Boolean(),
///      summary: Type.Optional(Type.String()),
///      latestVersion: Type.Optional(NonEmptyString),
///      runtimeId: Type.Optional(NonEmptyString),
///      downloads: Type.Optional(Type.Number({ minimum: 0 })),
///      verificationTier: Type.Optional(NonEmptyString),
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginSearchPackageSchema {
    pub name: String,
    pub display_name: String,
    pub family: PluginSearchPackageFamily,
    pub channel: PluginSearchPackageChannel,
    pub is_official: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub latest_version: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub runtime_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub downloads: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub verification_tier: Option<String>,
}

impl PluginSearchPackageSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_non_empty_string("name", &self.name)?;
        validate_non_empty_string("displayName", &self.display_name)?;
        validate_optional_non_empty_string("latestVersion", self.latest_version.as_deref())?;
        validate_optional_non_empty_string("runtimeId", self.runtime_id.as_deref())?;
        validate_optional_non_negative_number("downloads", self.downloads)?;
        validate_optional_non_empty_string("verificationTier", self.verification_tier.as_deref())?;
        Ok(())
    }
}

/// Ranked ClawHub plugin search hit.
/// 对齐 TS:
///   `Type.Object({
///      score: Type.Number(),
///      package: PluginSearchPackageSchema,
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginSearchResultEntrySchema {
    pub score: f64,
    pub package: PluginSearchPackageSchema,
}

impl PluginSearchResultEntrySchema {
    pub fn validate(&self) -> Result<(), String> {
        self.package
            .validate()
            .map_err(|e| format!("package: {}", e))?;
        Ok(())
    }
}

/// Ranked installable plugin packages matching the query.
/// 对齐 TS: `Type.Object({ results: Type.Array(PluginSearchResultEntrySchema) }, ...)`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginsSearchResultSchema {
    pub results: Vec<PluginSearchResultEntrySchema>,
}

impl PluginsSearchResultSchema {
    pub fn validate(&self) -> Result<(), String> {
        for (i, r) in self.results.iter().enumerate() {
            r.validate().map_err(|e| format!("results[{}]: {}", i, e))?;
        }
        Ok(())
    }
}

// ============================================================================
// PluginsInstall — discriminated union params
// ============================================================================

/// ClawHub install params variant.
/// 对齐 TS:
///   `Type.Object({
///      source: Type.Literal("clawhub"),
///      packageName: NonEmptyString,
///      version: Type.Optional(NonEmptyString),
///      acknowledgeClawHubRisk: Type.Optional(Type.Boolean()),
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginsInstallClawhubParamsSchema {
    pub source: PluginCatalogInstallSourceClawhub,
    pub package_name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub acknowledge_claw_hub_risk: Option<bool>,
}

impl PluginsInstallClawhubParamsSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_non_empty_string("packageName", &self.package_name)?;
        Ok(())
    }
}

/// Official install params variant.
/// 对齐 TS:
///   `Type.Object({
///      source: Type.Literal("official"),
///      pluginId: NonEmptyString,
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginsInstallOfficialParamsSchema {
    pub source: PluginCatalogInstallSourceOfficial,
    pub plugin_id: String,
}

impl PluginsInstallOfficialParamsSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_non_empty_string("pluginId", &self.plugin_id)?;
        Ok(())
    }
}

/// Trusted official-catalog or acknowledged ClawHub install request.
/// 对齐 TS:
///   `Type.Union([
///      Type.Object({ source: Type.Literal("clawhub"), ... }),
///      Type.Object({ source: Type.Literal("official"), ... }),
///   ])`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum PluginsInstallParamsSchema {
    Clawhub(PluginsInstallClawhubParamsSchema),
    Official(PluginsInstallOfficialParamsSchema),
}

impl PluginsInstallParamsSchema {
    pub fn validate(&self) -> Result<(), String> {
        match self {
            Self::Clawhub(c) => c.validate(),
            Self::Official(o) => o.validate(),
        }
    }
}

/// Successful plugin installation result.
/// 对齐 TS:
///   `Type.Object({
///      ok: Type.Literal(true),
///      plugin: PluginCatalogEntrySchema,
///      restartRequired: Type.Literal(true),
///      warnings: Type.Optional(Type.Array(Type.String())),
///   }, { additionalProperties: false })`.
///
/// `ok` and `restartRequired` are constrained to literal `true`; validate checks them.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginsInstallResultSchema {
    pub ok: bool,
    pub plugin: PluginCatalogEntrySchema,
    pub restart_required: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub warnings: Option<Vec<String>>,
}

impl PluginsInstallResultSchema {
    pub fn validate(&self) -> Result<(), String> {
        if !self.ok {
            return Err("ok: expected literal true".to_string());
        }
        if !self.restart_required {
            return Err("restartRequired: expected literal true".to_string());
        }
        self.plugin.validate().map_err(|e| format!("plugin: {}", e))?;
        Ok(())
    }
}

// ============================================================================
// PluginsUninstall
// ============================================================================

/// Request payload for removing one installed plugin and its managed files.
/// 对齐 TS: `Type.Object({ pluginId: NonEmptyString }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginsUninstallParamsSchema {
    pub plugin_id: String,
}

impl PluginsUninstallParamsSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_non_empty_string("pluginId", &self.plugin_id)?;
        Ok(())
    }
}

/// Successful plugin removal result listing the cleanup actions that ran.
/// 对齐 TS:
///   `Type.Object({
///      ok: Type.Literal(true),
///      pluginId: NonEmptyString,
///      restartRequired: Type.Literal(true),
///      removed: Type.Array(Type.String()),
///      warnings: Type.Optional(Type.Array(Type.String())),
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginsUninstallResultSchema {
    pub ok: bool,
    pub plugin_id: String,
    pub restart_required: bool,
    pub removed: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub warnings: Option<Vec<String>>,
}

impl PluginsUninstallResultSchema {
    pub fn validate(&self) -> Result<(), String> {
        if !self.ok {
            return Err("ok: expected literal true".to_string());
        }
        if !self.restart_required {
            return Err("restartRequired: expected literal true".to_string());
        }
        validate_non_empty_string("pluginId", &self.plugin_id)?;
        Ok(())
    }
}

// ============================================================================
// PluginsSetEnabled
// ============================================================================

/// Request payload for changing one installed plugin's policy state.
/// 对齐 TS:
///   `Type.Object({
///      pluginId: NonEmptyString,
///      enabled: Type.Boolean(),
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginsSetEnabledParamsSchema {
    pub plugin_id: String,
    pub enabled: bool,
}

impl PluginsSetEnabledParamsSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_non_empty_string("pluginId", &self.plugin_id)?;
        Ok(())
    }
}

/// Successful plugin enablement policy update.
/// 对齐 TS:
///   `Type.Object({
///      ok: Type.Literal(true),
///      plugin: PluginCatalogEntrySchema,
///      restartRequired: Type.Boolean(),
///      warnings: Type.Optional(Type.Array(Type.String())),
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginsSetEnabledResultSchema {
    pub ok: bool,
    pub plugin: PluginCatalogEntrySchema,
    pub restart_required: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub warnings: Option<Vec<String>>,
}

impl PluginsSetEnabledResultSchema {
    pub fn validate(&self) -> Result<(), String> {
        if !self.ok {
            return Err("ok: expected literal true".to_string());
        }
        self.plugin.validate().map_err(|e| format!("plugin: {}", e))?;
        Ok(())
    }
}