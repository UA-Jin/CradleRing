// Gateway Protocol schema: worker-admission.
// 翻译自 packages/gateway-protocol/src/schema/worker-admission.ts
//
// Worker-side admission / heartbeat / transcript commit / live event protocol.
// This module covers the dedicated first-frame handshake that admits a worker
// process into a gateway environment, plus the steady-state RPC surface the
// gateway exposes to that worker. The session is intentionally minimal — the
// general gateway hello/snapshot is never sent to a worker.
//
// TS 用 TypeBox 定义 schema（运行时验证 + 类型）。
// Rust 用 serde struct + 验证函数实现等价的序列化/反序列化语义。

use serde::{Deserialize, Serialize};

use crate::client_info::{gateway_client_ids, gateway_client_modes};

// ============================================================================
// Protocol constants (mirroring the TS module-level consts)
// ============================================================================

/// Build-bound RPC set version. Bump only for an incompatible base set.
pub const WORKER_RPC_SET_VERSION: i64 = 1;

/// Required heartbeat cadence.
pub const WORKER_HEARTBEAT_INTERVAL_MS: i64 = 15_000;

/// Canonical worker RPC method names (kept in declaration order).
pub const WORKER_PROTOCOL_METHODS: [&str; 3] = [
    "worker.heartbeat",
    "worker.transcript.commit",
    "worker.live-event",
];

/// Capability strings a worker can advertise.
pub const WORKER_TRANSCRIPT_COMMIT_PROTOCOL_FEATURE: &str = "worker-transcript-commit-v1";
pub const WORKER_LIVE_EVENT_PROTOCOL_FEATURE: &str = "worker-live-event-v1";
pub const WORKER_PROTOCOL_FEATURES: [&str; 3] = [
    "worker-heartbeat-v1",
    WORKER_TRANSCRIPT_COMMIT_PROTOCOL_FEATURE,
    WORKER_LIVE_EVENT_PROTOCOL_FEATURE,
];

/// Maximum stable identifier / frame id / method / feature / payload sizes.
pub const WORKER_PROTOCOL_MAX_IDENTIFIER_LENGTH: usize = 256;
pub const WORKER_PROTOCOL_MAX_FRAME_ID_LENGTH: usize = 128;
pub const WORKER_PROTOCOL_MAX_METHOD_LENGTH: usize = 64;
pub const WORKER_PROTOCOL_MAX_PAYLOAD_BYTES: usize = 64 * 1024;
pub const WORKER_PROTOCOL_MAX_FEATURES: usize = 64;
pub const WORKER_PROTOCOL_MAX_FEATURE_LENGTH: usize = 128;

/// Transcript batch size limits.
pub const WORKER_TRANSCRIPT_MAX_BATCH_MESSAGES: usize = 64;
pub const WORKER_TRANSCRIPT_MAX_CONTENT_PARTS: usize = 128;
pub const WORKER_TRANSCRIPT_MAX_JSON_DEPTH: usize = 32;

// ============================================================================
// 基础验证原语
// ============================================================================

#[allow(dead_code)]
fn is_non_empty_string(s: &str) -> bool {
    !s.is_empty()
}

#[allow(dead_code)]
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

fn validate_string_length_range(
    field: &str,
    value: &str,
    min: usize,
    max: usize,
) -> Result<(), String> {
    let len = value.chars().count();
    if len < min || len > max {
        return Err(format!(
            "{}: expected length [{}, {}], got {}",
            field, min, max, len
        ));
    }
    Ok(())
}

fn validate_non_negative_integer(field: &str, n: i64) -> Result<(), String> {
    if n >= 0 {
        Ok(())
    } else {
        Err(format!("{}: expected integer >= 0, got {}", field, n))
    }
}

fn validate_optional_non_negative_integer(field: &str, n: Option<i64>) -> Result<(), String> {
    if let Some(v) = n {
        validate_non_negative_integer(field, v)?;
    }
    Ok(())
}

fn validate_positive_integer(field: &str, n: i64) -> Result<(), String> {
    if n >= 1 {
        Ok(())
    } else {
        Err(format!("{}: expected integer >= 1, got {}", field, n))
    }
}

#[allow(dead_code)]
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

fn validate_non_negative_number(field: &str, n: f64) -> Result<(), String> {
    if n >= 0.0 {
        Ok(())
    } else {
        Err(format!("{}: expected number >= 0, got {}", field, n))
    }
}

#[allow(dead_code)]
fn validate_optional_non_negative_number(field: &str, n: Option<f64>) -> Result<(), String> {
    if let Some(v) = n {
        validate_non_negative_number(field, v)?;
    }
    Ok(())
}

fn regex(pattern: &str) -> regex::Regex {
    regex::Regex::new(pattern).expect("invalid regex pattern compiled into worker-admission")
}

#[allow(dead_code)]
fn validate_pattern(field: &str, value: &str, pattern: &str) -> Result<(), String> {
    if regex(pattern).is_match(value) {
        Ok(())
    } else {
        Err(format!(
            "{}: expected string matching {}, got {:?}",
            field, pattern, value
        ))
    }
}

// ============================================================================
// Identifier / feature / hash primitives
// ============================================================================

/// Worker identifier grammar: `^\S(?:.*\S)?$` with bounded length.
const WORKER_IDENTIFIER_PATTERN: &str = r"^\S(?:.*\S)?$";

/// Bundle hash: 64 lowercase hex chars.
const WORKER_BUNDLE_HASH_PATTERN: &str = r"^[a-f0-9]{64}$";

/// Identifier validation (length + non-empty + no-leading/trailing whitespace).
pub fn is_valid_worker_identifier(s: &str) -> bool {
    if s.is_empty() || s.chars().count() > WORKER_PROTOCOL_MAX_IDENTIFIER_LENGTH {
        return false;
    }
    regex(WORKER_IDENTIFIER_PATTERN).is_match(s)
}

pub fn validate_worker_identifier(field: &str, s: &str) -> Result<(), String> {
    if !is_valid_worker_identifier(s) {
        return Err(format!(
            "{}: expected non-empty identifier up to {} chars matching {}, got {:?}",
            field, WORKER_PROTOCOL_MAX_IDENTIFIER_LENGTH, WORKER_IDENTIFIER_PATTERN, s
        ));
    }
    Ok(())
}

/// Worker credential: length 16..=256.
pub fn is_valid_worker_credential(s: &str) -> bool {
    let len = s.chars().count();
    (16..=256).contains(&len)
}

pub fn validate_worker_credential(field: &str, s: &str) -> Result<(), String> {
    if !is_valid_worker_credential(s) {
        return Err(format!(
            "{}: expected length 16..=256, got {}",
            field,
            s.chars().count()
        ));
    }
    Ok(())
}

/// Worker frame id: 1..=128 chars.
pub fn is_valid_worker_frame_id(s: &str) -> bool {
    let len = s.chars().count();
    (1..=WORKER_PROTOCOL_MAX_FRAME_ID_LENGTH).contains(&len)
}

pub fn validate_worker_frame_id(field: &str, s: &str) -> Result<(), String> {
    if !is_valid_worker_frame_id(s) {
        return Err(format!(
            "{}: expected length 1..={}, got {}",
            field,
            WORKER_PROTOCOL_MAX_FRAME_ID_LENGTH,
            s.chars().count()
        ));
    }
    Ok(())
}

/// Worker protocol feature token: 1..=WORKER_PROTOCOL_MAX_FEATURE_LENGTH.
pub fn is_valid_worker_protocol_feature(s: &str) -> bool {
    let len = s.chars().count();
    (1..=WORKER_PROTOCOL_MAX_FEATURE_LENGTH).contains(&len)
}

pub fn validate_worker_protocol_feature(field: &str, s: &str) -> Result<(), String> {
    if !is_valid_worker_protocol_feature(s) {
        return Err(format!(
            "{}: expected length 1..={}, got {}",
            field,
            WORKER_PROTOCOL_MAX_FEATURE_LENGTH,
            s.chars().count()
        ));
    }
    Ok(())
}

/// Worker bundle hash: 64 lowercase hex chars.
pub fn is_valid_worker_bundle_hash(s: &str) -> bool {
    s.chars().count() == 64 && regex(WORKER_BUNDLE_HASH_PATTERN).is_match(s)
}

pub fn validate_worker_bundle_hash(field: &str, s: &str) -> Result<(), String> {
    if !is_valid_worker_bundle_hash(s) {
        return Err(format!(
            "{}: expected 64-char lowercase hex bundle hash, got {:?}",
            field, s
        ));
    }
    Ok(())
}

// ============================================================================
// Frame type literal markers
// ============================================================================

/// Frame type discriminator for worker-side request/response frames.
/// 对齐 TS: `Type.Literal("req")`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum WorkerFrameTypeReq {
    #[serde(rename = "req")]
    Req,
}

impl WorkerFrameTypeReq {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Req => "req",
        }
    }
}

/// Frame type discriminator for worker-side response frames.
/// 对齐 TS: `Type.Literal("res")`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum WorkerFrameTypeRes {
    #[serde(rename = "res")]
    Res,
}

impl WorkerFrameTypeRes {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Res => "res",
        }
    }
}

// ============================================================================
// Worker admission handshake
// ============================================================================

/// Build identity presented by a worker before the gateway admits it.
/// 对齐 TS:
///   `Type.Object({
///      bundleHash: WorkerBundleHashSchema,
///      openclawVersion: Type.String({ minLength: 1, maxLength: 128 }),
///      protocolFeatures: Type.Array(WorkerProtocolFeatureSchema, {
///        maxItems: WORKER_PROTOCOL_MAX_FEATURES, uniqueItems: true,
///      }),
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkerAdmissionHandshakeSchema {
    pub bundle_hash: String,
    pub openclaw_version: String,
    pub protocol_features: Vec<String>,
}

impl WorkerAdmissionHandshakeSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_worker_bundle_hash("bundleHash", &self.bundle_hash)?;
        validate_string_length_range("openclawVersion", &self.openclaw_version, 1, 128)?;
        if self.protocol_features.len() > WORKER_PROTOCOL_MAX_FEATURES {
            return Err(format!(
                "protocolFeatures: expected at most {} items, got {}",
                WORKER_PROTOCOL_MAX_FEATURES,
                self.protocol_features.len()
            ));
        }
        // uniqueItems enforcement.
        let mut seen = std::collections::HashSet::new();
        for f in &self.protocol_features {
            validate_worker_protocol_feature("protocolFeatures[]", f)?;
            if !seen.insert(f.as_str()) {
                return Err(format!(
                    "protocolFeatures: expected unique items, got duplicate {:?}",
                    f
                ));
            }
        }
        Ok(())
    }
}

// ============================================================================
// Worker connect (first-frame payload)
// ============================================================================

/// Worker hello client block (id + version + platform + mode).
/// 对齐 TS:
///   `Type.Object({
///      id: Type.Literal(GATEWAY_CLIENT_IDS.WORKER),
///      version: Type.String({ minLength: 1, maxLength: 128 }),
///      platform: Type.String({ minLength: 1, maxLength: 128 }),
///      mode: Type.Literal(GATEWAY_CLIENT_MODES.WORKER),
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkerHelloClientSchema {
    /// TS 强约束为字面量 "cradle-ring-worker"; 这里用字符串允许反序列化
    /// 容忍大小写差异, validate 时再校验.
    pub id: String,
    pub version: String,
    pub platform: String,
    pub mode: String,
}

impl WorkerHelloClientSchema {
    pub fn validate(&self) -> Result<(), String> {
        if self.id != gateway_client_ids::WORKER {
            return Err(format!(
                "client.id: expected literal {:?}, got {:?}",
                gateway_client_ids::WORKER,
                self.id
            ));
        }
        validate_string_length_range("client.version", &self.version, 1, 128)?;
        validate_string_length_range("client.platform", &self.platform, 1, 128)?;
        if self.mode != gateway_client_modes::WORKER {
            return Err(format!(
                "client.mode: expected literal {:?}, got {:?}",
                gateway_client_modes::WORKER,
                self.mode
            ));
        }
        Ok(())
    }
}

/// Worker admission block embedded in the connect frame.
/// 对齐 TS:
///   `Type.Object({
///      environmentId: WorkerIdentifierSchema,
///      credential: WorkerCredentialSchema,
///      sessionId: Type.Union([WorkerIdentifierSchema, Type.Null()]),
///      ownerEpoch: Type.Integer({ minimum: 0, maximum: Number.MAX_SAFE_INTEGER }),
///      rpcSetVersion: Type.Integer({ minimum: 1, maximum: Number.MAX_SAFE_INTEGER }),
///      handshake: WorkerAdmissionHandshakeSchema,
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkerAdmissionSchema {
    pub environment_id: String,
    pub credential: String,
    pub session_id: Option<String>,
    pub owner_epoch: i64,
    pub rpc_set_version: i64,
    pub handshake: WorkerAdmissionHandshakeSchema,
}

impl WorkerAdmissionSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_worker_identifier("environmentId", &self.environment_id)?;
        validate_worker_credential("credential", &self.credential)?;
        if let Some(sid) = &self.session_id {
            validate_worker_identifier("sessionId", sid)?;
        }
        validate_non_negative_integer("ownerEpoch", self.owner_epoch)?;
        validate_positive_integer("rpcSetVersion", self.rpc_set_version)?;
        self.handshake
            .validate()
            .map_err(|e| format!("handshake: {}", e))?;
        Ok(())
    }
}

