// External code plugin package.json compatibility and validation contracts.
// 翻译自 packages/plugin-package-contract/src/index.ts

use std::collections::BTreeMap;

/// JSON object shape accepted by package contract helpers.
pub type JsonObject = BTreeMap<String, serde_json::Value>;

/// Compatibility metadata extracted from an external plugin package.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct ExternalPluginCompatibility {
    #[serde(rename = "pluginApiRange", skip_serializing_if = "Option::is_none")]
    pub plugin_api_range: Option<String>,
    #[serde(rename = "builtWithOpenClawVersion", skip_serializing_if = "Option::is_none")]
    pub built_with_openclaw_version: Option<String>,
    #[serde(rename = "pluginSdkVersion", skip_serializing_if = "Option::is_none")]
    pub plugin_sdk_version: Option<String>,
    #[serde(rename = "minGatewayVersion", skip_serializing_if = "Option::is_none")]
    pub min_gateway_version: Option<String>,
}

/// One validation issue for an external plugin package.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ExternalPluginValidationIssue {
    #[serde(rename = "fieldPath")]
    pub field_path: String,
    #[serde(rename = "message")]
    pub message: String,
}

/// Validation result plus any normalized compatibility metadata.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct ExternalCodePluginValidationResult {
    #[serde(rename = "compatibility", skip_serializing_if = "Option::is_none")]
    pub compatibility: Option<ExternalPluginCompatibility>,
    #[serde(rename = "issues")]
    pub issues: Vec<ExternalPluginValidationIssue>,
}

/// Required package.json field paths for external code plugin packages.
pub const EXTERNAL_CODE_PLUGIN_REQUIRED_FIELD_PATHS: &[&str] = &[
    "openclaw.compat.pluginApi",
    "openclaw.build.cradle-ringVersion",
];

/// Narrow unknown values to plain records.
fn is_record(value: &serde_json::Value) -> bool {
    value.is_object()
}

/// Normalize optional package metadata strings.
fn normalize_optional_string(value: Option<&serde_json::Value>) -> Option<String> {
    let raw = value?.as_str()?;
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

struct OpenClawBlock<'a> {
    root: Option<&'a serde_json::Value>,
    #[allow(dead_code)]
    openclaw: Option<&'a serde_json::Value>,
    compat: Option<&'a serde_json::Value>,
    build: Option<&'a serde_json::Value>,
    install: Option<&'a serde_json::Value>,
}

/// Read OpenClaw package.json blocks without trusting caller input shape.
fn read_openclaw_block<'a>(package_json: Option<&'a serde_json::Value>) -> OpenClawBlock<'a> {
    let root = package_json.filter(|v| is_record(v));
    let openclaw = root
        .and_then(|r| r.get("openclaw"))
        .filter(|v| is_record(v));
    let compat = openclaw
        .and_then(|o| o.get("compat"))
        .filter(|v| is_record(v));
    let build = openclaw
        .and_then(|o| o.get("build"))
        .filter(|v| is_record(v));
    let install = openclaw
        .and_then(|o| o.get("install"))
        .filter(|v| is_record(v));
    OpenClawBlock {
        root,
        openclaw,
        compat,
        build,
        install,
    }
}

/// Normalize compatibility metadata from an external plugin package.json.
pub fn normalize_external_plugin_compatibility(
    package_json: &serde_json::Value,
) -> Option<ExternalPluginCompatibility> {
    let block = read_openclaw_block(Some(package_json));
    let version = normalize_optional_string(block.root.and_then(|r| r.get("version")));
    let min_host_version =
        normalize_optional_string(block.install.and_then(|i| i.get("minHostVersion")));
    let mut compatibility = ExternalPluginCompatibility::default();

    let plugin_api = normalize_optional_string(block.compat.and_then(|c| c.get("pluginApi")));
    if let Some(api) = plugin_api {
        compatibility.plugin_api_range = Some(api);
    }

    let min_gateway_version = normalize_optional_string(
        block.compat.and_then(|c| c.get("minGatewayVersion")),
    )
    .or(min_host_version);
    if let Some(mgv) = min_gateway_version {
        compatibility.min_gateway_version = Some(mgv);
    }

    let built_with_openclaw_version =
        normalize_optional_string(block.build.and_then(|b| b.get("openclawVersion")))
            .or(version);
    if let Some(bv) = built_with_openclaw_version {
        compatibility.built_with_openclaw_version = Some(bv);
    }

    let plugin_sdk_version =
        normalize_optional_string(block.build.and_then(|b| b.get("pluginSdkVersion")));
    if let Some(psv) = plugin_sdk_version {
        compatibility.plugin_sdk_version = Some(psv);
    }

    if compatibility.plugin_api_range.is_none()
        && compatibility.built_with_openclaw_version.is_none()
        && compatibility.plugin_sdk_version.is_none()
        && compatibility.min_gateway_version.is_none()
    {
        None
    } else {
        Some(compatibility)
    }
}

/// List missing required field paths for an external code plugin package.json.
pub fn list_missing_external_code_plugin_field_paths(
    package_json: &serde_json::Value,
) -> Vec<String> {
    let block = read_openclaw_block(Some(package_json));
    let mut missing: Vec<String> = Vec::new();
    if normalize_optional_string(block.compat.and_then(|c| c.get("pluginApi"))).is_none() {
        missing.push("openclaw.compat.pluginApi".to_string());
    }
    if normalize_optional_string(block.build.and_then(|b| b.get("openclawVersion"))).is_none() {
        missing.push("openclaw.build.cradle-ringVersion".to_string());
    }
    missing
}

/// Validate an external code plugin package.json against required compatibility fields.
pub fn validate_external_code_plugin_package_json(
    package_json: &serde_json::Value,
) -> ExternalCodePluginValidationResult {
    let issues = list_missing_external_code_plugin_field_paths(package_json)
        .into_iter()
        .map(|field_path| ExternalPluginValidationIssue {
            field_path: field_path.clone(),
            message: format!("{field_path} is required for external code plugin packages."),
        })
        .collect();
    ExternalCodePluginValidationResult {
        compatibility: normalize_external_plugin_compatibility(package_json),
        issues,
    }
}