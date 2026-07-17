// Gateway Protocol schema: devices.
// 翻译自 packages/gateway-protocol/src/schema/devices.ts
//
// TS 用 TypeBox 定义 schema（运行时验证 + 类型）。
// Rust 用 serde struct + 验证函数实现等价的序列化/反序列化语义。

use serde::{Deserialize, Serialize};

// ---------- 基础验证原语 (对齐 TypeBox: NonEmptyString / String{min,max} / Integer{min}) ----------

/// 对齐 TS: `NonEmptyString = Type.String({ minLength: 1 })`
fn is_non_empty_string(s: &str) -> bool {
    !s.is_empty()
}

fn validate_non_empty_string(field: &str, value: &str) -> Result<(), String> {
    if is_non_empty_string(value) {
        Ok(())
    } else {
        Err(format!("{}: expected non-empty string, got {:?}", field, value))
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

fn validate_optional_non_empty_string_list(
    field: &str,
    value: Option<&Vec<String>>,
) -> Result<(), String> {
    if let Some(arr) = value {
        validate_non_empty_string_list(field, arr.as_slice())?;
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

// ---------- Module-private bounds ----------

/// Maximum length for an operator-assigned device label.
/// 对齐 TS: `DevicePairLabelString = Type.String({ minLength: 1, maxLength: 64 })`.
const DEVICE_PAIR_LABEL_MAX_LENGTH: usize = 64;
const DEVICE_PAIR_LABEL_MIN_LENGTH: usize = 1;

fn validate_device_pair_label(field: &str, value: &str) -> Result<(), String> {
    let len = value.chars().count();
    if len < DEVICE_PAIR_LABEL_MIN_LENGTH || len > DEVICE_PAIR_LABEL_MAX_LENGTH {
        return Err(format!(
            "{}: expected length [{}, {}], got {}",
            field, DEVICE_PAIR_LABEL_MIN_LENGTH, DEVICE_PAIR_LABEL_MAX_LENGTH, len
        ));
    }
    Ok(())
}

/// Maximum length for an inline data URL holding a base64 PNG QR code.
/// 对齐 TS: `SetupCodeQrDataUrlSchema = Type.String({ maxLength: 16_384, pattern: "^data:image/png;base64," })`.
const SETUP_CODE_QR_DATA_URL_MAX_LENGTH: usize = 16_384;
const SETUP_CODE_QR_DATA_URL_PATTERN: &str = r"^data:image/png;base64,";

fn regex(pattern: &str) -> regex::Regex {
    regex::Regex::new(pattern).expect("invalid regex pattern compiled into devices")
}

fn validate_setup_code_qr_data_url(field: &str, value: &str) -> Result<(), String> {
    if value.len() > SETUP_CODE_QR_DATA_URL_MAX_LENGTH {
        return Err(format!(
            "{}: expected length <= {}, got {}",
            field,
            SETUP_CODE_QR_DATA_URL_MAX_LENGTH,
            value.len()
        ));
    }
    if !regex(SETUP_CODE_QR_DATA_URL_PATTERN).is_match(value) {
        return Err(format!(
            "{}: expected string matching {:?}, got {:?}",
            field, SETUP_CODE_QR_DATA_URL_PATTERN, value
        ));
    }
    Ok(())
}

/// Bounds for `gatewayUrls: Type.Array(NonEmptyString, { minItems: 2, maxItems: 8, uniqueItems: true })`.
const DEVICE_PAIR_GATEWAY_URLS_MIN_ITEMS: usize = 2;
const DEVICE_PAIR_GATEWAY_URLS_MAX_ITEMS: usize = 8;

// ---------- BootstrapProfile enum ----------

/// Bootstrap profile literal accepted by `DevicePairSetupCodeParamsSchema`.
/// 对齐 TS: `Type.Literal("node")`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DevicePairSetupCodeBootstrapProfile {
    Node,
}

impl DevicePairSetupCodeBootstrapProfile {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Node => "node",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "node" => Some(Self::Node),
            _ => None,
        }
    }

    pub fn all() -> &'static [Self] {
        &[Self::Node]
    }
}

pub fn is_valid_device_pair_setup_code_bootstrap_profile(profile: &str) -> bool {
    DevicePairSetupCodeBootstrapProfile::from_str(profile).is_some()
}

// ---------- SetupAuth enum ----------

/// Auth label returned in `DevicePairSetupCodeResultSchema`.
/// 对齐 TS: `Type.Union([Type.Literal("token"), Type.Literal("password")])`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DevicePairSetupCodeAuth {
    #[serde(rename = "token")]
    Token,
    #[serde(rename = "password")]
    Password,
}