/// Dedicated first-frame payload accepted only on the worker ingress.
/// 对齐 TS:
///   `Type.Object({
///      minProtocol: Type.Integer({ minimum: 1 }),
///      maxProtocol: Type.Integer({ minimum: 1 }),
///      client: Type.Object({...}, { additionalProperties: false }),
///      role: Type.Literal("worker"),
///      admission: Type.Object({...}, { additionalProperties: false }),
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkerConnectParamsSchema {
    pub min_protocol: i64,
    pub max_protocol: i64,
    pub client: WorkerHelloClientSchema,
    pub role: WorkerConnectRole,
    pub admission: WorkerAdmissionSchema,
}

/// Literal marker for `WorkerConnectParamsSchema.role`.
/// 对齐 TS: `Type.Literal("worker")`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum WorkerConnectRole {
    #[serde(rename = "worker")]
    Worker,
}

impl WorkerConnectRole {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Worker => "worker",
        }
    }
}

impl WorkerConnectParamsSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_positive_integer("minProtocol", self.min_protocol)?;
        validate_positive_integer("maxProtocol", self.max_protocol)?;
        self.client.validate().map_err(|e| format!("client: {}", e))?;
        self.admission
            .validate()
            .map_err(|e| format!("admission: {}", e))?;
        Ok(())
    }
}

// ============================================================================
// Connect request frame
// ============================================================================

/// Connect request frame envelope.
/// 对齐 TS:
///   `Type.Object({
///      type: Type.Literal("req"),
///      id: WorkerFrameIdSchema,
///      method: Type.Literal("connect"),
///      params: WorkerConnectParamsSchema,
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkerConnectRequestFrameSchema {
    #[serde(rename = "type")]
    pub frame_type: WorkerFrameTypeReq,
    pub id: String,
    /// TS 字面量 "connect"; Rust 端用 String 承载并由 validate 校验.
    pub method: String,
    pub params: WorkerConnectParamsSchema,
}

impl WorkerConnectRequestFrameSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_worker_frame_id("id", &self.id)?;
        if self.method != "connect" {
            return Err(format!(
                "method: expected literal \"connect\", got {:?}",
                self.method
            ));
        }
        self.params.validate().map_err(|e| format!("params: {}", e))?;
        Ok(())
    }
}

// ============================================================================
// Admission failure / protocol close reasons
// ============================================================================

/// Admission failure reason.
/// 对齐 TS:
///   `Type.Union([
///      Type.Literal("invalid-credential"),
///      Type.Literal("credential-expired"),
///      Type.Literal("environment-mismatch"),
///      Type.Literal("environment-unavailable"),
///      Type.Literal("bundle-mismatch"),
///      Type.Literal("version-mismatch"),
///      Type.Literal("session-mismatch"),
///      Type.Literal("owner-epoch-mismatch"),
///      Type.Literal("rpc-set-mismatch"),
///      Type.Literal("protocol-features-mismatch"),
///   ])`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum WorkerAdmissionFailureReasonSchema {
    #[serde(rename = "invalid-credential")]
    InvalidCredential,
    #[serde(rename = "credential-expired")]
    CredentialExpired,
    #[serde(rename = "environment-mismatch")]
    EnvironmentMismatch,
    #[serde(rename = "environment-unavailable")]
    EnvironmentUnavailable,
    #[serde(rename = "bundle-mismatch")]
    BundleMismatch,
    #[serde(rename = "version-mismatch")]
    VersionMismatch,
    #[serde(rename = "session-mismatch")]
    SessionMismatch,
    #[serde(rename = "owner-epoch-mismatch")]
    OwnerEpochMismatch,
    #[serde(rename = "rpc-set-mismatch")]
    RpcSetMismatch,
    #[serde(rename = "protocol-features-mismatch")]
    ProtocolFeaturesMismatch,
}

impl WorkerAdmissionFailureReasonSchema {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::InvalidCredential => "invalid-credential",
            Self::CredentialExpired => "credential-expired",
            Self::EnvironmentMismatch => "environment-mismatch",
            Self::EnvironmentUnavailable => "environment-unavailable",
            Self::BundleMismatch => "bundle-mismatch",
            Self::VersionMismatch => "version-mismatch",
            Self::SessionMismatch => "session-mismatch",
            Self::OwnerEpochMismatch => "owner-epoch-mismatch",
            Self::RpcSetMismatch => "rpc-set-mismatch",
            Self::ProtocolFeaturesMismatch => "protocol-features-mismatch",
        }
    }
}

pub fn is_valid_worker_admission_failure_reason(s: &str) -> bool {
    matches!(
        s,
        "invalid-credential"
            | "credential-expired"
            | "environment-mismatch"
            | "environment-unavailable"
            | "bundle-mismatch"
            | "version-mismatch"
            | "session-mismatch"
            | "owner-epoch-mismatch"
            | "rpc-set-mismatch"
            | "protocol-features-mismatch"
    )
}

/// Protocol close reason (admission failures + transport-level reasons).
/// 对齐 TS: `Type.Union([WorkerAdmissionFailureReasonSchema,
///   Type.Literal("invalid-handshake"),
///   Type.Literal("protocol-mismatch"),
///   Type.Literal("gateway-unavailable"),
///   Type.Literal("invalid-frame"),
///   Type.Literal("slow-consumer"),
///   Type.Literal("method-not-allowed"),
///   Type.Literal("invalid-heartbeat"),
///   Type.Literal("credential-replaced"),
///   Type.Literal("gateway-shutdown"),
/// ])`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum WorkerProtocolCloseReasonSchema {
    #[serde(rename = "invalid-credential")]
    InvalidCredential,
    #[serde(rename = "credential-expired")]
    CredentialExpired,
    #[serde(rename = "environment-mismatch")]
    EnvironmentMismatch,
    #[serde(rename = "environment-unavailable")]
    EnvironmentUnavailable,
    #[serde(rename = "bundle-mismatch")]
    BundleMismatch,
    #[serde(rename = "version-mismatch")]
    VersionMismatch,
    #[serde(rename = "session-mismatch")]
    SessionMismatch,
    #[serde(rename = "owner-epoch-mismatch")]
    OwnerEpochMismatch,
    #[serde(rename = "rpc-set-mismatch")]
    RpcSetMismatch,
    #[serde(rename = "protocol-features-mismatch")]
    ProtocolFeaturesMismatch,
    #[serde(rename = "invalid-handshake")]
    InvalidHandshake,
    #[serde(rename = "protocol-mismatch")]
    ProtocolMismatch,
    #[serde(rename = "gateway-unavailable")]
    GatewayUnavailable,
    #[serde(rename = "invalid-frame")]
    InvalidFrame,
    #[serde(rename = "slow-consumer")]
    SlowConsumer,
    #[serde(rename = "method-not-allowed")]
    MethodNotAllowed,
    #[serde(rename = "invalid-heartbeat")]
    InvalidHeartbeat,
    #[serde(rename = "credential-replaced")]
    CredentialReplaced,
    #[serde(rename = "gateway-shutdown")]
    GatewayShutdown,
}

impl WorkerProtocolCloseReasonSchema {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::InvalidCredential => "invalid-credential",
            Self::CredentialExpired => "credential-expired",
            Self::EnvironmentMismatch => "environment-mismatch",
            Self::EnvironmentUnavailable => "environment-unavailable",
            Self::BundleMismatch => "bundle-mismatch",
            Self::VersionMismatch => "version-mismatch",
            Self::SessionMismatch => "session-mismatch",
            Self::OwnerEpochMismatch => "owner-epoch-mismatch",
            Self::RpcSetMismatch => "rpc-set-mismatch",
            Self::ProtocolFeaturesMismatch => "protocol-features-mismatch",
            Self::InvalidHandshake => "invalid-handshake",
            Self::ProtocolMismatch => "protocol-mismatch",
            Self::GatewayUnavailable => "gateway-unavailable",
            Self::InvalidFrame => "invalid-frame",
            Self::SlowConsumer => "slow-consumer",
            Self::MethodNotAllowed => "method-not-allowed",
            Self::InvalidHeartbeat => "invalid-heartbeat",
            Self::CredentialReplaced => "credential-replaced",
            Self::GatewayShutdown => "gateway-shutdown",
        }
    }
}

pub fn is_valid_worker_protocol_close_reason(s: &str) -> bool {
    is_valid_worker_admission_failure_reason(s)
        || matches!(
            s,
            "invalid-handshake"
                | "protocol-mismatch"
                | "gateway-unavailable"
                | "invalid-frame"
                | "slow-consumer"
                | "method-not-allowed"
                | "invalid-heartbeat"
                | "credential-replaced"
                | "gateway-shutdown"
        )
}

// ============================================================================
// Worker error shape + frames
// ============================================================================

/// Worker error code (closed enum).
/// 对齐 TS: `Type.Union([Type.Literal("INVALID_REQUEST"), Type.Literal("UNAVAILABLE")])`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum WorkerErrorCodeSchema {
    InvalidRequest,
    Unavailable,
}

impl WorkerErrorCodeSchema {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::InvalidRequest => "INVALID_REQUEST",
            Self::Unavailable => "UNAVAILABLE",
        }
    }
}

/// Worker error details — wraps the close reason.
/// 对齐 TS: `Type.Object({ reason: WorkerProtocolCloseReasonSchema }, ...)`.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkerErrorDetailsSchema {
    pub reason: WorkerProtocolCloseReasonSchema,
}

impl WorkerErrorDetailsSchema {
    pub fn validate(&self) -> Result<(), String> {
        Ok(())
    }
}

/// Standard worker error envelope.
/// 对齐 TS:
///   `Type.Object({
///      code: WorkerErrorCodeSchema,
///      message: Type.String({ minLength: 1, maxLength: 256 }),
///      details: WorkerErrorDetailsSchema,
///      retryable: Type.Optional(Type.Boolean()),
///      retryAfterMs: Type.Optional(Type.Integer({ minimum: 0 })),
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkerErrorShapeSchema {
    pub code: WorkerErrorCodeSchema,
    pub message: String,
    pub details: WorkerErrorDetailsSchema,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub retryable: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub retry_after_ms: Option<i64>,
}

impl WorkerErrorShapeSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_string_length_range("message", &self.message, 1, 256)?;
        self.details
            .validate()
            .map_err(|e| format!("details: {}", e))?;
        validate_optional_non_negative_integer("retryAfterMs", self.retry_after_ms)?;
        Ok(())
    }
}

/// Generic worker error response frame.
/// 对齐 TS:
///   `Type.Object({
///      type: Type.Literal("res"),
///      id: WorkerFrameIdSchema,
///      ok: Type.Literal(false),
///      error: WorkerErrorShapeSchema,
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkerErrorResponseFrameSchema {
    #[serde(rename = "type")]
    pub frame_type: WorkerFrameTypeRes,
    pub id: String,
    pub ok: bool,
    pub error: WorkerErrorShapeSchema,
}

impl WorkerErrorResponseFrameSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_worker_frame_id("id", &self.id)?;
        if self.ok {
            return Err("ok: expected literal false".to_string());
        }
        self.error.validate().map_err(|e| format!("error: {}", e))?;
        Ok(())
    }
}

// ============================================================================
// Hello-ok payload and admission success frame
// ============================================================================

/// Minimal admission response policy block.
/// 对齐 TS:
///   `Type.Object({
///      heartbeatIntervalMs: Type.Integer({ minimum: 1 }),
///      maxPayload: Type.Integer({ minimum: 1 }),
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkerHelloPolicySchema {
    pub heartbeat_interval_ms: i64,
    pub max_payload: i64,
}

impl WorkerHelloPolicySchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_positive_integer("heartbeatIntervalMs", self.heartbeat_interval_ms)?;
        validate_positive_integer("maxPayload", self.max_payload)?;
        Ok(())
    }
}

/// Minimal admission response; workers never receive the general gateway snapshot.
/// 对齐 TS:
///   `Type.Object({
///      type: Type.Literal("worker-hello-ok"),
///      environmentId: WorkerIdentifierSchema,
///      sessionId: Type.Union([WorkerIdentifierSchema, Type.Null()]),
///      ownerEpoch: Type.Integer({ minimum: 0, maximum: Number.MAX_SAFE_INTEGER }),
///      rpcSetVersion: Type.Integer({ minimum: 1, maximum: Number.MAX_SAFE_INTEGER }),
///      protocolFeatures: Type.Array(WorkerProtocolFeatureSchema, {
///        maxItems: WORKER_PROTOCOL_MAX_FEATURES, uniqueItems: true,
///      }),
///      credentialExpiresAtMs: Type.Integer({ minimum: 0 }),
///      policy: Type.Object({...}, { additionalProperties: false }),
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkerHelloOkSchema {
    #[serde(rename = "type")]
    pub payload_type: WorkerHelloOkType,
    pub environment_id: String,
    pub session_id: Option<String>,
    pub owner_epoch: i64,
    pub rpc_set_version: i64,
    pub protocol_features: Vec<String>,
    pub credential_expires_at_ms: i64,
    pub policy: WorkerHelloPolicySchema,
}

