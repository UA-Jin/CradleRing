// Gateway Protocol schema: environments.
// 翻译自 packages/gateway-protocol/src/schema/environments.ts
//
// TS 用 TypeBox 定义 schema（运行时验证 + 类型）。
// Rust 用 serde struct + 验证函数实现等价的序列化/反序列化语义。
//
// Environment inventory protocol schemas.
// Environments are runtime targets such as local hosts, VMs, or remote workers;
// this schema layer only describes their gateway-visible status summary.

use serde::{Deserialize, Serialize};

// ---------- 基础验证原语 (对齐 TypeBox: NonEmptyString) ----------

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

// ---------- EnvironmentStatusSchema ----------
// 对齐 TS:
//   export const EnvironmentStatusSchema = Type.String({
//     enum: ["available", "unavailable", "starting", "stopping", "error"],
//   });

/// Runtime availability state for an environment target.
/// 对齐 TS: `EnvironmentStatusSchema = Type.String({ enum: [...] })`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EnvironmentStatus {
    Available,
    Unavailable,
    Starting,
    Stopping,
    Error,
}

impl EnvironmentStatus {
    pub fn validate(&self) -> Result<(), String> {
        // Closed enum; serde rejects unknown values at deserialization time.
        Ok(())
    }
}

// ---------- WorkerEnvironmentStateSchema ----------
// 对齐 TS:
//   Type.Union([
//     Type.Literal("requested"), Type.Literal("provisioning"),
//     Type.Literal("bootstrapping"), Type.Literal("ready"),
//     Type.Literal("attached"), Type.Literal("idle"),
//     Type.Literal("draining"), Type.Literal("destroying"),
//     Type.Literal("destroyed"), Type.Literal("failed"),
//     Type.Literal("orphaned"),
//   ])

/// Durable lifecycle states for plugin-provisioned worker environments.
/// 对齐 TS: `WorkerEnvironmentStateSchema = Type.Union([...literals...])`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum WorkerEnvironmentState {
    Requested,
    Provisioning,
    Bootstrapping,
    Ready,
    Attached,
    Idle,
    Draining,
    Destroying,
    Destroyed,
    Failed,
    Orphaned,
}

impl WorkerEnvironmentState {
    pub fn validate(&self) -> Result<(), String> {
        Ok(())
    }
}

// ---------- WorkerTunnelStatusSchema ----------
// 对齐 TS:
//   Type.Union([
//     Type.Literal("stopped"), Type.Literal("connecting"),
//     Type.Literal("connected"), Type.Literal("reconnecting"),
//   ])

/// Process-local SSH tunnel connectivity for a worker environment.
/// 对齐 TS: `WorkerTunnelStatusSchema = Type.Union([...literals...])`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum WorkerTunnelStatus {
    Stopped,
    Connecting,
    Connected,
    Reconnecting,
}

impl WorkerTunnelStatus {
    pub fn validate(&self) -> Result<(), String> {
        Ok(())
    }
}

// ---------- WorkerEnvironmentMetadataSchema ----------
// 对齐 TS:
//   Type.Object({
//     providerId: NonEmptyString,
//     leaseId: Type.Optional(NonEmptyString),
//     state: WorkerEnvironmentStateSchema,
//     ageMs: Type.Integer({ minimum: 0 }),
//     idleMs: Type.Optional(Type.Integer({ minimum: 0 })),
//     attachedSessionIds: Type.Array(NonEmptyString),
//     tunnelStatus: WorkerTunnelStatusSchema,
//   }, { additionalProperties: false })

/// Worker-only lifecycle metadata layered onto the existing environment projection.
/// 对齐 TS: `WorkerEnvironmentMetadataSchema`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkerEnvironmentMetadata {
    pub provider_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lease_id: Option<String>,
    pub state: WorkerEnvironmentState,
    pub age_ms: i64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub idle_ms: Option<i64>,
    pub attached_session_ids: Vec<String>,
    pub tunnel_status: WorkerTunnelStatus,
}

impl WorkerEnvironmentMetadata {
    pub fn validate(&self) -> Result<(), String> {
        validate_non_empty_string("providerId", &self.provider_id)?;
        validate_optional_non_empty_string("leaseId", self.lease_id.as_deref())?;
        self.state.validate()?;
        if self.age_ms < 0 {
            return Err(format!("ageMs: expected non-negative integer, got {}", self.age_ms));
        }
        if let Some(idle_ms) = self.idle_ms {
            if idle_ms < 0 {
                return Err(format!("idleMs: expected non-negative integer, got {}", idle_ms));
            }
        }
        validate_non_empty_string_list("attachedSessionIds", &self.attached_session_ids)?;
        self.tunnel_status.validate()?;
        Ok(())
    }
}

// ---------- createEnvironmentSummarySchema ----------
// 对齐 TS:
//   function createEnvironmentSummarySchema() {
//     return Type.Object({
//       id: NonEmptyString,
//       type: NonEmptyString,
//       label: Type.Optional(NonEmptyString),
//       status: EnvironmentStatusSchema,
//       capabilities: Type.Optional(Type.Array(NonEmptyString)),
//       worker: Type.Optional(WorkerEnvironmentMetadataSchema),
//     }, { additionalProperties: false });
//   }