impl DevicePairSetupCodeAuth {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Token => "token",
            Self::Password => "password",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "token" => Some(Self::Token),
            "password" => Some(Self::Password),
            _ => None,
        }
    }

    pub fn all() -> &'static [Self] {
        &[Self::Token, Self::Password]
    }
}

pub fn is_valid_device_pair_setup_code_auth(auth: &str) -> bool {
    DevicePairSetupCodeAuth::from_str(auth).is_some()
}

// ---------- DevicePairListParamsSchema ----------
// 对齐 TS: `Type.Object({}, { additionalProperties: false })`

/// Lists pending and approved device pairing records.
/// 对齐 TS: `DevicePairListParamsSchema = Type.Object({}, { additionalProperties: false })`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DevicePairListParams {}

impl DevicePairListParams {
    pub fn validate(&self) -> Result<(), String> {
        // No required/constrained fields; the empty schema always validates.
        Ok(())
    }
}

// ---------- DevicePairApproveParamsSchema ----------
// 对齐 TS: `Type.Object({ requestId: NonEmptyString }, { additionalProperties: false })`

/// Approves a pending pairing request by request id.
/// 对齐 TS: `DevicePairApproveParamsSchema`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DevicePairApproveParams {
    pub request_id: String,
}

impl DevicePairApproveParams {
    pub fn validate(&self) -> Result<(), String> {
        validate_non_empty_string("requestId", &self.request_id)?;
        Ok(())
    }
}

// ---------- DevicePairRejectParamsSchema ----------
// 对齐 TS: `Type.Object({ requestId: NonEmptyString }, { additionalProperties: false })`

/// Rejects a pending pairing request by request id.
/// 对齐 TS: `DevicePairRejectParamsSchema`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DevicePairRejectParams {
    pub request_id: String,
}

impl DevicePairRejectParams {
    pub fn validate(&self) -> Result<(), String> {
        validate_non_empty_string("requestId", &self.request_id)?;
        Ok(())
    }
}

// ---------- DevicePairRemoveParamsSchema ----------
// 对齐 TS: `Type.Object({ deviceId: NonEmptyString }, { additionalProperties: false })`

/// Removes an approved or remembered device by device id.
/// 对齐 TS: `DevicePairRemoveParamsSchema`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DevicePairRemoveParams {
    pub device_id: String,
}

impl DevicePairRemoveParams {
    pub fn validate(&self) -> Result<(), String> {
        validate_non_empty_string("deviceId", &self.device_id)?;
        Ok(())
    }
}

// ---------- DevicePairRenameParamsSchema ----------
// 对齐 TS: `Type.Object({
//   deviceId: NonEmptyString,
//   label: DevicePairLabelString,  // string { minLength: 1, maxLength: 64 }
// }, { additionalProperties: false })`

/// Renames a paired device while preserving its stable device id.
/// 对齐 TS: `DevicePairRenameParamsSchema`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DevicePairRenameParams {
    pub device_id: String,
    pub label: String,
}

impl DevicePairRenameParams {
    pub fn validate(&self) -> Result<(), String> {
        validate_non_empty_string("deviceId", &self.device_id)?;
        validate_device_pair_label("label", &self.label)?;
        Ok(())
    }
}

// ---------- DeviceTokenRotateParamsSchema ----------
// 对齐 TS: `Type.Object({
//   deviceId: NonEmptyString,
//   role: NonEmptyString,
//   scopes: Type.Optional(Type.Array(NonEmptyString)),
// }, { additionalProperties: false })`

/// Rotates or issues a device token for a specific role/scope grant.
/// 对齐 TS: `DeviceTokenRotateParamsSchema`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeviceTokenRotateParams {
    pub device_id: String,
    pub role: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scopes: Option<Vec<String>>,
}

impl DeviceTokenRotateParams {
    pub fn validate(&self) -> Result<(), String> {
        validate_non_empty_string("deviceId", &self.device_id)?;
        validate_non_empty_string("role", &self.role)?;
        validate_optional_non_empty_string_list("scopes", self.scopes.as_ref())?;
        Ok(())
    }
}

// ---------- DeviceTokenRevokeParamsSchema ----------
// 对齐 TS: `Type.Object({
//   deviceId: NonEmptyString,
//   role: NonEmptyString,
// }, { additionalProperties: false })`

/// Revokes one role-bound device token grant.
/// 对齐 TS: `DeviceTokenRevokeParamsSchema`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeviceTokenRevokeParams {
    pub device_id: String,
    pub role: String,
}

impl DeviceTokenRevokeParams {
    pub fn validate(&self) -> Result<(), String> {
        validate_non_empty_string("deviceId", &self.device_id)?;
        validate_non_empty_string("role", &self.role)?;
        Ok(())
    }
}