/// Literal marker for `WorkerHelloOkSchema.payload_type`.
/// 对齐 TS: `Type.Literal("worker-hello-ok")`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum WorkerHelloOkType {
    #[serde(rename = "worker-hello-ok")]
    WorkerHelloOk,
}

impl WorkerHelloOkType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::WorkerHelloOk => "worker-hello-ok",
        }
    }
}

impl WorkerHelloOkSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_worker_identifier("environmentId", &self.environment_id)?;
        if let Some(sid) = &self.session_id {
            validate_worker_identifier("sessionId", sid)?;
        }
        validate_non_negative_integer("ownerEpoch", self.owner_epoch)?;
        validate_positive_integer("rpcSetVersion", self.rpc_set_version)?;
        if self.protocol_features.len() > WORKER_PROTOCOL_MAX_FEATURES {
            return Err(format!(
                "protocolFeatures: expected at most {} items, got {}",
                WORKER_PROTOCOL_MAX_FEATURES,
                self.protocol_features.len()
            ));
        }
        let mut seen = std::collections::HashSet::new();
        for f in &self.protocol_features {
            validate_worker_protocol_feature("protocolFeatures[]", f)?;
            if !seen.insert(f.as_str()) {
                return Err(format!(
                    "protocolFeatures: expected unique items, got duplicate {:?}",
                    f
                ));
            }
        }
        validate_non_negative_integer("credentialExpiresAtMs", self.credential_expires_at_ms)?;
        self.policy.validate().map_err(|e| format!("policy: {}", e))?;
        Ok(())
    }
}

/// Successful admission response frame.
/// 对齐 TS:
///   `Type.Object({
///      type: Type.Literal("res"),
///      id: WorkerFrameIdSchema,
///      ok: Type.Literal(true),
///      payload: WorkerHelloOkSchema,
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkerAdmissionSuccessResponseFrameSchema {
    #[serde(rename = "type")]
    pub frame_type: WorkerFrameTypeRes,
    pub id: String,
    pub ok: bool,
    pub payload: WorkerHelloOkSchema,
}

impl WorkerAdmissionSuccessResponseFrameSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_worker_frame_id("id", &self.id)?;
        if !self.ok {
            return Err("ok: expected literal true".to_string());
        }
        self.payload
            .validate()
            .map_err(|e| format!("payload: {}", e))?;
        Ok(())
    }
}

/// Admission response: success or error.
/// 对齐 TS: `Type.Union([WorkerAdmissionSuccessResponseFrameSchema, WorkerErrorResponseFrameSchema])`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum WorkerAdmissionResponseFrameSchema {
    Success(WorkerAdmissionSuccessResponseFrameSchema),
    Error(WorkerErrorResponseFrameSchema),
}

impl WorkerAdmissionResponseFrameSchema {
    pub fn validate(&self) -> Result<(), String> {
        match self {
            Self::Success(s) => s.validate(),
            Self::Error(e) => e.validate(),
        }
    }
}

// ============================================================================
// Worker heartbeat (request/response)
// ============================================================================

/// Worker status discriminator.
/// 对齐 TS: `Type.Union([Type.Literal("ready"), Type.Literal("busy"), Type.Literal("draining")])`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum WorkerStatusSchema {
    Ready,
    Busy,
    Draining,
}

impl WorkerStatusSchema {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Ready => "ready",
            Self::Busy => "busy",
            Self::Draining => "draining",
        }
    }
}

/// Heartbeat request params.
/// 对齐 TS:
///   `Type.Object({
///      sentAtMs: Type.Integer({ minimum: 0 }),
///      status: WorkerStatusSchema,
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkerHeartbeatParamsSchema {
    pub sent_at_ms: i64,
    pub status: WorkerStatusSchema,
}

impl WorkerHeartbeatParamsSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_non_negative_integer("sentAtMs", self.sent_at_ms)?;
        Ok(())
    }
}

/// Heartbeat result block.
/// 对齐 TS:
///   `Type.Object({
///      receivedAtMs: Type.Integer({ minimum: 0 }),
///      status: Type.Literal("ok"),
///      ownerEpoch: Type.Integer({ minimum: 0, maximum: Number.MAX_SAFE_INTEGER }),
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkerHeartbeatResultSchema {
    pub received_at_ms: i64,
    pub status: WorkerHeartbeatStatus,
    pub owner_epoch: i64,
}

/// Literal marker for `WorkerHeartbeatResultSchema.status`.
/// 对齐 TS: `Type.Literal("ok")`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum WorkerHeartbeatStatus {
    #[serde(rename = "ok")]
    Ok,
}

impl WorkerHeartbeatStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Ok => "ok",
        }
    }
}

impl WorkerHeartbeatResultSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_non_negative_integer("receivedAtMs", self.received_at_ms)?;
        validate_non_negative_integer("ownerEpoch", self.owner_epoch)?;
        Ok(())
    }
}

/// Heartbeat request frame.
/// 对齐 TS:
///   `Type.Object({
///      type: Type.Literal("req"),
///      id: WorkerFrameIdSchema,
///      method: Type.Literal("worker.heartbeat"),
///      params: WorkerHeartbeatParamsSchema,
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkerHeartbeatRequestFrameSchema {
    #[serde(rename = "type")]
    pub frame_type: WorkerFrameTypeReq,
    pub id: String,
    pub method: String,
    pub params: WorkerHeartbeatParamsSchema,
}

impl WorkerHeartbeatRequestFrameSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_worker_frame_id("id", &self.id)?;
        if self.method != WORKER_PROTOCOL_METHODS[0] {
            return Err(format!(
                "method: expected literal {:?}, got {:?}",
                WORKER_PROTOCOL_METHODS[0],
                self.method
            ));
        }
        self.params.validate().map_err(|e| format!("params: {}", e))?;
        Ok(())
    }
}

/// Heartbeat success response frame.
/// 对齐 TS:
///   `Type.Object({
///      type: Type.Literal("res"),
///      id: WorkerFrameIdSchema,
///      ok: Type.Literal(true),
///      payload: WorkerHeartbeatResultSchema,
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkerHeartbeatSuccessResponseFrameSchema {
    #[serde(rename = "type")]
    pub frame_type: WorkerFrameTypeRes,
    pub id: String,
    pub ok: bool,
    pub payload: WorkerHeartbeatResultSchema,
}

impl WorkerHeartbeatSuccessResponseFrameSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_worker_frame_id("id", &self.id)?;
        if !self.ok {
            return Err("ok: expected literal true".to_string());
        }
        self.payload
            .validate()
            .map_err(|e| format!("payload: {}", e))?;
        Ok(())
    }
}

/// Heartbeat response frame.
/// 对齐 TS: `Type.Union([success, error])`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum WorkerHeartbeatResponseFrameSchema {
    Success(WorkerHeartbeatSuccessResponseFrameSchema),
    Error(WorkerErrorResponseFrameSchema),
}

impl WorkerHeartbeatResponseFrameSchema {
    pub fn validate(&self) -> Result<(), String> {
        match self {
            Self::Success(s) => s.validate(),
            Self::Error(e) => e.validate(),
        }
    }
}

// ============================================================================
// Worker transcript content blocks
// ============================================================================

/// Text content part.
/// 对齐 TS:
///   `Type.Object({
///      type: Type.Literal("text"),
///      text: Type.String({ maxLength: WORKER_PROTOCOL_MAX_PAYLOAD_BYTES }),
///      textSignature: Type.Optional(Type.String({
///        minLength: 1, maxLength: WORKER_PROTOCOL_MAX_PAYLOAD_BYTES,
///      })),
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkerTranscriptTextContentSchema {
    #[serde(rename = "type")]
    pub content_type: WorkerTranscriptContentTypeText,
    pub text: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text_signature: Option<String>,
}

/// Literal marker for text content type.
/// 对齐 TS: `Type.Literal("text")`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum WorkerTranscriptContentTypeText {
    #[serde(rename = "text")]
    Text,
}

impl WorkerTranscriptContentTypeText {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Text => "text",
        }
    }
}

impl WorkerTranscriptTextContentSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_string_length_range("text", &self.text, 0, WORKER_PROTOCOL_MAX_PAYLOAD_BYTES)?;
        if let Some(sig) = &self.text_signature {
            validate_string_length_range("textSignature", sig, 1, WORKER_PROTOCOL_MAX_PAYLOAD_BYTES)?;
        }
        Ok(())
    }
}

/// Thinking content part.
/// 对齐 TS:
///   `Type.Object({
///      type: Type.Literal("thinking"),
///      thinking: Type.String({ maxLength: WORKER_PROTOCOL_MAX_PAYLOAD_BYTES }),
///      thinkingSignature: Type.Optional(Type.String({
///        minLength: 1, maxLength: WORKER_PROTOCOL_MAX_PAYLOAD_BYTES,
///      })),
///      redacted: Type.Optional(Type.Boolean()),
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkerTranscriptThinkingContentSchema {
    #[serde(rename = "type")]
    pub content_type: WorkerTranscriptContentTypeThinking,
    pub thinking: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub thinking_signature: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub redacted: Option<bool>,
}

/// Literal marker for thinking content type.
/// 对齐 TS: `Type.Literal("thinking")`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum WorkerTranscriptContentTypeThinking {
    #[serde(rename = "thinking")]
    Thinking,
}

impl WorkerTranscriptContentTypeThinking {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Thinking => "thinking",
        }
    }
}

impl WorkerTranscriptThinkingContentSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_string_length_range(
            "thinking",
            &self.thinking,
            0,
            WORKER_PROTOCOL_MAX_PAYLOAD_BYTES,
        )?;
        if let Some(sig) = &self.thinking_signature {
            validate_string_length_range(
                "thinkingSignature",
                sig,
                1,
                WORKER_PROTOCOL_MAX_PAYLOAD_BYTES,
            )?;
        }
        Ok(())
    }
}

/// Image content part.
/// 对齐 TS:
///   `Type.Object({
///      type: Type.Literal("image"),
///      data: Type.String({ minLength: 1, maxLength: WORKER_PROTOCOL_MAX_PAYLOAD_BYTES }),
///      mimeType: Type.String({ minLength: 1, maxLength: 256 }),
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkerTranscriptImageContentSchema {
    #[serde(rename = "type")]
    pub content_type: WorkerTranscriptContentTypeImage,
    pub data: String,
    pub mime_type: String,
}

/// Literal marker for image content type.
/// 对齐 TS: `Type.Literal("image")`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum WorkerTranscriptContentTypeImage {
    #[serde(rename = "image")]
    Image,
}

impl WorkerTranscriptContentTypeImage {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Image => "image",
        }
    }
}

impl WorkerTranscriptImageContentSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_string_length_range(
            "data",
            &self.data,
            1,
            WORKER_PROTOCOL_MAX_PAYLOAD_BYTES,
        )?;
        validate_string_length_range("mimeType", &self.mime_type, 1, 256)?;
        Ok(())
    }
}

/// Tool-call content part.
/// 对齐 TS:
///   `Type.Object({
///      type: Type.Literal("toolCall"),
///      id: WorkerIdentifierSchema,
///      name: WorkerIdentifierSchema,
///      arguments: Type.Record(Type.String({ minLength: 1, maxLength: 256 }), Type.Unknown()),
///      thoughtSignature: Type.Optional(Type.String({
///        minLength: 1, maxLength: WORKER_PROTOCOL_MAX_PAYLOAD_BYTES,
///      })),
///      executionMode: Type.Optional(Type.Union([
///        Type.Literal("sequential"), Type.Literal("parallel"),
///      ])),
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkerTranscriptToolCallSchema {
    #[serde(rename = "type")]
    pub content_type: WorkerTranscriptContentTypeToolCall,
    pub id: String,
    pub name: String,
    pub arguments: std::collections::BTreeMap<String, serde_json::Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub thought_signature: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub execution_mode: Option<WorkerTranscriptToolCallExecutionMode>,
}

/// Literal marker for tool-call content type.
/// 对齐 TS: `Type.Literal("toolCall")`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum WorkerTranscriptContentTypeToolCall {
    #[serde(rename = "toolCall")]
    ToolCall,
}

impl WorkerTranscriptContentTypeToolCall {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::ToolCall => "toolCall",
        }
    }
}

/// Execution mode discriminator for a tool call.
/// 对齐 TS: `Type.Union([Type.Literal("sequential"), Type.Literal("parallel")])`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum WorkerTranscriptToolCallExecutionMode {
    Sequential,
    Parallel,
}

impl WorkerTranscriptToolCallExecutionMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Sequential => "sequential",
            Self::Parallel => "parallel",
        }
    }
}

impl WorkerTranscriptToolCallSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_worker_identifier("id", &self.id)?;
        validate_worker_identifier("name", &self.name)?;
        for (k, _) in &self.arguments {
            let len = k.chars().count();
            if len < 1 || len > 256 {
                return Err(format!(
                    "arguments[{}]: expected key length 1..=256, got {}",
                    k, len
                ));
            }
        }
        if let Some(sig) = &self.thought_signature {
            validate_string_length_range(
                "thoughtSignature",
                sig,
                1,
                WORKER_PROTOCOL_MAX_PAYLOAD_BYTES,
            )?;
        }
        Ok(())
    }
}

