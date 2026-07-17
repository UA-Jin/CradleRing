// Gateway Protocol schema: push.
// 翻译自 packages/gateway-protocol/src/schema/push.ts
//
// Push-notification protocol schemas.
//
// APNS test schemas exercise native push routing; Web Push schemas describe the
// browser subscription lifecycle exposed by the gateway.
//
// TS 用 TypeBox 定义 schema（运行时验证 + 类型）。
// Rust 用 serde struct + 验证函数实现等价的序列化/反序列化语义。

use serde::{Deserialize, Serialize};

use super::primitives::NonEmptyString;

// ---------- Module-private bounds ----------

/// Length bounds for the Web Push subscription key fields (`p256dh` / `auth`).
/// 对齐 TS: `Type.String({ minLength: 1, maxLength: 512 })`.
pub const WEB_PUSH_KEY_MAX_LENGTH: usize = 512;
pub const WEB_PUSH_KEY_MIN_LENGTH: usize = 1;

/// Length bounds for the Web Push subscription endpoint URL.
/// 对齐 TS: `Type.String({ minLength: 1, maxLength: 2048, pattern: "^https://" })`.
pub const WEB_PUSH_ENDPOINT_MAX_LENGTH: usize = 2048;
pub const WEB_PUSH_ENDPOINT_MIN_LENGTH: usize = 1;
/// 对齐 TS: `pattern: "^https://"`.
const WEB_PUSH_ENDPOINT_PATTERN: &str = r"^https://";

// ---------- 基础验证原语 ----------

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

/// Bounded string primitive used for `WebPushKeys.p256dh` and `.auth`.
/// 对齐 TS: `Type.String({ minLength: 1, maxLength: 512 })`.
fn validate_web_push_key(field: &str, value: &str) -> Result<(), String> {
    let len = value.chars().count();
    if len < WEB_PUSH_KEY_MIN_LENGTH || len > WEB_PUSH_KEY_MAX_LENGTH {
        return Err(format!(
            "{}: expected length [{}, {}], got {}",
            field, WEB_PUSH_KEY_MIN_LENGTH, WEB_PUSH_KEY_MAX_LENGTH, len
        ));
    }
    Ok(())
}

fn regex(pattern: &str) -> regex::Regex {
    regex::Regex::new(pattern).expect("invalid regex pattern compiled into push")
}

/// Bounded string primitive used for `WebPushSubscribeParams.endpoint` and
/// `WebPushUnsubscribeParams.endpoint`.
/// 对齐 TS: `Type.String({ minLength: 1, maxLength: 2048, pattern: "^https://" })`.
fn validate_web_push_endpoint(field: &str, value: &str) -> Result<(), String> {
    let len = value.chars().count();
    if len < WEB_PUSH_ENDPOINT_MIN_LENGTH || len > WEB_PUSH_ENDPOINT_MAX_LENGTH {
        return Err(format!(
            "{}: expected length [{}, {}], got {}",
            field, WEB_PUSH_ENDPOINT_MIN_LENGTH, WEB_PUSH_ENDPOINT_MAX_LENGTH, len
        ));
    }
    if !regex(WEB_PUSH_ENDPOINT_PATTERN).is_match(value) {
        return Err(format!(
            "{}: expected string matching {:?}, got {:?}",
            field, WEB_PUSH_ENDPOINT_PATTERN, value
        ));
    }
    Ok(())
}

// ---------- ApnsEnvironmentSchema ----------

/// Closed enumeration of APNS environments accepted by push-test requests.
/// 对齐 TS: `Type.String({ enum: ["sandbox", "production"] })`.
pub mod apns_environments {
    pub const SANDBOX: &str = "sandbox";
    pub const PRODUCTION: &str = "production";

    pub fn all() -> &'static [&'static str] {
        &[SANDBOX, PRODUCTION]
    }

    pub fn from_str(s: &str) -> Option<&'static str> {
        match s {
            "sandbox" => Some(SANDBOX),
            "production" => Some(PRODUCTION),
            _ => None,
        }
    }
}

pub fn is_valid_apns_environment(env: &str) -> bool {
    apns_environments::from_str(env).is_some()
}

/// Closed enumeration of APNS environments accepted by push-test requests.
/// 对齐 TS: `Type.String({ enum: ["sandbox", "production"] })`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ApnsEnvironmentSchema {
    #[serde(rename = "sandbox")]
    Sandbox,
    #[serde(rename = "production")]
    Production,
}

impl ApnsEnvironmentSchema {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Sandbox => "sandbox",
            Self::Production => "production",
        }
    }
}

// ---------- PushTestParamsSchema ----------

/// Request payload for sending a test APNS notification to one node.
/// 对齐 TS:
///   `Type.Object({
///      nodeId:      NonEmptyString,
///      title:       Type.Optional(Type.String()),
///      body:        Type.Optional(Type.String()),
///      environment: Type.Optional(ApnsEnvironmentSchema),
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PushTestParams {
    pub node_id: NonEmptyString,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub body: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub environment: Option<ApnsEnvironmentSchema>,
}

impl PushTestParams {
    pub fn validate(&self) -> Result<(), String> {
        validate_non_empty_string("nodeId", &self.node_id)?;
        Ok(())
    }
}

// ---------- PushTestResultSchema ----------