/// Public environment summary shown in listings and status responses.
/// 对齐 TS: `EnvironmentSummarySchema = createEnvironmentSummarySchema()`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EnvironmentSummary {
    pub id: String,
    #[serde(rename = "type")]
    pub environment_type: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    pub status: EnvironmentStatus,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub capabilities: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub worker: Option<WorkerEnvironmentMetadata>,
}

impl EnvironmentSummary {
    pub fn validate(&self) -> Result<(), String> {
        validate_non_empty_string("id", &self.id)?;
        validate_non_empty_string("type", &self.environment_type)?;
        validate_optional_non_empty_string("label", self.label.as_deref())?;
        self.status.validate()?;
        validate_optional_non_empty_string_list("capabilities", self.capabilities.as_ref())?;
        if let Some(worker) = &self.worker {
            worker.validate()?;
        }
        Ok(())
    }
}

// ---------- EnvironmentsListParamsSchema ----------
// 对齐 TS: `Type.Object({}, { additionalProperties: false })`

/// Empty request payload for listing known environments.
/// 对齐 TS: `EnvironmentsListParamsSchema = Type.Object({}, { additionalProperties: false })`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EnvironmentsListParams {}

impl EnvironmentsListParams {
    pub fn validate(&self) -> Result<(), String> {
        // No required/constrained fields; the empty schema always validates.
        Ok(())
    }
}

// ---------- EnvironmentsListResultSchema ----------
// 对齐 TS:
//   Type.Object({
//     environments: Type.Array(EnvironmentSummarySchema),
//   }, { additionalProperties: false })

/// List response containing all gateway-visible environment summaries.
/// 对齐 TS: `EnvironmentsListResultSchema`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EnvironmentsListResult {
    pub environments: Vec<EnvironmentSummary>,
}

impl EnvironmentsListResult {
    pub fn validate(&self) -> Result<(), String> {
        for (i, env) in self.environments.iter().enumerate() {
            env.validate().map_err(|e| format!("environments[{}]: {}", i, e))?;
        }
        Ok(())
    }
}

// ---------- EnvironmentsStatusParamsSchema ----------
// 对齐 TS:
//   Type.Object({ environmentId: NonEmptyString }, { additionalProperties: false })

/// Status lookup request for one environment id.
/// 对齐 TS: `EnvironmentsStatusParamsSchema`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EnvironmentsStatusParams {
    pub environment_id: String,
}

impl EnvironmentsStatusParams {
    pub fn validate(&self) -> Result<(), String> {
        validate_non_empty_string("environmentId", &self.environment_id)?;
        Ok(())
    }
}

/// Status lookup result for one environment id.
/// 对齐 TS: `EnvironmentsStatusResultSchema = createEnvironmentSummarySchema()`.
pub type EnvironmentsStatusResult = EnvironmentSummary;

// ---------- EnvironmentsCreateParamsSchema ----------
// 对齐 TS:
//   Type.Object({
//     profileId: NonEmptyString,
//     idempotencyKey: NonEmptyString,
//   }, { additionalProperties: false })

/// Creates a worker environment from one configured provider profile.
/// 对齐 TS: `EnvironmentsCreateParamsSchema`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EnvironmentsCreateParams {
    pub profile_id: String,
    pub idempotency_key: String,
}

impl EnvironmentsCreateParams {
    pub fn validate(&self) -> Result<(), String> {
        validate_non_empty_string("profileId", &self.profile_id)?;
        validate_non_empty_string("idempotencyKey", &self.idempotency_key)?;
        Ok(())
    }
}

/// Create result uses the same public summary shape as list and status.
/// 对齐 TS: `EnvironmentsCreateResultSchema = createEnvironmentSummarySchema()`.
pub type EnvironmentsCreateResult = EnvironmentSummary;

// ---------- EnvironmentsDestroyParamsSchema ----------
// 对齐 TS:
//   Type.Object({ environmentId: NonEmptyString }, { additionalProperties: false })

/// Destroys one durable worker environment by its gateway-owned id.
/// 对齐 TS: `EnvironmentsDestroyParamsSchema`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EnvironmentsDestroyParams {
    pub environment_id: String,
}

impl EnvironmentsDestroyParams {
    pub fn validate(&self) -> Result<(), String> {
        validate_non_empty_string("environmentId", &self.environment_id)?;
        Ok(())
    }
}

/// Destroy result exposes the terminal worker lifecycle state.
/// 对齐 TS: `EnvironmentsDestroyResultSchema = createEnvironmentSummarySchema()`.
pub type EnvironmentsDestroyResult = EnvironmentSummary;

// Wire type aliases (对标 TS `type X = Static<typeof YSchema>`)
pub type EnvironmentStatusType = EnvironmentStatus;
pub type WorkerEnvironmentStateType = WorkerEnvironmentState;
pub type WorkerTunnelStatusType = WorkerTunnelStatus;
pub type WorkerEnvironmentMetadataType = WorkerEnvironmentMetadata;
pub type EnvironmentSummaryType = EnvironmentSummary;
pub type EnvironmentsListParamsType = EnvironmentsListParams;
pub type EnvironmentsListResultType = EnvironmentsListResult;
pub type EnvironmentsStatusParamsType = EnvironmentsStatusParams;
pub type EnvironmentsCreateParamsType = EnvironmentsCreateParams;
pub type EnvironmentsDestroyParamsType = EnvironmentsDestroyParams;