// ============================================================================
// Worker transcript usage + diagnostic + messages
// ============================================================================

/// Context usage block.
/// 对齐 TS:
///   `Type.Union([
///      Type.Object({ state: Type.Literal("available"), promptTokens: ..., totalTokens: ... }, ...),
///      Type.Object({ state: Type.Literal("unavailable") }, ...),
///   ])`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "state", rename_all = "lowercase")]
pub enum WorkerTranscriptContextUsageSchema {
    Available {
        prompt_tokens: f64,
        total_tokens: f64,
    },
    Unavailable,
}

impl WorkerTranscriptContextUsageSchema {
    pub fn validate(&self) -> Result<(), String> {
        match self {
            Self::Available {
                prompt_tokens,
                total_tokens,
            } => {
                validate_non_negative_number("contextUsage.promptTokens", *prompt_tokens)?;
                validate_non_negative_number("contextUsage.totalTokens", *total_tokens)?;
            }
            Self::Unavailable => {}
        }
        Ok(())
    }
}

/// Cost breakdown block.
/// 对齐 TS:
///   `Type.Object({
///      input: Type.Number({ minimum: 0 }),
///      output: Type.Number({ minimum: 0 }),
///      cacheRead: Type.Number({ minimum: 0 }),
///      cacheWrite: Type.Number({ minimum: 0 }),
///      total: Type.Number({ minimum: 0 }),
///      totalOrigin: Type.Optional(Type.Literal("provider-billed")),
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkerTranscriptUsageCostSchema {
    pub input: f64,
    pub output: f64,
    pub cache_read: f64,
    pub cache_write: f64,
    pub total: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub total_origin: Option<WorkerTranscriptUsageCostTotalOrigin>,
}

/// Literal marker for cost.totalOrigin.
/// 对齐 TS: `Type.Literal("provider-billed")`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum WorkerTranscriptUsageCostTotalOrigin {
    #[serde(rename = "provider-billed")]
    ProviderBilled,
}

impl WorkerTranscriptUsageCostTotalOrigin {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::ProviderBilled => "provider-billed",
        }
    }
}

impl WorkerTranscriptUsageCostSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_non_negative_number("cost.input", self.input)?;
        validate_non_negative_number("cost.output", self.output)?;
        validate_non_negative_number("cost.cacheRead", self.cache_read)?;
        validate_non_negative_number("cost.cacheWrite", self.cache_write)?;
        validate_non_negative_number("cost.total", self.total)?;
        Ok(())
    }
}

/// Usage block on an assistant transcript message.
/// 对齐 TS:
///   `Type.Object({
///      input: Type.Number({ minimum: 0 }),
///      output: Type.Number({ minimum: 0 }),
///      cacheRead: Type.Number({ minimum: 0 }),
///      cacheWrite: Type.Number({ minimum: 0 }),
///      contextUsage: Type.Optional(Type.Union([...])),
///      totalTokens: Type.Number({ minimum: 0 }),
///      cost: Type.Object({...}, { additionalProperties: false }),
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkerTranscriptUsageSchema {
    pub input: f64,
    pub output: f64,
    pub cache_read: f64,
    pub cache_write: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub context_usage: Option<WorkerTranscriptContextUsageSchema>,
    pub total_tokens: f64,
    pub cost: WorkerTranscriptUsageCostSchema,
}

impl WorkerTranscriptUsageSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_non_negative_number("usage.input", self.input)?;
        validate_non_negative_number("usage.output", self.output)?;
        validate_non_negative_number("usage.cacheRead", self.cache_read)?;
        validate_non_negative_number("usage.cacheWrite", self.cache_write)?;
        validate_non_negative_number("usage.totalTokens", self.total_tokens)?;
        if let Some(cu) = &self.context_usage {
            cu.validate().map_err(|e| format!("contextUsage: {}", e))?;
        }
        self.cost.validate().map_err(|e| format!("cost: {}", e))?;
        Ok(())
    }
}

/// Assistant diagnostic block (per-attempt error or detail payload).
/// 对齐 TS:
///   `Type.Object({
///      type: WorkerIdentifierSchema,
///      timestamp: Type.Integer({ minimum: 0 }),
///      error: Type.Optional(Type.Object({
///        name: Type.Optional(Type.String({ maxLength: 256 })),
///        message: Type.String({ maxLength: WORKER_PROTOCOL_MAX_PAYLOAD_BYTES }),
///        stack: Type.Optional(Type.String({ maxLength: WORKER_PROTOCOL_MAX_PAYLOAD_BYTES })),
///        code: Type.Optional(Type.Union([Type.String({ maxLength: 256 }), Type.Number()])),
///      }, { additionalProperties: false })),
///      details: Type.Optional(Type.Record(Type.String({ minLength: 1, maxLength: 256 }), Type.Unknown())),
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkerTranscriptAssistantDiagnosticSchema {
    #[serde(rename = "type")]
    pub diagnostic_type: String,
    pub timestamp: i64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<WorkerTranscriptAssistantDiagnosticErrorSchema>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub details: Option<std::collections::BTreeMap<String, serde_json::Value>>,
}

/// Diagnostic error block.
/// 对齐 TS: embedded `Type.Object({ name, message, stack, code }, ...)`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkerTranscriptAssistantDiagnosticErrorSchema {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    pub message: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stack: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub code: Option<serde_json::Value>,
}

impl WorkerTranscriptAssistantDiagnosticErrorSchema {
    pub fn validate(&self) -> Result<(), String> {
        if let Some(name) = &self.name {
            validate_string_length_range("error.name", name, 0, 256)?;
        }
        validate_string_length_range(
            "error.message",
            &self.message,
            0,
            WORKER_PROTOCOL_MAX_PAYLOAD_BYTES,
        )?;
        if let Some(stack) = &self.stack {
            validate_string_length_range(
                "error.stack",
                stack,
                0,
                WORKER_PROTOCOL_MAX_PAYLOAD_BYTES,
            )?;
        }
        if let Some(code) = &self.code {
            if let Some(s) = code.as_str() {
                validate_string_length_range("error.code", s, 0, 256)?;
            }
        }
        Ok(())
    }
}

impl WorkerTranscriptAssistantDiagnosticSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_worker_identifier("diagnostic.type", &self.diagnostic_type)?;
        validate_non_negative_integer("diagnostic.timestamp", self.timestamp)?;
        if let Some(err) = &self.error {
            err.validate().map_err(|e| format!("error: {}", e))?;
        }
        Ok(())
    }
}

/// Stop reason discriminator.
/// 对齐 TS:
///   `Type.Union([
///      Type.Literal("stop"),
///      Type.Literal("length"),
///      Type.Literal("toolUse"),
///      Type.Literal("error"),
///      Type.Literal("aborted"),
///   ])`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum WorkerTranscriptAssistantStopReason {
    Stop,
    Length,
    #[serde(rename = "toolUse")]
    ToolUse,
    Error,
    Aborted,
}

impl WorkerTranscriptAssistantStopReason {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Stop => "stop",
            Self::Length => "length",
            Self::ToolUse => "toolUse",
            Self::Error => "error",
            Self::Aborted => "aborted",
        }
    }
}

/// User transcript message.
/// 对齐 TS:
///   `Type.Object({
///      role: Type.Literal("user"),
///      content: Type.Array(Type.Union([text, image]), {
///        minItems: 1, maxItems: WORKER_TRANSCRIPT_MAX_CONTENT_PARTS,
///      }),
///      timestamp: Type.Integer({ minimum: 0 }),
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkerTranscriptUserMessageSchema {
    pub role: WorkerTranscriptRoleUser,
    pub content: Vec<WorkerTranscriptUserContentPart>,
    pub timestamp: i64,
}

/// Literal marker for user message role.
/// 对齐 TS: `Type.Literal("user")`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum WorkerTranscriptRoleUser {
    #[serde(rename = "user")]
    User,
}

impl WorkerTranscriptRoleUser {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::User => "user",
        }
    }
}

/// Discriminated union for user message content parts (text | image).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum WorkerTranscriptUserContentPart {
    Text(WorkerTranscriptTextContentSchema),
    Image(WorkerTranscriptImageContentSchema),
}

impl WorkerTranscriptUserContentPart {
    pub fn validate(&self) -> Result<(), String> {
        match self {
            Self::Text(t) => t.validate(),
            Self::Image(i) => i.validate(),
        }
    }
}

impl WorkerTranscriptUserMessageSchema {
    pub fn validate(&self) -> Result<(), String> {
        if self.content.is_empty() {
            return Err("content: expected at least 1 item, got 0".to_string());
        }
        if self.content.len() > WORKER_TRANSCRIPT_MAX_CONTENT_PARTS {
            return Err(format!(
                "content: expected at most {} items, got {}",
                WORKER_TRANSCRIPT_MAX_CONTENT_PARTS,
                self.content.len()
            ));
        }
        for (i, part) in self.content.iter().enumerate() {
            part.validate().map_err(|e| format!("content[{}]: {}", i, e))?;
        }
        validate_non_negative_integer("timestamp", self.timestamp)?;
        Ok(())
    }
}

/// Assistant transcript message.
/// 对齐 TS: long `Type.Object(...)` with role / content union / api / provider /
/// model / responseModel / responseId / diagnostics / usage / stopReason /
/// error* / timestamp.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkerTranscriptAssistantMessageSchema {
    pub role: WorkerTranscriptRoleAssistant,
    pub content: Vec<WorkerTranscriptAssistantContentPart>,
    pub api: String,
    pub provider: String,
    pub model: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub response_model: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub response_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub diagnostics: Option<Vec<WorkerTranscriptAssistantDiagnosticSchema>>,
    pub usage: WorkerTranscriptUsageSchema,
    pub stop_reason: WorkerTranscriptAssistantStopReason,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error_message: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error_code: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error_type: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error_body: Option<String>,
    pub timestamp: i64,
}

/// Literal marker for assistant message role.
/// 对齐 TS: `Type.Literal("assistant")`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum WorkerTranscriptRoleAssistant {
    #[serde(rename = "assistant")]
    Assistant,
}

impl WorkerTranscriptRoleAssistant {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Assistant => "assistant",
        }
    }
}

/// Discriminated union for assistant content parts (text | thinking | toolCall).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum WorkerTranscriptAssistantContentPart {
    Text(WorkerTranscriptTextContentSchema),
    Thinking(WorkerTranscriptThinkingContentSchema),
    ToolCall(WorkerTranscriptToolCallSchema),
}

impl WorkerTranscriptAssistantContentPart {
    pub fn validate(&self) -> Result<(), String> {
        match self {
            Self::Text(t) => t.validate(),
            Self::Thinking(t) => t.validate(),
            Self::ToolCall(t) => t.validate(),
        }
    }
}

impl WorkerTranscriptAssistantMessageSchema {
    pub fn validate(&self) -> Result<(), String> {
        if self.content.len() > WORKER_TRANSCRIPT_MAX_CONTENT_PARTS {
            return Err(format!(
                "content: expected at most {} items, got {}",
                WORKER_TRANSCRIPT_MAX_CONTENT_PARTS,
                self.content.len()
            ));
        }
        for (i, part) in self.content.iter().enumerate() {
            part.validate().map_err(|e| format!("content[{}]: {}", i, e))?;
        }
        validate_worker_identifier("api", &self.api)?;
        validate_worker_identifier("provider", &self.provider)?;
        validate_worker_identifier("model", &self.model)?;
        if let Some(rm) = &self.response_model {
            validate_worker_identifier("responseModel", rm)?;
        }
        if let Some(rid) = &self.response_id {
            validate_worker_identifier("responseId", rid)?;
        }
        if let Some(diags) = &self.diagnostics {
            if diags.len() > WORKER_TRANSCRIPT_MAX_CONTENT_PARTS {
                return Err(format!(
                    "diagnostics: expected at most {} items, got {}",
                    WORKER_TRANSCRIPT_MAX_CONTENT_PARTS,
                    diags.len()
                ));
            }
            for (i, d) in diags.iter().enumerate() {
                d.validate().map_err(|e| format!("diagnostics[{}]: {}", i, e))?;
            }
        }
        self.usage.validate().map_err(|e| format!("usage: {}", e))?;
        if let Some(em) = &self.error_message {
            validate_string_length_range(
                "errorMessage",
                em,
                0,
                WORKER_PROTOCOL_MAX_PAYLOAD_BYTES,
            )?;
        }
        if let Some(ec) = &self.error_code {
            validate_string_length_range("errorCode", ec, 0, 256)?;
        }
        if let Some(et) = &self.error_type {
            validate_string_length_range("errorType", et, 0, 256)?;
        }
        if let Some(eb) = &self.error_body {
            validate_string_length_range("errorBody", eb, 0, WORKER_PROTOCOL_MAX_PAYLOAD_BYTES)?;
        }
        validate_non_negative_integer("timestamp", self.timestamp)?;
        Ok(())
    }
}