/// Result payload from an APNS push test, including provider status and transport.
/// 对齐 TS:
///   `Type.Object({
///      ok:          Type.Boolean(),
///      status:      Type.Integer(),
///      apnsId:      Type.Optional(Type.String()),
///      reason:      Type.Optional(Type.String()),
///      tokenSuffix: Type.String(),
///      topic:       Type.String(),
///      environment: ApnsEnvironmentSchema,
///      transport:   Type.String({ enum: ["direct", "relay"] }),
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PushTestResult {
    pub ok: bool,
    pub status: i64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub apns_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    pub token_suffix: String,
    pub topic: String,
    pub environment: ApnsEnvironmentSchema,
    /// 对齐 TS: `Type.String({ enum: ["direct", "relay"] })`.
    pub transport: PushTransport,
}

// ---------- PushTransportSchema ----------

/// Closed enumeration of push transport modes.
/// 对齐 TS: `Type.String({ enum: ["direct", "relay"] })`.
pub mod push_transports {
    pub const DIRECT: &str = "direct";
    pub const RELAY: &str = "relay";

    pub fn all() -> &'static [&'static str] {
        &[DIRECT, RELAY]
    }

    pub fn from_str(s: &str) -> Option<&'static str> {
        match s {
            "direct" => Some(DIRECT),
            "relay" => Some(RELAY),
            _ => None,
        }
    }
}

pub fn is_valid_push_transport(t: &str) -> bool {
    push_transports::from_str(t).is_some()
}

/// Closed enumeration of push transport modes.
/// 对齐 TS: `Type.String({ enum: ["direct", "relay"] })`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PushTransport {
    #[serde(rename = "direct")]
    Direct,
    #[serde(rename = "relay")]
    Relay,
}

impl PushTransport {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Direct => "direct",
            Self::Relay => "relay",
        }
    }
}

// ---------- WebPushKeysSchema ----------

/// `p256dh` / `auth` keys bundled with a browser PushSubscription.
/// 对齐 TS:
///   `Type.Object({
///      p256dh: Type.String({ minLength: 1, maxLength: 512 }),
///      auth:   Type.String({ minLength: 1, maxLength: 512 }),
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WebPushKeysSchema {
    pub p256dh: String,
    pub auth: String,
}

impl WebPushKeysSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_web_push_key("p256dh", &self.p256dh)?;
        validate_web_push_key("auth", &self.auth)?;
        Ok(())
    }
}

// ---------- WebPushVapidPublicKeyParamsSchema ----------

/// Empty request payload for fetching the Web Push VAPID public key.
/// 对齐 TS: `Type.Object({}, { additionalProperties: false })`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct WebPushVapidPublicKeyParams {}

impl WebPushVapidPublicKeyParams {
    pub fn validate(&self) -> Result<(), String> {
        // No required/constrained fields; the empty schema always validates.
        Ok(())
    }
}

// ---------- WebPushSubscribeParamsSchema ----------

/// Browser Web Push subscription payload registered with the gateway.
/// 对齐 TS:
///   `Type.Object({
///      endpoint: Type.String({ minLength: 1, maxLength: 2048, pattern: "^https://" }),
///      keys:     WebPushKeysSchema,
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WebPushSubscribeParams {
    pub endpoint: String,
    pub keys: WebPushKeysSchema,
}

impl WebPushSubscribeParams {
    pub fn validate(&self) -> Result<(), String> {
        validate_web_push_endpoint("endpoint", &self.endpoint)?;
        self.keys.validate().map_err(|e| format!("keys: {}", e))?;
        Ok(())
    }
}

// ---------- WebPushUnsubscribeParamsSchema ----------

/// Browser Web Push endpoint removal payload.
/// 对齐 TS:
///   `Type.Object({
///      endpoint: Type.String({ minLength: 1, maxLength: 2048, pattern: "^https://" }),
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WebPushUnsubscribeParams {
    pub endpoint: String,
}

impl WebPushUnsubscribeParams {
    pub fn validate(&self) -> Result<(), String> {
        validate_web_push_endpoint("endpoint", &self.endpoint)?;
        Ok(())
    }
}

// ---------- WebPushTestParamsSchema ----------

/// Request payload for sending a test Web Push notification to current subscriptions.
/// 对齐 TS:
///   `Type.Object({
///      title: Type.Optional(Type.String()),
///      body:  Type.Optional(Type.String()),
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WebPushTestParams {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub body: Option<String>,
}

// Wire types derive directly from local schema consts so public d.ts graphs never
// pull in the ProtocolSchemas registry.
// 对应 TS:
//   export type PushTestParams                  = Static<typeof PushTestParamsSchema>;
//   export type PushTestResult                  = Static<typeof PushTestResultSchema>;
//   export type WebPushVapidPublicKeyParams     = Record<string, never>;
//   export type WebPushSubscribeParams          = {
//     endpoint: string;
//     keys: { p256dh: string; auth: string };
//   };
//   export type WebPushUnsubscribeParams        = { endpoint: string };
//   export type WebPushTestParams               = { title?: string; body?: string };
pub type PushTestParamsType = PushTestParams;
pub type PushTestResultType = PushTestResult;
pub type WebPushVapidPublicKeyParamsType = WebPushVapidPublicKeyParams;
pub type WebPushSubscribeParamsType = WebPushSubscribeParams;
pub type WebPushUnsubscribeParamsType = WebPushUnsubscribeParams;
pub type WebPushTestParamsType = WebPushTestParams;