// ---------- DevicePairRequestedEventSchema ----------
// 对齐 TS: `Type.Object({
//   requestId: NonEmptyString,
//   deviceId: NonEmptyString,
//   publicKey: NonEmptyString,
//   displayName:     Type.Optional(NonEmptyString),
//   platform:        Type.Optional(NonEmptyString),
//   deviceFamily:    Type.Optional(NonEmptyString),
//   clientId:        Type.Optional(NonEmptyString),
//   clientMode:      Type.Optional(NonEmptyString),
//   role:            Type.Optional(NonEmptyString),
//   roles:           Type.Optional(Type.Array(NonEmptyString)),
//   scopes:          Type.Optional(Type.Array(NonEmptyString)),
//   remoteIp:        Type.Optional(NonEmptyString),
//   silent:          Type.Optional(Type.Boolean()),
//   isRepair:        Type.Optional(Type.Boolean()),
//   ts: Type.Integer({ minimum: 0 }),
// }, { additionalProperties: false })`

/// Event emitted when a client opens or refreshes a pairing request.
/// 对齐 TS: `DevicePairRequestedEventSchema`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DevicePairRequestedEvent {
    pub request_id: String,
    pub device_id: String,
    pub public_key: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub platform: Option<String>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "deviceFamily"
    )]
    pub device_family: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "clientId")]
    pub client_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "clientMode")]
    pub client_mode: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub roles: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scopes: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "remoteIp")]
    pub remote_ip: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub silent: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "isRepair")]
    pub is_repair: Option<bool>,
    pub ts: i64,
}

impl DevicePairRequestedEvent {
    pub fn validate(&self) -> Result<(), String> {
        validate_non_empty_string("requestId", &self.request_id)?;
        validate_non_empty_string("deviceId", &self.device_id)?;
        validate_non_empty_string("publicKey", &self.public_key)?;
        validate_optional_non_empty_string("displayName", self.display_name.as_deref())?;
        validate_optional_non_empty_string("platform", self.platform.as_deref())?;
        validate_optional_non_empty_string("deviceFamily", self.device_family.as_deref())?;
        validate_optional_non_empty_string("clientId", self.client_id.as_deref())?;
        validate_optional_non_empty_string("clientMode", self.client_mode.as_deref())?;
        validate_optional_non_empty_string("role", self.role.as_deref())?;
        validate_optional_non_empty_string_list("roles", self.roles.as_ref())?;
        validate_optional_non_empty_string_list("scopes", self.scopes.as_ref())?;
        validate_optional_non_empty_string("remoteIp", self.remote_ip.as_deref())?;
        validate_non_negative_integer("ts", self.ts)?;
        Ok(())
    }
}

// ---------- DevicePairResolvedEventSchema ----------
// 对齐 TS: `Type.Object({
//   requestId: NonEmptyString,
//   deviceId: NonEmptyString,
//   decision: NonEmptyString,
//   ts: Type.Integer({ minimum: 0 }),
// }, { additionalProperties: false })`

/// Event emitted after a pairing request is approved, rejected, or otherwise resolved.
/// 对齐 TS: `DevicePairResolvedEventSchema`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DevicePairResolvedEvent {
    pub request_id: String,
    pub device_id: String,
    pub decision: String,
    pub ts: i64,
}

impl DevicePairResolvedEvent {
    pub fn validate(&self) -> Result<(), String> {
        validate_non_empty_string("requestId", &self.request_id)?;
        validate_non_empty_string("deviceId", &self.device_id)?;
        validate_non_empty_string("decision", &self.decision)?;
        validate_non_negative_integer("ts", self.ts)?;
        Ok(())
    }
}

// ---------- DevicePairSetupCodeParamsSchema ----------
// 对齐 TS: `Type.Object({
//   publicUrl:         Type.Optional(NonEmptyString),
//   preferRemoteUrl:   Type.Optional(Type.Boolean()),
//   includeQr:         Type.Optional(Type.Boolean()),
//   bootstrapProfile:  Type.Optional(Type.Literal("node")),
// }, { additionalProperties: false })`

/// Generates a device-pairing setup code (and optional QR) so a mobile/companion
/// client can scan it and connect to this gateway.
/// 对齐 TS: `DevicePairSetupCodeParamsSchema`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DevicePairSetupCodeParams {
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "publicUrl")]
    pub public_url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "preferRemoteUrl")]
    pub prefer_remote_url: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "includeQr")]
    pub include_qr: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "bootstrapProfile")]
    pub bootstrap_profile: Option<DevicePairSetupCodeBootstrapProfile>,
}

impl DevicePairSetupCodeParams {
    pub fn validate(&self) -> Result<(), String> {
        validate_optional_non_empty_string("publicUrl", self.public_url.as_deref())?;
        Ok(())
    }
}