/// Tool-result transcript message.
/// 对齐 TS:
///   `Type.Object({
///      role: Type.Literal("toolResult"),
///      toolCallId: WorkerIdentifierSchema,
///      toolName: WorkerIdentifierSchema,
///      content: Type.Array(Type.Union([text, image]), { maxItems: ... }),
///      details: Type.Optional(Type.Unknown()),
///      isError: Type.Boolean(),
///      timestamp: Type.Integer({ minimum: 0 }),
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkerTranscriptToolResultMessageSchema {
    pub role: WorkerTranscriptRoleToolResult,
    pub tool_call_id: String,
    pub tool_name: String,
    pub content: Vec<WorkerTranscriptUserContentPart>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
    pub is_error: bool,
    pub timestamp: i64,
}

/// Literal marker for tool-result message role.
/// 对齐 TS: `Type.Literal("toolResult")`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum WorkerTranscriptRoleToolResult {
    #[serde(rename = "toolResult")]
    ToolResult,
}

impl WorkerTranscriptRoleToolResult {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::ToolResult => "toolResult",
        }
    }
}

impl WorkerTranscriptToolResultMessageSchema {
    pub fn validate(&self) -> Result<(), String> {
        if self.content.len() > WORKER_TRANSCRIPT_MAX_CONTENT_PARTS {
            return Err(format!(
                "content: expected at most {} items, got {}",
                WORKER_TRANSCRIPT_MAX_CONTENT_PARTS,
                self.content.len()
            ));
        }
        for (i, part) in self.content.iter().enumerate() {
            part.validate().map_err(|e| format!("content[{}]: {}", i, e))?;
        }
        validate_worker_identifier("toolCallId", &self.tool_call_id)?;
        validate_worker_identifier("toolName", &self.tool_name)?;
        validate_non_negative_integer("timestamp", self.timestamp)?;
        Ok(())
    }
}

/// Discriminated union of all worker transcript messages.
/// 对齐 TS: `Type.Union([user, assistant, toolResult])`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum WorkerTranscriptMessageSchema {
    User(WorkerTranscriptUserMessageSchema),
    Assistant(WorkerTranscriptAssistantMessageSchema),
    ToolResult(WorkerTranscriptToolResultMessageSchema),
}

impl WorkerTranscriptMessageSchema {
    pub fn validate(&self) -> Result<(), String> {
        match self {
            Self::User(m) => m.validate(),
            Self::Assistant(m) => m.validate(),
            Self::ToolResult(m) => m.validate(),
        }
    }
}

// ============================================================================
// Worker transcript commit (request/response/error)
// ============================================================================

/// Worker transcript commit params.
/// 对齐 TS:
///   `Type.Object({
///      runEpoch: Type.Integer({ minimum: 0, maximum: Number.MAX_SAFE_INTEGER }),
///      seq: Type.Integer({ minimum: 1, maximum: Number.MAX_SAFE_INTEGER }),
///      baseLeafId: Type.Union([WorkerIdentifierSchema, Type.Null()]),
///      messages: Type.Array(WorkerTranscriptMessageSchema, {
///        minItems: 1, maxItems: WORKER_TRANSCRIPT_MAX_BATCH_MESSAGES,
///      }),
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkerTranscriptCommitParamsSchema {
    pub run_epoch: i64,
    pub seq: i64,
    pub base_leaf_id: Option<String>,
    pub messages: Vec<WorkerTranscriptMessageSchema>,
}

impl WorkerTranscriptCommitParamsSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_non_negative_integer("runEpoch", self.run_epoch)?;
        validate_positive_integer("seq", self.seq)?;
        if let Some(leaf) = &self.base_leaf_id {
            validate_worker_identifier("baseLeafId", leaf)?;
        }
        if self.messages.is_empty() {
            return Err("messages: expected at least 1 item, got 0".to_string());
        }
        if self.messages.len() > WORKER_TRANSCRIPT_MAX_BATCH_MESSAGES {
            return Err(format!(
                "messages: expected at most {} items, got {}",
                WORKER_TRANSCRIPT_MAX_BATCH_MESSAGES,
                self.messages.len()
            ));
        }
        for (i, m) in self.messages.iter().enumerate() {
            m.validate().map_err(|e| format!("messages[{}]: {}", i, e))?;
        }
        Ok(())
    }
}

/// Worker transcript commit result.
/// 对齐 TS:
///   `Type.Object({
///      entryIds: Type.Array(WorkerIdentifierSchema, {
///        minItems: 1, maxItems: WORKER_TRANSCRIPT_MAX_BATCH_MESSAGES,
///      }),
///      newLeafId: WorkerIdentifierSchema,
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkerTranscriptCommitResultSchema {
    pub entry_ids: Vec<String>,
    pub new_leaf_id: String,
}

impl WorkerTranscriptCommitResultSchema {
    pub fn validate(&self) -> Result<(), String> {
        if self.entry_ids.is_empty() {
            return Err("entryIds: expected at least 1 item, got 0".to_string());
        }
        if self.entry_ids.len() > WORKER_TRANSCRIPT_MAX_BATCH_MESSAGES {
            return Err(format!(
                "entryIds: expected at most {} items, got {}",
                WORKER_TRANSCRIPT_MAX_BATCH_MESSAGES,
                self.entry_ids.len()
            ));
        }
        for (i, e) in self.entry_ids.iter().enumerate() {
            validate_worker_identifier(&format!("entryIds[{}]", i), e)?;
        }
        validate_worker_identifier("newLeafId", &self.new_leaf_id)?;
        Ok(())
    }
}

/// Transcript commit error reason.
/// 对齐 TS:
///   `Type.Union([
///      Type.Literal("stale-base-leaf"),
///      Type.Literal("epoch-mismatch"),
///      Type.Literal("invalid-batch"),
///      Type.Literal("session-not-attached"),
///   ])`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum WorkerTranscriptCommitErrorReasonSchema {
    #[serde(rename = "stale-base-leaf")]
    StaleBaseLeaf,
    #[serde(rename = "epoch-mismatch")]
    EpochMismatch,
    #[serde(rename = "invalid-batch")]
    InvalidBatch,
    #[serde(rename = "session-not-attached")]
    SessionNotAttached,
}

impl WorkerTranscriptCommitErrorReasonSchema {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::StaleBaseLeaf => "stale-base-leaf",
            Self::EpochMismatch => "epoch-mismatch",
            Self::InvalidBatch => "invalid-batch",
            Self::SessionNotAttached => "session-not-attached",
        }
    }
}

/// Transcript commit error details block.
/// 对齐 TS: `Type.Object({ reason: WorkerTranscriptCommitErrorReasonSchema }, ...)`.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkerTranscriptCommitErrorDetailsSchema {
    pub reason: WorkerTranscriptCommitErrorReasonSchema,
}

impl WorkerTranscriptCommitErrorDetailsSchema {
    pub fn validate(&self) -> Result<(), String> {
        Ok(())
    }
}

/// Transcript commit error shape.
/// 对齐 TS:
///   `Type.Object({
///      code: Type.Literal("INVALID_REQUEST"),
///      message: Type.String({ minLength: 1, maxLength: 256 }),
///      details: Type.Object({ reason: ... }, { additionalProperties: false }),
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkerTranscriptCommitErrorShapeSchema {
    pub code: WorkerErrorCodeSchema,
    pub message: String,
    pub details: WorkerTranscriptCommitErrorDetailsSchema,
}

impl WorkerTranscriptCommitErrorShapeSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_string_length_range("message", &self.message, 1, 256)?;
        self.details
            .validate()
            .map_err(|e| format!("details: {}", e))?;
        Ok(())
    }
}

/// Transcript commit request frame.
/// 对齐 TS:
///   `Type.Object({
///      type: Type.Literal("req"),
///      id: WorkerFrameIdSchema,
///      method: Type.Literal("worker.transcript.commit"),
///      params: WorkerTranscriptCommitParamsSchema,
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkerTranscriptCommitRequestFrameSchema {
    #[serde(rename = "type")]
    pub frame_type: WorkerFrameTypeReq,
    pub id: String,
    pub method: String,
    pub params: WorkerTranscriptCommitParamsSchema,
}

impl WorkerTranscriptCommitRequestFrameSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_worker_frame_id("id", &self.id)?;
        if self.method != WORKER_PROTOCOL_METHODS[1] {
            return Err(format!(
                "method: expected literal {:?}, got {:?}",
                WORKER_PROTOCOL_METHODS[1],
                self.method
            ));
        }
        self.params.validate().map_err(|e| format!("params: {}", e))?;
        Ok(())
    }
}

/// Transcript commit success response frame.
/// 对齐 TS:
///   `Type.Object({
///      type: Type.Literal("res"),
///      id: WorkerFrameIdSchema,
///      ok: Type.Literal(true),
///      payload: WorkerTranscriptCommitResultSchema,
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkerTranscriptCommitSuccessResponseFrameSchema {
    #[serde(rename = "type")]
    pub frame_type: WorkerFrameTypeRes,
    pub id: String,
    pub ok: bool,
    pub payload: WorkerTranscriptCommitResultSchema,
}

impl WorkerTranscriptCommitSuccessResponseFrameSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_worker_frame_id("id", &self.id)?;
        if !self.ok {
            return Err("ok: expected literal true".to_string());
        }
        self.payload
            .validate()
            .map_err(|e| format!("payload: {}", e))?;
        Ok(())
    }
}

/// Transcript commit error response frame.
/// 对齐 TS:
///   `Type.Object({
///      type: Type.Literal("res"),
///      id: WorkerFrameIdSchema,
///      ok: Type.Literal(false),
///      error: WorkerTranscriptCommitErrorShapeSchema,
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkerTranscriptCommitErrorResponseFrameSchema {
    #[serde(rename = "type")]
    pub frame_type: WorkerFrameTypeRes,
    pub id: String,
    pub ok: bool,
    pub error: WorkerTranscriptCommitErrorShapeSchema,
}

impl WorkerTranscriptCommitErrorResponseFrameSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_worker_frame_id("id", &self.id)?;
        if self.ok {
            return Err("ok: expected literal false".to_string());
        }
        self.error.validate().map_err(|e| format!("error: {}", e))?;
        Ok(())
    }
}

/// Transcript commit response frame.
/// 对齐 TS: `Type.Union([success, error, genericError])`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum WorkerTranscriptCommitResponseFrameSchema {
    Success(WorkerTranscriptCommitSuccessResponseFrameSchema),
    CommitError(WorkerTranscriptCommitErrorResponseFrameSchema),
    Error(WorkerErrorResponseFrameSchema),
}

impl WorkerTranscriptCommitResponseFrameSchema {
    pub fn validate(&self) -> Result<(), String> {
        match self {
            Self::Success(s) => s.validate(),
            Self::CommitError(e) => e.validate(),
            Self::Error(e) => e.validate(),
        }
    }
}

// ============================================================================
// Worker live event (assistant / thinking / tool / approval / lifecycle)
// ============================================================================

/// Live event kind discriminator.
/// 对齐 TS: 5-variant union (assistant / thinking / tool / approval / lifecycle).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum WorkerLiveEventKind {
    Assistant,
    Thinking,
    Tool,
    Approval,
    Lifecycle,
}

impl WorkerLiveEventKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Assistant => "assistant",
            Self::Thinking => "thinking",
            Self::Tool => "tool",
            Self::Approval => "approval",
            Self::Lifecycle => "lifecycle",
        }
    }
}

/// Assistant live event phase.
/// 对齐 TS: `Type.Union([Type.Literal("commentary"), Type.Literal("final_answer")])`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WorkerLiveAssistantPhase {
    Commentary,
    #[serde(rename = "final_answer")]
    FinalAnswer,
}

impl WorkerLiveAssistantPhase {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Commentary => "commentary",
            Self::FinalAnswer => "final_answer",
        }
    }
}

/// Literal marker for `replace` field.
/// 对齐 TS: `Type.Literal(true)`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum WorkerLiveAssistantReplace {
    #[serde(rename = "true")]
    True,
}

impl WorkerLiveAssistantReplace {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::True => "true",
        }
    }
}

/// Assistant live event payload.
/// 对齐 TS:
///   `workerLiveObject({
///      text, delta,
///      replace: Type.Optional(Type.Literal(true)),
///      mediaUrls: Type.Optional(Type.Array(LiveIdentifierSchema, { maxItems: WORKER_TRANSCRIPT_MAX_CONTENT_PARTS })),
///      phase: Type.Optional(Type.Union([Type.Literal("commentary"), Type.Literal("final_answer")])),
///      itemId: Type.Optional(WorkerIdentifierSchema),
///   })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkerLiveAssistantPayloadSchema {
    pub text: String,
    pub delta: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub replace: Option<WorkerLiveAssistantReplace>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub media_urls: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub phase: Option<WorkerLiveAssistantPhase>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub item_id: Option<String>,
}

impl WorkerLiveAssistantPayloadSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_string_length_range("text", &self.text, 0, WORKER_PROTOCOL_MAX_PAYLOAD_BYTES)?;
        validate_string_length_range("delta", &self.delta, 0, WORKER_PROTOCOL_MAX_PAYLOAD_BYTES)?;
        if let Some(urls) = &self.media_urls {
            if urls.len() > WORKER_TRANSCRIPT_MAX_CONTENT_PARTS {
                return Err(format!(
                    "mediaUrls: expected at most {} items, got {}",
                    WORKER_TRANSCRIPT_MAX_CONTENT_PARTS,
                    urls.len()
                ));
            }
            for (i, u) in urls.iter().enumerate() {
                validate_live_identifier(&format!("mediaUrls[{}]", i), u)?;
            }
        }
        if let Some(id) = &self.item_id {
            validate_worker_identifier("itemId", id)?;
        }
        Ok(())
    }
}

