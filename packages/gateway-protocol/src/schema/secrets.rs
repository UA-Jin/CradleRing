// Gateway Protocol schema: secrets.
// 翻译自 packages/gateway-protocol/src/schema/secrets.ts
//
// TS 用 TypeBox 定义 schema（运行时验证 + 类型）。
// Rust 用 serde struct + 验证函数实现等价的序列化/反序列化语义。
//
// Secret-provider protocol schemas.
// These payloads request secret materialization from the gateway while keeping
// caller scope, allowed paths, and provider overrides explicit.

use serde::{Deserialize, Serialize};
use serde_json::Value;

// ---------- 基础验证原语 (对齐 TypeBox: NonEmptyString) ----------

fn is_non_empty_string(s: &str) -> bool {
    !s.trim().is_empty()
}

fn validate_non_empty_string(field: &str, value: &str) -> Result<(), String> {
    if is_non_empty_string(value) {
        Ok(())
    } else {
        Err(format!("{}: expected non-empty string, got {:?}", field, value))
    }
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

fn validate_optional_non_empty_string_list(
    field: &str,
    value: Option<&Vec<String>>,
) -> Result<(), String> {
    if let Some(arr) = value {
        validate_non_empty_string_list(field, arr.as_slice())?;
    }
    Ok(())
}

// ---------- SecretsReloadParamsSchema ----------
// 对齐 TS: `Type.Object({}, { additionalProperties: false })`

/// Empty request payload for reloading configured secret providers.
/// 对齐 TS: `SecretsReloadParamsSchema = Type.Object({}, { additionalProperties: false })`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SecretsReloadParams {}

impl SecretsReloadParams {
    pub fn validate(&self) -> Result<(), String> {
        // No required/constrained fields; the empty schema always validates.
        Ok(())
    }
}

// ---------- SecretsResolveParamsSchema ----------
// 对齐 TS:
//   Type.Object({
//     commandName: NonEmptyString,
//     targetIds: Type.Array(NonEmptyString),
//     allowedPaths?: Type.Array(NonEmptyString),
//     forcedActivePaths?: Type.Array(NonEmptyString),
//     optionalActivePaths?: Type.Array(NonEmptyString),
//     providerOverrides?: Type.Object({
//       webSearch?: NonEmptyString,
//       webFetch?: NonEmptyString,
//     }, { additionalProperties: false }),
//   }, { additionalProperties: false })

/// Optional provider overrides attached to a secret-resolution request.
/// 对齐 TS nested object inside `SecretsResolveParamsSchema`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SecretsResolveProviderOverrides {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub web_search: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub web_fetch: Option<String>,
}

impl SecretsResolveProviderOverrides {
    pub fn validate(&self) -> Result<(), String> {
        validate_non_empty_string("providerOverrides.webSearch", self.web_search.as_deref().unwrap_or(""))?;
        validate_non_empty_string("providerOverrides.webFetch", self.web_fetch.as_deref().unwrap_or(""))?;
        // Allow `None` as "field omitted"; reject empty *strings* only.
        self.web_search
            .as_deref()
            .into_iter()
            .try_for_each(|s| validate_non_empty_string("providerOverrides.webSearch", s))?;
        self.web_fetch
            .as_deref()
            .into_iter()
            .try_for_each(|s| validate_non_empty_string("providerOverrides.webFetch", s))?;
        Ok(())
    }
}

/// Request payload for resolving the secrets needed by one command invocation.
/// 对齐 TS: `SecretsResolveParamsSchema`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SecretsResolveParams {
    pub command_name: String,
    pub target_ids: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub allowed_paths: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub forced_active_paths: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub optional_active_paths: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provider_overrides: Option<SecretsResolveProviderOverrides>,
}

impl SecretsResolveParams {
    pub fn validate(&self) -> Result<(), String> {
        validate_non_empty_string("commandName", &self.command_name)?;
        validate_non_empty_string_list("targetIds", &self.target_ids)?;
        validate_optional_non_empty_string_list("allowedPaths", self.allowed_paths.as_ref())?;
        validate_optional_non_empty_string_list(
            "forcedActivePaths",
            self.forced_active_paths.as_ref(),
        )?;
        validate_optional_non_empty_string_list(
            "optionalActivePaths",
            self.optional_active_paths.as_ref(),
        )?;
        if let Some(po) = &self.provider_overrides {
            po.validate()?;
        }
        Ok(())
    }
}

// ---------- SecretsResolveAssignmentSchema ----------
// 对齐 TS:
//   Type.Object({
//     path?: NonEmptyString,
//     pathSegments: Type.Array(NonEmptyString),
//     value: Type.Unknown(),
//   }, { additionalProperties: false })

/// One resolved secret assignment path plus its provider-owned value.
/// 对齐 TS: `SecretsResolveAssignmentSchema`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecretsResolveAssignment {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    #[serde(rename = "pathSegments")]
    pub path_segments: Vec<String>,
    pub value: Value,
}

impl SecretsResolveAssignment {
    pub fn validate(&self) -> Result<(), String> {
        if let Some(p) = &self.path {
            validate_non_empty_string("path", p)?;
        }
        validate_non_empty_string_list("pathSegments", &self.path_segments)?;
        Ok(())
    }
}

// ---------- SecretsResolveResultSchema ----------
// 对齐 TS:
//   Type.Object({
//     ok?: Type.Boolean(),
//     assignments?: Type.Array(SecretsResolveAssignmentSchema),
//     diagnostics?: Type.Array(NonEmptyString),
//     inactiveRefPaths?: Type.Array(NonEmptyString),
//   }, { additionalProperties: false })

/// Secret resolution response with assignments and safe diagnostics.
/// 对齐 TS: `SecretsResolveResultSchema`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SecretsResolveResult {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ok: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub assignments: Option<Vec<SecretsResolveAssignment>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub diagnostics: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "inactiveRefPaths")]
    pub inactive_ref_paths: Option<Vec<String>>,
}

impl SecretsResolveResult {
    pub fn validate(&self) -> Result<(), String> {
        if let Some(assignments) = &self.assignments {
            for (i, a) in assignments.iter().enumerate() {
                a.validate().map_err(|e| format!("assignments[{}]: {}", i, e))?;
            }
        }
        validate_optional_non_empty_string_list("diagnostics", self.diagnostics.as_ref())?;
        validate_optional_non_empty_string_list(
            "inactiveRefPaths",
            self.inactive_ref_paths.as_ref(),
        )?;
        Ok(())
    }
}

// Wire type aliases (对标 TS `type X = Static<typeof YSchema>`)
pub type SecretsReloadParamsType = SecretsReloadParams;
pub type SecretsResolveParamsType = SecretsResolveParams;
pub type SecretsResolveAssignmentType = SecretsResolveAssignment;
pub type SecretsResolveResultType = SecretsResolveResult;