// ---------- DevicePairSetupCodeResultSchema ----------
// 对齐 TS: `Type.Object({
//   setupCode:    NonEmptyString,
//   qrDataUrl:    Type.Optional(SetupCodeQrDataUrlSchema),
//   gatewayUrl:   NonEmptyString,
//   gatewayUrls:  Type.Optional(
//     Type.Array(NonEmptyString, { minItems: 2, maxItems: 8, uniqueItems: true }),
//   ),
//   auth:    Type.Union([Type.Literal("token"), Type.Literal("password")]),
//   urlSource: NonEmptyString,
// }, { additionalProperties: false })`

/// Setup code plus non-secret connection metadata. `auth` is a label only.
/// 对齐 TS: `DevicePairSetupCodeResultSchema`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DevicePairSetupCodeResult {
    #[serde(rename = "setupCode")]
    pub setup_code: String,
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "qrDataUrl")]
    pub qr_data_url: Option<String>,
    #[serde(rename = "gatewayUrl")]
    pub gateway_url: String,
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "gatewayUrls")]
    pub gateway_urls: Option<Vec<String>>,
    pub auth: DevicePairSetupCodeAuth,
    #[serde(rename = "urlSource")]
    pub url_source: String,
}

impl DevicePairSetupCodeResult {
    pub fn validate(&self) -> Result<(), String> {
        validate_non_empty_string("setupCode", &self.setup_code)?;
        if let Some(qr) = &self.qr_data_url {
            validate_setup_code_qr_data_url("qrDataUrl", qr)?;
        }
        validate_non_empty_string("gatewayUrl", &self.gateway_url)?;
        if let Some(urls) = &self.gateway_urls {
            if urls.len() < DEVICE_PAIR_GATEWAY_URLS_MIN_ITEMS
                || urls.len() > DEVICE_PAIR_GATEWAY_URLS_MAX_ITEMS
            {
                return Err(format!(
                    "gatewayUrls: expected length [{}, {}], got {}",
                    DEVICE_PAIR_GATEWAY_URLS_MIN_ITEMS,
                    DEVICE_PAIR_GATEWAY_URLS_MAX_ITEMS,
                    urls.len()
                ));
            }
            for (i, u) in urls.iter().enumerate() {
                validate_non_empty_string(&format!("gatewayUrls[{}]", i), u)?;
            }
            let mut seen: Vec<&String> = Vec::new();
            for u in urls.iter() {
                if seen.iter().any(|x| *x == u) {
                    return Err(format!("gatewayUrls: expected unique items, duplicate {:?}", u));
                }
                seen.push(u);
            }
        }
        validate_non_empty_string("urlSource", &self.url_source)?;
        Ok(())
    }
}

// Wire types derive directly from local schema consts so public d.ts graphs never
// pull in the ProtocolSchemas registry.
// 对应 TS:
//   export type DevicePairListParams        = Static<typeof DevicePairListParamsSchema>;
//   export type DevicePairApproveParams     = Static<typeof DevicePairApproveParamsSchema>;
//   export type DevicePairRejectParams      = Static<typeof DevicePairRejectParamsSchema>;
//   export type DevicePairRemoveParams      = Static<typeof DevicePairRemoveParamsSchema>;
//   export type DevicePairSetupCodeParams   = Static<typeof DevicePairSetupCodeParamsSchema>;
//   export type DevicePairSetupCodeResult   = Static<typeof DevicePairSetupCodeResultSchema>;
//   export type DevicePairRenameParams      = Static<typeof DevicePairRenameParamsSchema>;
//   export type DeviceTokenRotateParams     = Static<typeof DeviceTokenRotateParamsSchema>;
//   export type DeviceTokenRevokeParams     = Static<typeof DeviceTokenRevokeParamsSchema>;
//   export type DevicePairRequestedEvent    = Static<typeof DevicePairRequestedEventSchema>;
//   export type DevicePairResolvedEvent     = Static<typeof DevicePairResolvedEventSchema>;
pub type DevicePairListParamsType = DevicePairListParams;
pub type DevicePairApproveParamsType = DevicePairApproveParams;
pub type DevicePairRejectParamsType = DevicePairRejectParams;
pub type DevicePairRemoveParamsType = DevicePairRemoveParams;
pub type DevicePairRenameParamsType = DevicePairRenameParams;
pub type DeviceTokenRotateParamsType = DeviceTokenRotateParams;
pub type DeviceTokenRevokeParamsType = DeviceTokenRevokeParams;
pub type DevicePairRequestedEventType = DevicePairRequestedEvent;
pub type DevicePairResolvedEventType = DevicePairResolvedEvent;
pub type DevicePairSetupCodeParamsType = DevicePairSetupCodeParams;
pub type DevicePairSetupCodeResultType = DevicePairSetupCodeResult;