/// Thinking live event payload.
/// 对齐 TS: `workerLiveObject({ text, delta })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkerLiveThinkingPayloadSchema {
    pub text: String,
    pub delta: String,
}

impl WorkerLiveThinkingPayloadSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_string_length_range("text", &self.text, 0, WORKER_PROTOCOL_MAX_PAYLOAD_BYTES)?;
        validate_string_length_range("delta", &self.delta, 0, WORKER_PROTOCOL_MAX_PAYLOAD_BYTES)?;
        Ok(())
    }
}

// ----- Live identifier helper -----

/// Live identifier grammar (mirrors worker identifier but bound by payload size).
const LIVE_IDENTIFIER_PATTERN: &str = r"^\S(?:.*\S)?$";

fn is_valid_live_identifier(s: &str) -> bool {
    !s.is_empty()
        && s.chars().count() <= WORKER_PROTOCOL_MAX_PAYLOAD_BYTES
        && regex(LIVE_IDENTIFIER_PATTERN).is_match(s)
}

fn validate_live_identifier(field: &str, s: &str) -> Result<(), String> {
    if !is_valid_live_identifier(s) {
        return Err(format!(
            "{}: expected non-empty identifier up to {} chars matching {}, got {:?}",
            field, WORKER_PROTOCOL_MAX_PAYLOAD_BYTES, LIVE_IDENTIFIER_PATTERN, s
        ));
    }
    Ok(())
}

/// Literal marker for `WorkerLiveToolPayloadSchema.hideFromChannelProgress`.
/// 对齐 TS: `Type.Literal(true)`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum WorkerLiveToolHideFromChannelProgress {
    #[serde(rename = "true")]
    True,
}

impl WorkerLiveToolHideFromChannelProgress {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::True => "true",
        }
    }
}

/// Tool live event phase.
/// 对齐 TS: `Type.Union([Type.Literal("start"), Type.Literal("update"), Type.Literal("result")])`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum WorkerLiveToolPhase {
    Start,
    Update,
    Result,
}

impl WorkerLiveToolPhase {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Start => "start",
            Self::Update => "update",
            Self::Result => "result",
        }
    }
}

/// Tool live event — start variant.
/// 对齐 TS: `workerLiveObject({ name, toolCallId, hideFromChannelProgress?, phase: "start", args: Unknown })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkerLiveToolStartPayloadSchema {
    pub name: String,
    pub tool_call_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hide_from_channel_progress: Option<WorkerLiveToolHideFromChannelProgress>,
    pub phase: WorkerLiveToolPhaseStart,
    pub args: serde_json::Value,
}

/// Literal marker for tool start phase.
/// 对齐 TS: `Type.Literal("start")`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum WorkerLiveToolPhaseStart {
    #[serde(rename = "start")]
    Start,
}

impl WorkerLiveToolPhaseStart {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Start => "start",
        }
    }
}

impl WorkerLiveToolStartPayloadSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_worker_identifier("name", &self.name)?;
        validate_worker_identifier("toolCallId", &self.tool_call_id)?;
        Ok(())
    }
}

/// Tool live event — update variant.
/// 对齐 TS: `workerLiveObject({ name, toolCallId, hideFromChannelProgress?, phase: "update", partialResult: Unknown })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkerLiveToolUpdatePayloadSchema {
    pub name: String,
    pub tool_call_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hide_from_channel_progress: Option<WorkerLiveToolHideFromChannelProgress>,
    pub phase: WorkerLiveToolPhaseUpdate,
    pub partial_result: serde_json::Value,
}

/// Literal marker for tool update phase.
/// 对齐 TS: `Type.Literal("update")`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum WorkerLiveToolPhaseUpdate {
    #[serde(rename = "update")]
    Update,
}

impl WorkerLiveToolPhaseUpdate {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Update => "update",
        }
    }
}

impl WorkerLiveToolUpdatePayloadSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_worker_identifier("name", &self.name)?;
        validate_worker_identifier("toolCallId", &self.tool_call_id)?;
        Ok(())
    }
}

/// Tool live event — result variant.
/// 对齐 TS: `workerLiveObject({ name, toolCallId, hideFromChannelProgress?, phase: "result",
///                                meta?, isError, result, toolErrorSummary? })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkerLiveToolResultPayloadSchema {
    pub name: String,
    pub tool_call_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hide_from_channel_progress: Option<WorkerLiveToolHideFromChannelProgress>,
    pub phase: WorkerLiveToolPhaseResult,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub meta: Option<String>,
    pub is_error: bool,
    pub result: serde_json::Value,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool_error_summary: Option<String>,
}

/// Literal marker for tool result phase.
/// 对齐 TS: `Type.Literal("result")`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum WorkerLiveToolPhaseResult {
    #[serde(rename = "result")]
    Result,
}

impl WorkerLiveToolPhaseResult {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Result => "result",
        }
    }
}

impl WorkerLiveToolResultPayloadSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_worker_identifier("name", &self.name)?;
        validate_worker_identifier("toolCallId", &self.tool_call_id)?;
        if let Some(meta) = &self.meta {
            validate_string_length_range("meta", meta, 0, WORKER_PROTOCOL_MAX_PAYLOAD_BYTES)?;
        }
        if let Some(s) = &self.tool_error_summary {
            validate_string_length_range(
                "toolErrorSummary",
                s,
                0,
                WORKER_PROTOCOL_MAX_PAYLOAD_BYTES,
            )?;
        }
        Ok(())
    }
}

/// Tool live event payload union.
/// 对齐 TS: `Type.Union([start, update, result])`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum WorkerLiveToolPayloadSchema {
    Start(WorkerLiveToolStartPayloadSchema),
    Update(WorkerLiveToolUpdatePayloadSchema),
    Result(WorkerLiveToolResultPayloadSchema),
}

impl WorkerLiveToolPayloadSchema {
    pub fn validate(&self) -> Result<(), String> {
        match self {
            Self::Start(v) => v.validate(),
            Self::Update(v) => v.validate(),
            Self::Result(v) => v.validate(),
        }
    }
}

/// Approval live event common fields.
/// 对齐 TS: shared properties block (kind, title, itemId?, toolCallId?, approvalId?,
/// approvalSlug?, command?, host?, reason?, scope?, message?).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkerLiveApprovalCommonSchema {
    pub kind: WorkerLiveApprovalKind,
    pub title: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub item_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub approval_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub approval_slug: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub command: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub host: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scope: Option<WorkerLiveApprovalScope>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

/// Approval kind discriminator.
/// 对齐 TS: `Type.Union([Type.Literal("exec"), Type.Literal("plugin"), Type.Literal("unknown")])`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum WorkerLiveApprovalKind {
    #[default]
    Exec,
    Plugin,
    Unknown,
}

impl WorkerLiveApprovalKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Exec => "exec",
            Self::Plugin => "plugin",
            Self::Unknown => "unknown",
        }
    }
}

/// Approval scope discriminator.
/// 对齐 TS: `Type.Union([Type.Literal("turn"), Type.Literal("session")])`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum WorkerLiveApprovalScope {
    Turn,
    Session,
}

impl WorkerLiveApprovalScope {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Turn => "turn",
            Self::Session => "session",
        }
    }
}

impl WorkerLiveApprovalCommonSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_string_length_range("title", &self.title, 0, WORKER_PROTOCOL_MAX_PAYLOAD_BYTES)?;
        for (field, value) in [
            ("itemId", self.item_id.as_deref()),
            ("toolCallId", self.tool_call_id.as_deref()),
            ("approvalId", self.approval_id.as_deref()),
            ("approvalSlug", self.approval_slug.as_deref()),
        ] {
            if let Some(v) = value {
                validate_worker_identifier(field, v)?;
            }
        }
        for (field, value) in [
            ("command", self.command.as_deref()),
            ("host", self.host.as_deref()),
            ("reason", self.reason.as_deref()),
            ("message", self.message.as_deref()),
        ] {
            if let Some(v) = value {
                validate_string_length_range(field, v, 0, WORKER_PROTOCOL_MAX_PAYLOAD_BYTES)?;
            }
        }
        Ok(())
    }
}

/// Approval requested status.
/// 对齐 TS: `Type.Union([Type.Literal("pending"), Type.Literal("unavailable")])`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum WorkerLiveApprovalRequestedStatus {
    Pending,
    Unavailable,
}

impl WorkerLiveApprovalRequestedStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Unavailable => "unavailable",
        }
    }
}

/// Approval resolved status.
/// 对齐 TS: `Type.Union([Type.Literal("approved"), Type.Literal("denied"), Type.Literal("failed")])`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum WorkerLiveApprovalResolvedStatus {
    Approved,
    Denied,
    Failed,
}

impl WorkerLiveApprovalResolvedStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Approved => "approved",
            Self::Denied => "denied",
            Self::Failed => "failed",
        }
    }
}

/// Approval requested payload.
/// 对齐 TS: `workerLiveObject({...common, phase: "requested", status: pending|unavailable})`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkerLiveApprovalRequestedPayloadSchema {
    #[serde(flatten)]
    pub common: WorkerLiveApprovalCommonSchema,
    pub phase: WorkerLiveApprovalPhaseRequested,
    pub status: WorkerLiveApprovalRequestedStatus,
}

/// Literal marker for approval requested phase.
/// 对齐 TS: `Type.Literal("requested")`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum WorkerLiveApprovalPhaseRequested {
    #[serde(rename = "requested")]
    Requested,
}

impl WorkerLiveApprovalPhaseRequested {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
        }
    }
}

impl WorkerLiveApprovalRequestedPayloadSchema {
    pub fn validate(&self) -> Result<(), String> {
        self.common.validate().map_err(|e| format!("common: {}", e))?;
        Ok(())
    }
}

/// Approval resolved payload.
/// 对齐 TS: `workerLiveObject({...common, phase: "resolved", status: approved|denied|failed})`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkerLiveApprovalResolvedPayloadSchema {
    #[serde(flatten)]
    pub common: WorkerLiveApprovalCommonSchema,
    pub phase: WorkerLiveApprovalPhaseResolved,
    pub status: WorkerLiveApprovalResolvedStatus,
}

/// Literal marker for approval resolved phase.
/// 对齐 TS: `Type.Literal("resolved")`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum WorkerLiveApprovalPhaseResolved {
    #[serde(rename = "resolved")]
    Resolved,
}

impl WorkerLiveApprovalPhaseResolved {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Resolved => "resolved",
        }
    }
}

impl WorkerLiveApprovalResolvedPayloadSchema {
    pub fn validate(&self) -> Result<(), String> {
        self.common.validate().map_err(|e| format!("common: {}", e))?;
        Ok(())
    }
}

/// Approval live event payload union.
/// 对齐 TS: `Type.Union([requested, resolved])`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum WorkerLiveApprovalPayloadSchema {
    Requested(WorkerLiveApprovalRequestedPayloadSchema),
    Resolved(WorkerLiveApprovalResolvedPayloadSchema),
}

impl WorkerLiveApprovalPayloadSchema {
    pub fn validate(&self) -> Result<(), String> {
        match self {
            Self::Requested(v) => v.validate(),
            Self::Resolved(v) => v.validate(),
        }
    }
}

// ----- Lifecycle payloads -----

/// Lifecycle start phase.
/// 对齐 TS: `Type.Literal("start")`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum WorkerLiveLifecycleStartPhase {
    #[serde(rename = "start")]
    Start,
}

impl WorkerLiveLifecycleStartPhase {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Start => "start",
        }
    }
}

/// Lifecycle start payload.
/// 对齐 TS: `workerLiveObject({ phase: "start", startedAt: LiveIntegerSchema })`.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkerLiveLifecycleStartPayloadSchema {
    pub phase: WorkerLiveLifecycleStartPhase,
    pub started_at: i64,
}

impl WorkerLiveLifecycleStartPayloadSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_non_negative_integer("startedAt", self.started_at)?;
        Ok(())
    }
}

/// Live fallback reason.
/// 对齐 TS:
///   `Type.Union([
///      Type.Literal("auth"), Type.Literal("auth_permanent"), Type.Literal("format"),
///      Type.Literal("rate_limit"), Type.Literal("overloaded"), Type.Literal("billing"),
///      Type.Literal("server_error"), Type.Literal("timeout"),
///      Type.Literal("context_overflow"), Type.Literal("model_not_found"),
///      Type.Literal("session_expired"), Type.Literal("empty_response"),
///      Type.Literal("no_error_details"), Type.Literal("unclassified"),
///      Type.Literal("unknown"),
///   ])`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WorkerLiveFallbackReasonSchema {
    Auth,
    #[serde(rename = "auth_permanent")]
    AuthPermanent,
    Format,
    #[serde(rename = "rate_limit")]
    RateLimit,
    Overloaded,
    Billing,
    #[serde(rename = "server_error")]
    ServerError,
    Timeout,
    #[serde(rename = "context_overflow")]
    ContextOverflow,
    #[serde(rename = "model_not_found")]
    ModelNotFound,
    #[serde(rename = "session_expired")]
    SessionExpired,
    #[serde(rename = "empty_response")]
    EmptyResponse,
    #[serde(rename = "no_error_details")]
    NoErrorDetails,
    Unclassified,
    Unknown,
}

impl WorkerLiveFallbackReasonSchema {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Auth => "auth",
            Self::AuthPermanent => "auth_permanent",
            Self::Format => "format",
            Self::RateLimit => "rate_limit",
            Self::Overloaded => "overloaded",
            Self::Billing => "billing",
            Self::ServerError => "server_error",
            Self::Timeout => "timeout",
            Self::ContextOverflow => "context_overflow",
            Self::ModelNotFound => "model_not_found",
            Self::SessionExpired => "session_expired",
            Self::EmptyResponse => "empty_response",
            Self::NoErrorDetails => "no_error_details",
            Self::Unclassified => "unclassified",
            Self::Unknown => "unknown",
        }
    }
}

/// Live fallback attempt block.
/// 对齐 TS:
///   `workerLiveObject({
///      provider: LiveIdentifierSchema,
///      model: LiveIdentifierSchema,
///      error: LiveTextSchema,
///      reason: Type.Optional(WorkerLiveFallbackReasonSchema),
///      authMode: Type.Optional(LiveIdentifierSchema),
///      status: OptionalLiveIntegerSchema,
///      code: Type.Optional(Type.String({ minLength: 1, maxLength: WORKER_PROTOCOL_MAX_PAYLOAD_BYTES })),
///   })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkerLiveFallbackAttemptSchema {
    pub provider: String,
    pub model: String,
    pub error: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<WorkerLiveFallbackReasonSchema>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub auth_mode: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
}

impl WorkerLiveFallbackAttemptSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_live_identifier("provider", &self.provider)?;
        validate_live_identifier("model", &self.model)?;
        validate_string_length_range("error", &self.error, 0, WORKER_PROTOCOL_MAX_PAYLOAD_BYTES)?;
        if let Some(mode) = &self.auth_mode {
            validate_live_identifier("authMode", mode)?;
        }
        validate_optional_non_negative_integer("status", self.status)?;
        if let Some(code) = &self.code {
            validate_string_length_range(
                "code",
                code,
                1,
                WORKER_PROTOCOL_MAX_PAYLOAD_BYTES,
            )?;
        }
        Ok(())
    }
}

/// Lifecycle fallback payload.
/// 对齐 TS: `workerLiveObject({ selectedProvider, selectedModel, activeProvider, activeModel,
///                                phase: "fallback", reasonSummary, attemptSummaries, attempts })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkerLiveLifecycleFallbackPayloadSchema {
    pub selected_provider: String,
    pub selected_model: String,
    pub active_provider: String,
    pub active_model: String,
    pub phase: WorkerLiveLifecyclePhaseFallback,
    pub reason_summary: String,
    pub attempt_summaries: Vec<String>,
    pub attempts: Vec<WorkerLiveFallbackAttemptSchema>,
}

/// Literal marker for lifecycle fallback phase.
/// 对齐 TS: `Type.Literal("fallback")`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum WorkerLiveLifecyclePhaseFallback {
    #[serde(rename = "fallback")]
    Fallback,
}

impl WorkerLiveLifecyclePhaseFallback {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Fallback => "fallback",
        }
    }
}

impl WorkerLiveLifecycleFallbackPayloadSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_live_identifier("selectedProvider", &self.selected_provider)?;
        validate_live_identifier("selectedModel", &self.selected_model)?;
        validate_live_identifier("activeProvider", &self.active_provider)?;
        validate_live_identifier("activeModel", &self.active_model)?;
        validate_string_length_range(
            "reasonSummary",
            &self.reason_summary,
            0,
            WORKER_PROTOCOL_MAX_PAYLOAD_BYTES,
        )?;
        if self.attempt_summaries.len() > WORKER_TRANSCRIPT_MAX_CONTENT_PARTS {
            return Err(format!(
                "attemptSummaries: expected at most {} items, got {}",
                WORKER_TRANSCRIPT_MAX_CONTENT_PARTS,
                self.attempt_summaries.len()
            ));
        }
        for (i, s) in self.attempt_summaries.iter().enumerate() {
            validate_string_length_range(
                &format!("attemptSummaries[{}]", i),
                s,
                0,
                WORKER_PROTOCOL_MAX_PAYLOAD_BYTES,
            )?;
        }
        if self.attempts.len() > WORKER_TRANSCRIPT_MAX_CONTENT_PARTS {
            return Err(format!(
                "attempts: expected at most {} items, got {}",
                WORKER_TRANSCRIPT_MAX_CONTENT_PARTS,
                self.attempts.len()
            ));
        }
        for (i, a) in self.attempts.iter().enumerate() {
            a.validate().map_err(|e| format!("attempts[{}]: {}", i, e))?;
        }
        Ok(())
    }
}

/// Lifecycle fallback-cleared payload.
/// 对齐 TS: `workerLiveObject({ selectedProvider, selectedModel, activeProvider, activeModel,
///                                phase: "fallback_cleared", previousActiveModel? })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkerLiveLifecycleFallbackClearedPayloadSchema {
    pub selected_provider: String,
    pub selected_model: String,
    pub active_provider: String,
    pub active_model: String,
    pub phase: WorkerLiveLifecyclePhaseFallbackCleared,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub previous_active_model: Option<String>,
}

/// Literal marker for lifecycle fallback-cleared phase.
/// 对齐 TS: `Type.Literal("fallback_cleared")`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum WorkerLiveLifecyclePhaseFallbackCleared {
    #[serde(rename = "fallback_cleared")]
    FallbackCleared,
}

impl WorkerLiveLifecyclePhaseFallbackCleared {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::FallbackCleared => "fallback_cleared",
        }
    }
}

impl WorkerLiveLifecycleFallbackClearedPayloadSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_live_identifier("selectedProvider", &self.selected_provider)?;
        validate_live_identifier("selectedModel", &self.selected_model)?;
        validate_live_identifier("activeProvider", &self.active_provider)?;
        validate_live_identifier("activeModel", &self.active_model)?;
        if let Some(pam) = &self.previous_active_model {
            validate_live_identifier("previousActiveModel", pam)?;
        }
        Ok(())
    }
}

/// Fallback-step final outcome discriminator.
/// 对齐 TS: `Type.Union([next_fallback, succeeded, chain_exhausted])`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WorkerLiveLifecycleFallbackStepFinalOutcome {
    #[serde(rename = "next_fallback")]
    NextFallback,
    Succeeded,
    #[serde(rename = "chain_exhausted")]
    ChainExhausted,
}

impl WorkerLiveLifecycleFallbackStepFinalOutcome {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::NextFallback => "next_fallback",
            Self::Succeeded => "succeeded",
            Self::ChainExhausted => "chain_exhausted",
        }
    }
}

/// Lifecycle fallback-step payload.
/// 对齐 TS: `workerLiveObject({ phase: "fallback_step", fallbackStepType: "fallback_step",
///                                fallbackStepFromModel, fallbackStepToModel?,
///                                fallbackStepFromFailureReason?,
///                                fallbackStepFromFailureDetail?, fallbackStepChainPosition?,
///                                fallbackStepFinalOutcome })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkerLiveLifecycleFallbackStepPayloadSchema {
    pub phase: WorkerLiveLifecyclePhaseFallbackStep,
    pub fallback_step_type: WorkerLiveFallbackStepTypeFallbackStep,
    pub fallback_step_from_model: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fallback_step_to_model: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fallback_step_from_failure_reason: Option<WorkerLiveFallbackReasonSchema>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fallback_step_from_failure_detail: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fallback_step_chain_position: Option<i64>,
    pub fallback_step_final_outcome: WorkerLiveLifecycleFallbackStepFinalOutcome,
}

/// Literal marker for lifecycle fallback-step phase.
/// 对齐 TS: `Type.Literal("fallback_step")`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum WorkerLiveLifecyclePhaseFallbackStep {
    #[serde(rename = "fallback_step")]
    FallbackStep,
}

impl WorkerLiveLifecyclePhaseFallbackStep {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::FallbackStep => "fallback_step",
        }
    }
}

/// Literal marker for fallbackStepType.
/// 对齐 TS: `Type.Literal("fallback_step")`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum WorkerLiveFallbackStepTypeFallbackStep {
    #[serde(rename = "fallback_step")]
    FallbackStep,
}

impl WorkerLiveFallbackStepTypeFallbackStep {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::FallbackStep => "fallback_step",
        }
    }
}

impl WorkerLiveLifecycleFallbackStepPayloadSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_live_identifier(
            "fallbackStepFromModel",
            &self.fallback_step_from_model,
        )?;
        if let Some(m) = &self.fallback_step_to_model {
            validate_live_identifier("fallbackStepToModel", m)?;
        }
        if let Some(d) = &self.fallback_step_from_failure_detail {
            validate_string_length_range(
                "fallbackStepFromFailureDetail",
                d,
                0,
                WORKER_PROTOCOL_MAX_PAYLOAD_BYTES,
            )?;
        }
        validate_optional_non_negative_integer(
            "fallbackStepChainPosition",
            self.fallback_step_chain_position,
        )?;
        Ok(())
    }
}

/// Timeout phase discriminator.
/// 对齐 TS:
///   `Type.Union([
///      Type.Literal("queue"),
///      Type.Literal("preflight"),
///      Type.Literal("provider"),
///      Type.Literal("post_turn"),
///      Type.Literal("gateway_draining"),
///   ])`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WorkerLiveTimeoutPhase {
    Queue,
    Preflight,
    Provider,
    #[serde(rename = "post_turn")]
    PostTurn,
    #[serde(rename = "gateway_draining")]
    GatewayDraining,
}

impl WorkerLiveTimeoutPhase {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Queue => "queue",
            Self::Preflight => "preflight",
            Self::Provider => "provider",
            Self::PostTurn => "post_turn",
            Self::GatewayDraining => "gateway_draining",
        }
    }
}

/// Liveness state discriminator.
/// 对齐 TS:
///   `Type.Union([
///      Type.Literal("working"),
///      Type.Literal("paused"),
///      Type.Literal("blocked"),
///      Type.Literal("abandoned"),
///   ])`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum WorkerLiveLivenessState {
    Working,
    Paused,
    Blocked,
    Abandoned,
}

impl WorkerLiveLivenessState {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Working => "working",
            Self::Paused => "paused",
            Self::Blocked => "blocked",
            Self::Abandoned => "abandoned",
        }
    }
}

/// Lifecycle terminal common fields.
/// 对齐 TS: shared properties block (startedAt?, endedAt, stopReason?, yielded?, ...).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkerLiveLifecycleTerminalCommonSchema {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub started_at: Option<i64>,
    pub ended_at: i64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stop_reason: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub yielded: Option<WorkerLiveTerminalYielded>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub timeout_phase: Option<WorkerLiveTimeoutPhase>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provider_started: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub aborted: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool_error_summary: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub liveness_state: Option<WorkerLiveLivenessState>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub replay_invalid: Option<WorkerLiveTerminalReplayInvalid>,
}

/// Literal marker for terminal `yielded`.
/// 对齐 TS: `Type.Literal(true)`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum WorkerLiveTerminalYielded {
    #[serde(rename = "true")]
    True,
}

impl WorkerLiveTerminalYielded {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::True => "true",
        }
    }
}

/// Literal marker for terminal `replayInvalid`.
/// 对齐 TS: `Type.Literal(true)`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum WorkerLiveTerminalReplayInvalid {
    #[serde(rename = "true")]
    True,
}

impl WorkerLiveTerminalReplayInvalid {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::True => "true",
        }
    }
}

/// Literal marker for terminal fallback-exhausted.
/// 对齐 TS: `Type.Literal(true)`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum WorkerLiveTerminalFallbackExhausted {
    #[serde(rename = "true")]
    True,
}

impl WorkerLiveTerminalFallbackExhausted {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::True => "true",
        }
    }
}

impl WorkerLiveLifecycleTerminalCommonSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_optional_non_negative_integer("startedAt", self.started_at)?;
        validate_non_negative_integer("endedAt", self.ended_at)?;
        if let Some(sr) = &self.stop_reason {
            validate_worker_identifier("stopReason", sr)?;
        }
        if let Some(tes) = &self.tool_error_summary {
            validate_string_length_range(
                "toolErrorSummary",
                tes,
                0,
                WORKER_PROTOCOL_MAX_PAYLOAD_BYTES,
            )?;
        }
        Ok(())
    }
}

/// Terminal `finishing` payload.
/// 对齐 TS: `workerLiveObject({...common, phase: "finishing", error? })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkerLiveLifecycleFinishingPayloadSchema {
    #[serde(flatten)]
    pub common: WorkerLiveLifecycleTerminalCommonSchema,
    pub phase: WorkerLiveLifecyclePhaseFinishing,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// Literal marker for lifecycle finishing phase.
/// 对齐 TS: `Type.Literal("finishing")`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum WorkerLiveLifecyclePhaseFinishing {
    #[serde(rename = "finishing")]
    Finishing,
}

impl WorkerLiveLifecyclePhaseFinishing {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Finishing => "finishing",
        }
    }
}

impl WorkerLiveLifecycleFinishingPayloadSchema {
    pub fn validate(&self) -> Result<(), String> {
        self.common.validate().map_err(|e| format!("common: {}", e))?;
        if let Some(e) = &self.error {
            validate_string_length_range("error", e, 0, WORKER_PROTOCOL_MAX_PAYLOAD_BYTES)?;
        }
        Ok(())
    }
}

/// Terminal `end` payload.
/// 对齐 TS: `workerLiveObject({...common, phase: "end" })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkerLiveLifecycleEndPayloadSchema {
    #[serde(flatten)]
    pub common: WorkerLiveLifecycleTerminalCommonSchema,
    pub phase: WorkerLiveLifecyclePhaseEnd,
}

/// Literal marker for lifecycle end phase.
/// 对齐 TS: `Type.Literal("end")`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum WorkerLiveLifecyclePhaseEnd {
    #[serde(rename = "end")]
    End,
}

impl WorkerLiveLifecyclePhaseEnd {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::End => "end",
        }
    }
}

impl WorkerLiveLifecycleEndPayloadSchema {
    pub fn validate(&self) -> Result<(), String> {
        self.common.validate().map_err(|e| format!("common: {}", e))?;
        Ok(())
    }
}

/// Terminal `error` payload.
/// 对齐 TS: `workerLiveObject({...common, phase: "error", error: LiveTextSchema,
///                                fallbackExhaustedFailure? })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkerLiveLifecycleErrorPayloadSchema {
    #[serde(flatten)]
    pub common: WorkerLiveLifecycleTerminalCommonSchema,
    pub phase: WorkerLiveLifecyclePhaseError,
    pub error: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fallback_exhausted_failure: Option<WorkerLiveTerminalFallbackExhausted>,
}

/// Literal marker for lifecycle error phase.
/// 对齐 TS: `Type.Literal("error")`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum WorkerLiveLifecyclePhaseError {
    #[serde(rename = "error")]
    Error,
}

impl WorkerLiveLifecyclePhaseError {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Error => "error",
        }
    }
}

impl WorkerLiveLifecycleErrorPayloadSchema {
    pub fn validate(&self) -> Result<(), String> {
        self.common.validate().map_err(|e| format!("common: {}", e))?;
        validate_string_length_range("error", &self.error, 0, WORKER_PROTOCOL_MAX_PAYLOAD_BYTES)?;
        Ok(())
    }
}

/// Lifecycle payload union.
/// 对齐 TS: `Type.Union([start, fallback, fallback_cleared, fallback_step,
///                       finishing, end, error])`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum WorkerLiveLifecyclePayloadSchema {
    Start(WorkerLiveLifecycleStartPayloadSchema),
    Fallback(WorkerLiveLifecycleFallbackPayloadSchema),
    FallbackCleared(WorkerLiveLifecycleFallbackClearedPayloadSchema),
    FallbackStep(WorkerLiveLifecycleFallbackStepPayloadSchema),
    Finishing(WorkerLiveLifecycleFinishingPayloadSchema),
    End(WorkerLiveLifecycleEndPayloadSchema),
    Error(WorkerLiveLifecycleErrorPayloadSchema),
}

impl WorkerLiveLifecyclePayloadSchema {
    pub fn validate(&self) -> Result<(), String> {
        match self {
            Self::Start(v) => v.validate(),
            Self::Fallback(v) => v.validate(),
            Self::FallbackCleared(v) => v.validate(),
            Self::FallbackStep(v) => v.validate(),
            Self::Finishing(v) => v.validate(),
            Self::End(v) => v.validate(),
            Self::Error(v) => v.validate(),
        }
    }
}

// ============================================================================
// Worker live event envelope
// ============================================================================

/// Live event envelope (5 variants by `kind`).
/// 对齐 TS:
///   `Type.Union([
///      workerLiveObject({ kind: "assistant", payload: WorkerLiveAssistantPayloadSchema }),
///      workerLiveObject({ kind: "thinking", payload: WorkerLiveThinkingPayloadSchema }),
///      workerLiveObject({ kind: "tool", payload: WorkerLiveToolPayloadSchema }),
///      workerLiveObject({ kind: "approval", payload: WorkerLiveApprovalPayloadSchema }),
///      workerLiveObject({ kind: "lifecycle", payload: WorkerLiveLifecyclePayloadSchema }),
///   ])`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum WorkerLiveEventSchema {
    Assistant {
        payload: WorkerLiveAssistantPayloadSchema,
    },
    Thinking {
        payload: WorkerLiveThinkingPayloadSchema,
    },
    Tool {
        payload: WorkerLiveToolPayloadSchema,
    },
    Approval {
        payload: WorkerLiveApprovalPayloadSchema,
    },
    Lifecycle {
        payload: WorkerLiveLifecyclePayloadSchema,
    },
}

impl WorkerLiveEventSchema {
    pub fn validate(&self) -> Result<(), String> {
        match self {
            Self::Assistant { payload } => payload.validate(),
            Self::Thinking { payload } => payload.validate(),
            Self::Tool { payload } => payload.validate(),
            Self::Approval { payload } => payload.validate(),
            Self::Lifecycle { payload } => payload.validate(),
        }
    }
}

/// Live event params.
/// 对齐 TS:
///   `workerLiveObject({
///      runEpoch: LiveIntegerSchema,
///      lastAckedSeq: LiveIntegerSchema,
///      seq: LiveSequenceSchema,
///      runId: WorkerIdentifierSchema,
///      event: WorkerLiveEventSchema,
///   })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkerLiveEventParamsSchema {
    pub run_epoch: i64,
    pub last_acked_seq: i64,
    pub seq: i64,
    pub run_id: String,
    pub event: WorkerLiveEventSchema,
}

impl WorkerLiveEventParamsSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_non_negative_integer("runEpoch", self.run_epoch)?;
        validate_non_negative_integer("lastAckedSeq", self.last_acked_seq)?;
        validate_positive_integer("seq", self.seq)?;
        validate_worker_identifier("runId", &self.run_id)?;
        self.event.validate().map_err(|e| format!("event: {}", e))?;
        Ok(())
    }
}

/// Live event ack result.
/// 对齐 TS: `workerLiveObject({ ackedSeq: LiveIntegerSchema })`.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkerLiveEventResultSchema {
    pub acked_seq: i64,
}

impl WorkerLiveEventResultSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_non_negative_integer("ackedSeq", self.acked_seq)?;
        Ok(())
    }
}

/// Live event error reason.
/// 对齐 TS:
///   `Type.Union([
///      Type.Literal("epoch-mismatch"),
///      Type.Literal("session-not-attached"),
///      Type.Literal("invalid-event"),
///      Type.Literal("capacity-exceeded"),
///      Type.Literal("resync-required"),
///   ])`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum WorkerLiveEventErrorReasonSchema {
    #[serde(rename = "epoch-mismatch")]
    EpochMismatch,
    #[serde(rename = "session-not-attached")]
    SessionNotAttached,
    #[serde(rename = "invalid-event")]
    InvalidEvent,
    #[serde(rename = "capacity-exceeded")]
    CapacityExceeded,
    #[serde(rename = "resync-required")]
    ResyncRequired,
}

impl WorkerLiveEventErrorReasonSchema {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::EpochMismatch => "epoch-mismatch",
            Self::SessionNotAttached => "session-not-attached",
            Self::InvalidEvent => "invalid-event",
            Self::CapacityExceeded => "capacity-exceeded",
            Self::ResyncRequired => "resync-required",
        }
    }
}

/// Live event error details — discriminated by reason shape.
/// 对齐 TS: `Type.Union([
///   workerLiveObject({ reason: Type.Union([epoch, session, invalid, capacity]) }),
///   workerLiveObject({ reason: Type.Literal("resync-required"), ackedSeq, expectedSeq }),
/// ])`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum WorkerLiveEventErrorDetailsSchema {
    Simple {
        reason: WorkerLiveEventErrorReasonSchema,
    },
    Resync {
        reason: WorkerLiveEventErrorReasonResync,
        acked_seq: i64,
        expected_seq: i64,
    },
}

/// Literal marker for resync reason.
/// 对齐 TS: `Type.Literal("resync-required")`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum WorkerLiveEventErrorReasonResync {
    #[serde(rename = "resync-required")]
    ResyncRequired,
}

impl WorkerLiveEventErrorReasonResync {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::ResyncRequired => "resync-required",
        }
    }
}

impl WorkerLiveEventErrorDetailsSchema {
    pub fn validate(&self) -> Result<(), String> {
        match self {
            Self::Simple { .. } => Ok(()),
            Self::Resync {
                acked_seq,
                expected_seq,
                ..
            } => {
                validate_non_negative_integer("ackedSeq", *acked_seq)?;
                validate_positive_integer("expectedSeq", *expected_seq)?;
                Ok(())
            }
        }
    }
}

/// Live event error shape.
/// 对齐 TS:
///   `workerLiveObject({
///      code: Type.Literal("INVALID_REQUEST"),
///      message: Type.String({ minLength: 1, maxLength: 256 }),
///      details: WorkerLiveEventErrorDetailsSchema,
///   })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkerLiveEventErrorShapeSchema {
    pub code: WorkerErrorCodeSchema,
    pub message: String,
    pub details: WorkerLiveEventErrorDetailsSchema,
}

impl WorkerLiveEventErrorShapeSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_string_length_range("message", &self.message, 1, 256)?;
        self.details
            .validate()
            .map_err(|e| format!("details: {}", e))?;
        Ok(())
    }
}

/// Live event request frame.
/// 对齐 TS:
///   `workerLiveObject({
///      type: Type.Literal("req"),
///      id: WorkerFrameIdSchema,
///      method: Type.Literal("worker.live-event"),
///      params: WorkerLiveEventParamsSchema,
///   })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkerLiveEventRequestFrameSchema {
    #[serde(rename = "type")]
    pub frame_type: WorkerFrameTypeReq,
    pub id: String,
    pub method: String,
    pub params: WorkerLiveEventParamsSchema,
}

impl WorkerLiveEventRequestFrameSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_worker_frame_id("id", &self.id)?;
        if self.method != WORKER_PROTOCOL_METHODS[2] {
            return Err(format!(
                "method: expected literal {:?}, got {:?}",
                WORKER_PROTOCOL_METHODS[2],
                self.method
            ));
        }
        self.params.validate().map_err(|e| format!("params: {}", e))?;
        Ok(())
    }
}

/// Live event success response frame.
/// 对齐 TS:
///   `workerLiveObject({
///      type: Type.Literal("res"),
///      id: WorkerFrameIdSchema,
///      ok: Type.Literal(true),
///      payload: WorkerLiveEventResultSchema,
///   })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkerLiveEventSuccessResponseFrameSchema {
    #[serde(rename = "type")]
    pub frame_type: WorkerFrameTypeRes,
    pub id: String,
    pub ok: bool,
    pub payload: WorkerLiveEventResultSchema,
}

impl WorkerLiveEventSuccessResponseFrameSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_worker_frame_id("id", &self.id)?;
        if !self.ok {
            return Err("ok: expected literal true".to_string());
        }
        self.payload
            .validate()
            .map_err(|e| format!("payload: {}", e))?;
        Ok(())
    }
}

/// Live event error response frame.
/// 对齐 TS:
///   `workerLiveObject({
///      type: Type.Literal("res"),
///      id: WorkerFrameIdSchema,
///      ok: Type.Literal(false),
///      error: WorkerLiveEventErrorShapeSchema,
///   })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkerLiveEventErrorResponseFrameSchema {
    #[serde(rename = "type")]
    pub frame_type: WorkerLiveTypeResGeneric,
    pub id: String,
    pub ok: bool,
    pub error: WorkerLiveEventErrorShapeSchema,
}

/// Literal marker used for `WorkerLiveEventErrorResponseFrameSchema` so its
/// `type` field accepts the same `res` literal (kept distinct to satisfy the
/// Rust type system — both translate to the wire string `"res"`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum WorkerLiveTypeResGeneric {
    #[serde(rename = "res")]
    Res,
}

impl WorkerLiveTypeResGeneric {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Res => "res",
        }
    }
}

impl WorkerLiveEventErrorResponseFrameSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_worker_frame_id("id", &self.id)?;
        if self.ok {
            return Err("ok: expected literal false".to_string());
        }
        self.error.validate().map_err(|e| format!("error: {}", e))?;
        Ok(())
    }
}

/// Live event response frame.
/// 对齐 TS: `Type.Union([success, error, genericError])`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum WorkerLiveEventResponseFrameSchema {
    Success(WorkerLiveEventSuccessResponseFrameSchema),
    EventError(WorkerLiveEventErrorResponseFrameSchema),
    Error(WorkerErrorResponseFrameSchema),
}

impl WorkerLiveEventResponseFrameSchema {
    pub fn validate(&self) -> Result<(), String> {
        match self {
            Self::Success(s) => s.validate(),
            Self::EventError(e) => e.validate(),
            Self::Error(e) => e.validate(),
        }
    }
}