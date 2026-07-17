// Gateway Protocol schema: cron.
// 翻译自 packages/gateway-protocol/src/schema/cron.ts
//
// Cron scheduler protocol schemas.
// These contracts describe scheduled agent turns, system events, delivery
// routing, run history, and mutable job state shared by gateway RPC clients.
//
// TS 用 TypeBox 定义 schema（运行时验证 + 类型）。
// Rust 用 serde struct + 验证函数实现等价的序列化/反序列化语义。

use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::primitives::NonEmptyString;

// ===========================================================================
// Module-private bounds and patterns
// ===========================================================================

/// 对齐 TS: `Type.String({ minLength: 1, maxLength: 128 })`.
pub const CRON_CONFIG_REVISION_MAX_LENGTH: usize = 128;
pub const CRON_CONFIG_REVISION_MIN_LENGTH: usize = 1;

/// 对齐 TS: `Type.String({ minLength: 1, maxLength: 200, pattern: "\\S" })`.
pub const CRON_DECLARATION_KEY_MAX_LENGTH: usize = 200;
pub const CRON_DECLARATION_KEY_MIN_LENGTH: usize = 1;
const CRON_DECLARATION_KEY_PATTERN: &str = r"\S";

pub const CRON_DISPLAY_NAME_MAX_LENGTH: usize = 200;
pub const CRON_DISPLAY_NAME_MIN_LENGTH: usize = 1;
const CRON_DISPLAY_NAME_PATTERN: &str = r"\S";

/// 对齐 TS: `Type.String({ minLength: 1, maxLength: 65_536 })`.
pub const CRON_TRIGGER_SCRIPT_MAX_LENGTH: usize = 65_536;
pub const CRON_TRIGGER_SCRIPT_MIN_LENGTH: usize = 1;

/// 对齐 TS: `Type.String({ minLength: 1, maxLength: 1024 })`.
pub const CRON_DESCRIPTION_MAX_LENGTH: usize = 1024;

/// 对齐 TS: `Type.String({ minLength: 1, maxLength: 2048 })`.
pub const CRON_TEXT_MAX_LENGTH: usize = 2048;

/// 对齐 TS: `Type.String({ minLength: 1, pattern: "\\S" })` (NonBlankString).
pub const NON_BLANK_STRING_PATTERN: &str = r"\S";
pub const NON_BLANK_STRING_MIN_LENGTH: usize = 1;

/// 对齐 TS: `Type.Integer({ minimum: 1, maximum: 200 })`.
pub const CRON_LIST_LIMIT_MAX: i64 = 200;
pub const CRON_LIST_LIMIT_MIN: i64 = 1;

/// 对齐 TS: `Type.Integer({ minimum: 0 })`.
pub const CRON_LIST_OFFSET_MIN: i64 = 0;

/// 对齐 TS: `Type.Array(... statuses, { minItems: 1, maxItems: 3 })`.
pub const CRON_RUNS_STATUS_FILTER_MAX: usize = 3;
pub const CRON_RUNS_STATUS_FILTER_MIN: usize = 1;

/// 对齐 TS: `Type.Array(... statuses, { minItems: 1, maxItems: 4 })`.
pub const CRON_DELIVERY_STATUS_FILTER_MAX: usize = 4;
pub const CRON_DELIVERY_STATUS_FILTER_MIN: usize = 1;

/// 对齐 TS: `Type.Array(NonEmptyString, { maxItems: 256 })` (replacePaths).
pub const CRON_REPLACE_PATHS_MAX_ITEMS: usize = 256;

/// 对齐 TS: `Type.String({ minLength: 1, pattern: "^[^/\\\\]+$" })`.
pub const CRON_RUN_LOG_JOB_ID_PATTERN: &str = r"^[^/\\]+$";
pub const CRON_RUN_LOG_JOB_ID_MIN_LENGTH: usize = 1;

// ===========================================================================
// Validation primitives
// ===========================================================================

fn regex(pattern: &str) -> regex::Regex {
    regex::Regex::new(pattern).expect("invalid regex pattern compiled into cron")
}

fn validate_non_empty_string(field: &str, value: &str) -> Result<(), String> {
    if value.is_empty() {
        return Err(format!(
            "{}: expected non-empty string, got {:?}",
            field, value
        ));
    }
    Ok(())
}

fn validate_non_blank_string(field: &str, value: &str) -> Result<(), String> {
    if value.len() < NON_BLANK_STRING_MIN_LENGTH {
        return Err(format!(
            "{}: expected non-blank string (>= 1 char), got {:?}",
            field, value
        ));
    }
    if !regex(NON_BLANK_STRING_PATTERN).is_match(value) {
        return Err(format!(
            "{}: expected string matching {:?}, got {:?}",
            field, NON_BLANK_STRING_PATTERN, value
        ));
    }
    Ok(())
}

fn validate_bounded_string(
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

fn validate_non_negative_integer(field: &str, value: i64) -> Result<(), String> {
    if value < 0 {
        return Err(format!("{}: expected >= 0, got {}", field, value));
    }
    Ok(())
}

/// Returns true when `key` matches the cron declaration-key grammar.
pub fn is_valid_cron_declaration_key(key: &str) -> bool {
    let len = key.chars().count();
    if len < CRON_DECLARATION_KEY_MIN_LENGTH || len > CRON_DECLARATION_KEY_MAX_LENGTH {
        return false;
    }
    regex(CRON_DECLARATION_KEY_PATTERN).is_match(key)
}

/// Returns true when `name` matches the cron display-name grammar.
pub fn is_valid_cron_display_name(name: &str) -> bool {
    let len = name.chars().count();
    if len < CRON_DISPLAY_NAME_MIN_LENGTH || len > CRON_DISPLAY_NAME_MAX_LENGTH {
        return false;
    }
    regex(CRON_DISPLAY_NAME_PATTERN).is_match(name)
}

/// Returns true when `id` matches the cron run-log job id grammar.
pub fn is_valid_cron_run_log_job_id(id: &str) -> bool {
    if id.chars().count() < CRON_RUN_LOG_JOB_ID_MIN_LENGTH {
        return false;
    }
    regex(CRON_RUN_LOG_JOB_ID_PATTERN).is_match(id)
}

// ===========================================================================
// CronSessionTargetSchema
//   Union: "main" | "isolated" | "current" | "session:<anything>"
// ===========================================================================

/// Session target accepted by cron jobs.
/// 对齐 TS: `Type.Union([Type.Literal("main"), Type.Literal("isolated"),
///                      Type.Literal("current"), Type.String({ pattern: "^session:.+" })])`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum CronSessionTargetSchema {
    Main,
    Isolated,
    Current,
    /// Matches the `session:<anything>` pattern.
    Other(String),
}

impl CronSessionTargetSchema {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Main => "main",
            Self::Isolated => "isolated",
            Self::Current => "current",
            Self::Other(s) => s.as_str(),
        }
    }
}

// ===========================================================================
// CronWakeModeSchema
// ===========================================================================

/// Whether a cron job waits for heartbeat processing or wakes immediately.
/// 对齐 TS: `Type.Union([Type.Literal("next-heartbeat"), Type.Literal("now")])`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CronWakeModeSchema {
    #[serde(rename = "next-heartbeat")]
    NextHeartbeat,
    #[serde(rename = "now")]
    Now,
}

impl CronWakeModeSchema {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::NextHeartbeat => "next-heartbeat",
            Self::Now => "now",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "next-heartbeat" => Some(Self::NextHeartbeat),
            "now" => Some(Self::Now),
            _ => None,
        }
    }
}

pub fn is_valid_cron_wake_mode(s: &str) -> bool {
    CronWakeModeSchema::from_str(s).is_some()
}

// ===========================================================================
// CronRunStatusSchema
// ===========================================================================

/// Run status factory reused for the active field and deprecated alias
/// metadata. The deprecated flag is a TS-only annotation and does not affect
/// runtime semantics.
/// 对齐 TS: `Type.Union([Type.Literal("ok"), Type.Literal("error"), Type.Literal("skipped")])`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CronRunStatusSchema {
    #[serde(rename = "ok")]
    Ok,
    #[serde(rename = "error")]
    Error,
    #[serde(rename = "skipped")]
    Skipped,
}

impl CronRunStatusSchema {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Ok => "ok",
            Self::Error => "error",
            Self::Skipped => "skipped",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "ok" => Some(Self::Ok),
            "error" => Some(Self::Error),
            "skipped" => Some(Self::Skipped),
            _ => None,
        }
    }
}

pub fn is_valid_cron_run_status(s: &str) -> bool {
    CronRunStatusSchema::from_str(s).is_some()
}

// ===========================================================================
// CronConfigRevisionSchema / DeprecatedCronRunStatusSchema aliases
// ===========================================================================

/// 对齐 TS: `Type.String({ minLength: 1, maxLength: 128 })`.
pub type CronConfigRevisionSchema = String;

/// Returns true when `rev` fits the cron config-revision bounds.
pub fn is_valid_cron_config_revision(rev: &str) -> bool {
    let len = rev.chars().count();
    len >= CRON_CONFIG_REVISION_MIN_LENGTH && len <= CRON_CONFIG_REVISION_MAX_LENGTH
}

/// Type alias for the deprecated `lastRunStatus` field. The runtime type is
/// identical to `CronRunStatusSchema`; only the TS-side annotation differs.
pub type DeprecatedCronRunStatusSchema = CronRunStatusSchema;

// ===========================================================================
// CronSortDirSchema
// ===========================================================================

/// 对齐 TS: `Type.Union([Type.Literal("asc"), Type.Literal("desc")])`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CronSortDirSchema {
    #[serde(rename = "asc")]
    Asc,
    #[serde(rename = "desc")]
    Desc,
}

impl CronSortDirSchema {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Asc => "asc",
            Self::Desc => "desc",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "asc" => Some(Self::Asc),
            "desc" => Some(Self::Desc),
            _ => None,
        }
    }
}

pub fn is_valid_cron_sort_dir(s: &str) -> bool {
    CronSortDirSchema::from_str(s).is_some()
}

// ===========================================================================
// CronJobsEnabledFilterSchema
// ===========================================================================

/// 对齐 TS: `Type.Union([Type.Literal("all"), Type.Literal("enabled"), Type.Literal("disabled")])`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CronJobsEnabledFilterSchema {
    #[serde(rename = "all")]
    All,
    #[serde(rename = "enabled")]
    Enabled,
    #[serde(rename = "disabled")]
    Disabled,
}

impl CronJobsEnabledFilterSchema {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::All => "all",
            Self::Enabled => "enabled",
            Self::Disabled => "disabled",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "all" => Some(Self::All),
            "enabled" => Some(Self::Enabled),
            "disabled" => Some(Self::Disabled),
            _ => None,
        }
    }
}

pub fn is_valid_cron_jobs_enabled_filter(s: &str) -> bool {
    CronJobsEnabledFilterSchema::from_str(s).is_some()
}

// ===========================================================================
// CronJobsScheduleKindFilterSchema
// ===========================================================================

/// 对齐 TS: `Type.Union([Type.Literal("all"), Type.Literal("at"),
///                      Type.Literal("every"), Type.Literal("cron"),
///                      Type.Literal("on-exit")])`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum CronJobsScheduleKindFilterSchema {
    All,
    At,
    Every,
    Cron,
    OnExit,
}

impl CronJobsScheduleKindFilterSchema {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::All => "all",
            Self::At => "at",
            Self::Every => "every",
            Self::Cron => "cron",
            Self::OnExit => "on-exit",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "all" => Some(Self::All),
            "at" => Some(Self::At),
            "every" => Some(Self::Every),
            "cron" => Some(Self::Cron),
            "on-exit" => Some(Self::OnExit),
            _ => None,
        }
    }
}

pub fn is_valid_cron_jobs_schedule_kind_filter(s: &str) -> bool {
    CronJobsScheduleKindFilterSchema::from_str(s).is_some()
}

// ===========================================================================
// CronJobsLastRunStatusFilterSchema
// ===========================================================================

/// 对齐 TS: `Type.Union([Type.Literal("all"), Type.Literal("ok"),
///                      Type.Literal("error"), Type.Literal("skipped"),
///                      Type.Literal("unknown")])`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CronJobsLastRunStatusFilterSchema {
    #[serde(rename = "all")]
    All,
    #[serde(rename = "ok")]
    Ok,
    #[serde(rename = "error")]
    Error,
    #[serde(rename = "skipped")]
    Skipped,
    #[serde(rename = "unknown")]
    Unknown,
}

impl CronJobsLastRunStatusFilterSchema {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::All => "all",
            Self::Ok => "ok",
            Self::Error => "error",
            Self::Skipped => "skipped",
            Self::Unknown => "unknown",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "all" => Some(Self::All),
            "ok" => Some(Self::Ok),
            "error" => Some(Self::Error),
            "skipped" => Some(Self::Skipped),
            "unknown" => Some(Self::Unknown),
            _ => None,
        }
    }
}

pub fn is_valid_cron_jobs_last_run_status_filter(s: &str) -> bool {
    CronJobsLastRunStatusFilterSchema::from_str(s).is_some()
}

// ===========================================================================
// CronJobsSortBySchema
// ===========================================================================

/// 对齐 TS: `Type.Union([Type.Literal("nextRunAtMs"), Type.Literal("updatedAtMs"),
///                      Type.Literal("name")])`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CronJobsSortBySchema {
    #[serde(rename = "nextRunAtMs")]
    NextRunAtMs,
    #[serde(rename = "updatedAtMs")]
    UpdatedAtMs,
    #[serde(rename = "name")]
    Name,
}

impl CronJobsSortBySchema {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::NextRunAtMs => "nextRunAtMs",
            Self::UpdatedAtMs => "updatedAtMs",
            Self::Name => "name",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "nextRunAtMs" => Some(Self::NextRunAtMs),
            "updatedAtMs" => Some(Self::UpdatedAtMs),
            "name" => Some(Self::Name),
            _ => None,
        }
    }
}

pub fn is_valid_cron_jobs_sort_by(s: &str) -> bool {
    CronJobsSortBySchema::from_str(s).is_some()
}

// ===========================================================================
// CronRunsStatusFilterSchema / CronRunsStatusValueSchema
// ===========================================================================

/// 对齐 TS: `Type.Union([Type.Literal("all"), Type.Literal("ok"),
///                      Type.Literal("error"), Type.Literal("skipped")])`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CronRunsStatusFilterSchema {
    #[serde(rename = "all")]
    All,
    #[serde(rename = "ok")]
    Ok,
    #[serde(rename = "error")]
    Error,
    #[serde(rename = "skipped")]
    Skipped,
}

impl CronRunsStatusFilterSchema {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::All => "all",
            Self::Ok => "ok",
            Self::Error => "error",
            Self::Skipped => "skipped",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "all" => Some(Self::All),
            "ok" => Some(Self::Ok),
            "error" => Some(Self::Error),
            "skipped" => Some(Self::Skipped),
            _ => None,
        }
    }
}

pub fn is_valid_cron_runs_status_filter(s: &str) -> bool {
    CronRunsStatusFilterSchema::from_str(s).is_some()
}

/// Closed enumeration for `statuses[]` element values.
/// 对齐 TS: `Type.Union([Type.Literal("ok"), Type.Literal("error"), Type.Literal("skipped")])`.
pub type CronRunsStatusValueSchema = CronRunStatusSchema;

// ===========================================================================
// CronDeliveryStatusSchema
// ===========================================================================

/// 对齐 TS: `Type.Union([Type.Literal("delivered"), Type.Literal("not-delivered"),
///                      Type.Literal("unknown"), Type.Literal("not-requested")])`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CronDeliveryStatusSchema {
    #[serde(rename = "delivered")]
    Delivered,
    #[serde(rename = "not-delivered")]
    NotDelivered,
    #[serde(rename = "unknown")]
    Unknown,
    #[serde(rename = "not-requested")]
    NotRequested,
}

impl CronDeliveryStatusSchema {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Delivered => "delivered",
            Self::NotDelivered => "not-delivered",
            Self::Unknown => "unknown",
            Self::NotRequested => "not-requested",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "delivered" => Some(Self::Delivered),
            "not-delivered" => Some(Self::NotDelivered),
            "unknown" => Some(Self::Unknown),
            "not-requested" => Some(Self::NotRequested),
            _ => None,
        }
    }
}

pub fn is_valid_cron_delivery_status(s: &str) -> bool {
    CronDeliveryStatusSchema::from_str(s).is_some()
}

// ===========================================================================
// NonBlankString / CronDeclarationKeySchema / CronDisplayNameSchema
// ===========================================================================

/// 对齐 TS: `NonBlankString = Type.String({ minLength: 1, pattern: "\\S" })`.
pub type NonBlankString = String;

/// 对齐 TS: `Type.String({ minLength: 1, maxLength: 200, pattern: "\\S" })`.
pub type CronDeclarationKeySchema = String;

/// 对齐 TS: `Type.String({ minLength: 1, maxLength: 200, pattern: "\\S" })`.
pub type CronDisplayNameSchema = String;

// ===========================================================================
// CronOwnerSchema
// ===========================================================================

/// 对齐 TS:
///   `Type.Object({
///       agentId:    Type.Optional(NonEmptyString),
///       sessionKey: Type.Optional(NonEmptyString),
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CronOwnerSchema {
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "agentId")]
    pub agent_id: Option<NonEmptyString>,
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "sessionKey")]
    pub session_key: Option<NonEmptyString>,
}

impl CronOwnerSchema {
    pub fn validate(&self) -> Result<(), String> {
        if let Some(a) = &self.agent_id {
            validate_non_empty_string("agentId", a)?;
        }
        if let Some(s) = &self.session_key {
            validate_non_empty_string("sessionKey", s)?;
        }
        Ok(())
    }
}

// ===========================================================================
// CronAnnounceChannelSchema
//   Union: "last" | NonBlankString (free-form)
// ===========================================================================

/// Announcement channel for cron delivery. Closed enum value `last` plus
/// any non-blank string.
/// 对齐 TS: `Type.Union([Type.Literal("last"), NonBlankString])`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum CronAnnounceChannelSchema {
    Last,
    Other(String),
}

impl CronAnnounceChannelSchema {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Last => "last",
            Self::Other(s) => s.as_str(),
        }
    }
}

// ===========================================================================
// CronFailoverReasonSchema
// ===========================================================================

/// 对齐 TS: 14-literals union.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CronFailoverReasonSchema {
    #[serde(rename = "auth")]
    Auth,
    #[serde(rename = "auth_permanent")]
    AuthPermanent,
    #[serde(rename = "format")]
    Format,
    #[serde(rename = "rate_limit")]
    RateLimit,
    #[serde(rename = "overloaded")]
    Overloaded,
    #[serde(rename = "billing")]
    Billing,
    #[serde(rename = "server_error")]
    ServerError,
    #[serde(rename = "timeout")]
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
    #[serde(rename = "unclassified")]
    Unclassified,
    #[serde(rename = "unknown")]
    Unknown,
}

impl CronFailoverReasonSchema {
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

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "auth" => Some(Self::Auth),
            "auth_permanent" => Some(Self::AuthPermanent),
            "format" => Some(Self::Format),
            "rate_limit" => Some(Self::RateLimit),
            "overloaded" => Some(Self::Overloaded),
            "billing" => Some(Self::Billing),
            "server_error" => Some(Self::ServerError),
            "timeout" => Some(Self::Timeout),
            "context_overflow" => Some(Self::ContextOverflow),
            "model_not_found" => Some(Self::ModelNotFound),
            "session_expired" => Some(Self::SessionExpired),
            "empty_response" => Some(Self::EmptyResponse),
            "no_error_details" => Some(Self::NoErrorDetails),
            "unclassified" => Some(Self::Unclassified),
            "unknown" => Some(Self::Unknown),
            _ => None,
        }
    }
}

pub fn is_valid_cron_failover_reason(s: &str) -> bool {
    CronFailoverReasonSchema::from_str(s).is_some()
}

// ===========================================================================
// CronRunDiagnosticSeveritySchema / SourceSchema / DiagnosticSchema / DiagnosticsSchema
// ===========================================================================

/// 对齐 TS: `Type.Union([Type.Literal("info"), Type.Literal("warn"), Type.Literal("error")])`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CronRunDiagnosticSeveritySchema {
    #[serde(rename = "info")]
    Info,
    #[serde(rename = "warn")]
    Warn,
    #[serde(rename = "error")]
    Error,
}

impl CronRunDiagnosticSeveritySchema {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Info => "info",
            Self::Warn => "warn",
            Self::Error => "error",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "info" => Some(Self::Info),
            "warn" => Some(Self::Warn),
            "error" => Some(Self::Error),
            _ => None,
        }
    }
}

pub fn is_valid_cron_run_diagnostic_severity(s: &str) -> bool {
    CronRunDiagnosticSeveritySchema::from_str(s).is_some()
}

/// 对齐 TS: 7-literals union.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum CronRunDiagnosticSourceSchema {
    CronPreflight,
    CronSetup,
    ModelPreflight,
    AgentRun,
    Tool,
    Exec,
    Delivery,
}

impl CronRunDiagnosticSourceSchema {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::CronPreflight => "cron-preflight",
            Self::CronSetup => "cron-setup",
            Self::ModelPreflight => "model-preflight",
            Self::AgentRun => "agent-run",
            Self::Tool => "tool",
            Self::Exec => "exec",
            Self::Delivery => "delivery",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "cron-preflight" => Some(Self::CronPreflight),
            "cron-setup" => Some(Self::CronSetup),
            "model-preflight" => Some(Self::ModelPreflight),
            "agent-run" => Some(Self::AgentRun),
            "tool" => Some(Self::Tool),
            "exec" => Some(Self::Exec),
            "delivery" => Some(Self::Delivery),
            _ => None,
        }
    }
}

pub fn is_valid_cron_run_diagnostic_source(s: &str) -> bool {
    CronRunDiagnosticSourceSchema::from_str(s).is_some()
}

/// 对齐 TS:
///   `Type.Object({
///       ts:        Type.Integer({ minimum: 0 }),
///       source:    CronRunDiagnosticSourceSchema,
///       severity:  CronRunDiagnosticSeveritySchema,
///       message:   Type.String(),
///       toolName:  Type.Optional(Type.String()),
///       exitCode:  Type.Optional(Type.Union([Type.Number(), Type.Null()])),
///       truncated: Type.Optional(Type.Boolean()),
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CronRunDiagnosticSchema {
    pub ts: i64,
    pub source: CronRunDiagnosticSourceSchema,
    pub severity: CronRunDiagnosticSeveritySchema,
    pub message: String,
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "toolName")]
    pub tool_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "exitCode")]
    pub exit_code: Option<CronRunDiagnosticExitCode>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub truncated: Option<bool>,
}

/// `exitCode` accepts a number or `null`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum CronRunDiagnosticExitCode {
    Number(f64),
    Null,
}

impl CronRunDiagnosticSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_non_negative_integer("ts", self.ts)?;
        Ok(())
    }
}

/// 对齐 TS:
///   `Type.Object({
///       summary: Type.Optional(Type.String()),
///       entries: Type.Array(CronRunDiagnosticSchema),
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CronRunDiagnosticsSchema {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
    pub entries: Vec<CronRunDiagnosticSchema>,
}

impl CronRunDiagnosticsSchema {
    pub fn validate(&self) -> Result<(), String> {
        for (i, e) in self.entries.iter().enumerate() {
            e.validate().map_err(|e_| format!("entries[{}]: {}", i, e_))?;
        }
        Ok(())
    }
}

// ===========================================================================
// CronRunLogJobIdSchema
// ===========================================================================

/// 对齐 TS:
///   `Type.String({ minLength: 1, pattern: "^[^/\\\\]+$" })`.
pub type CronRunLogJobIdSchema = String;

// ===========================================================================
// CronScheduleSchema (discriminated union on `kind`)
// ===========================================================================

/// Schedule expression for one-time, interval, or cron-expression jobs.
/// 对齐 TS: `Type.Union([...4 variants])`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum CronScheduleSchema {
    At { at: NonEmptyString },
    Every {
        #[serde(rename = "everyMs")]
        every_ms: i64,
        #[serde(
            default,
            skip_serializing_if = "Option::is_none",
            rename = "anchorMs"
        )]
        anchor_ms: Option<i64>,
    },
    Cron {
        expr: NonEmptyString,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        tz: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none", rename = "staggerMs")]
        stagger_ms: Option<i64>,
    },
    /// Event-driven trigger: fires once when the gateway-owned watcher
    /// running `command` exits. Survives per-turn CLI teardown.
    OnExit {
        command: NonEmptyString,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        cwd: Option<NonEmptyString>,
    },
}

impl CronScheduleSchema {
    pub fn validate(&self) -> Result<(), String> {
        match self {
            Self::At { at } => validate_non_empty_string("at", at)?,
            Self::Every { every_ms, anchor_ms } => {
                if *every_ms < 1 {
                    return Err(format!("everyMs: expected >= 1, got {}", every_ms));
                }
                if let Some(a) = anchor_ms {
                    validate_non_negative_integer("anchorMs", *a)?;
                }
            }
            Self::Cron {
                expr,
                tz: _,
                stagger_ms,
            } => {
                validate_non_empty_string("expr", expr)?;
                if let Some(s) = stagger_ms {
                    validate_non_negative_integer("staggerMs", *s)?;
                }
            }
            Self::OnExit { command, cwd } => {
                validate_non_empty_string("command", command)?;
                if let Some(c) = cwd {
                    validate_non_empty_string("cwd", c)?;
                }
            }
        }
        Ok(())
    }
}

// ===========================================================================
// CronTriggerSchema
// ===========================================================================

/// Headless condition script evaluated before a recurring cron payload runs.
/// 对齐 TS:
///   `Type.Object({
///       script: Type.String({ minLength: 1, maxLength: 65_536 }),
///       once:   Type.Optional(Type.Boolean()),
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CronTriggerSchema {
    pub script: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub once: Option<bool>,
}

impl CronTriggerSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_bounded_string(
            "script",
            &self.script,
            CRON_TRIGGER_SCRIPT_MIN_LENGTH,
            CRON_TRIGGER_SCRIPT_MAX_LENGTH,
        )?;
        Ok(())
    }
}

// ===========================================================================
// CronPayloadSchema (discriminated union on `kind`)
// ===========================================================================

/// `agentTurn` payload variant.
/// 对齐 TS: `cronAgentTurnPayloadSchema({ message: NonEmptyString, ... })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CronAgentTurnPayloadSchema {
    #[serde(rename = "kind")]
    pub kind_value: CronPayloadAgentTurnKind,
    pub message: NonEmptyString,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fallbacks: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub thinking: Option<String>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "timeoutSeconds"
    )]
    pub timeout_seconds: Option<f64>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "allowUnsafeExternalContent"
    )]
    pub allow_unsafe_external_content: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "lightContext")]
    pub light_context: Option<bool>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "toolsAllow"
    )]
    pub tools_allow: Option<Vec<String>>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "toolsAllowIsDefault"
    )]
    pub tools_allow_is_default: Option<bool>,
}

/// Literal `"agentTurn"` discriminator used in `CronAgentTurnPayloadSchema.kind`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CronPayloadAgentTurnKind {
    #[serde(rename = "agentTurn")]
    AgentTurn,
}

/// `command` payload variant.
/// 对齐 TS: `cronCommandPayloadSchema({ argv: ... })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CronCommandPayloadSchema {
    #[serde(rename = "kind")]
    pub kind_value: CronPayloadCommandKind,
    pub argv: Vec<NonEmptyString>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cwd: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub env: Option<std::collections::BTreeMap<String, String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub input: Option<String>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "timeoutSeconds"
    )]
    pub timeout_seconds: Option<f64>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "noOutputTimeoutSeconds"
    )]
    pub no_output_timeout_seconds: Option<f64>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "outputMaxBytes"
    )]
    pub output_max_bytes: Option<i64>,
}

/// Literal `"command"` discriminator used in `CronCommandPayloadSchema.kind`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CronPayloadCommandKind {
    #[serde(rename = "command")]
    Command,
}

/// `systemEvent` payload variant.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CronSystemEventPayloadSchema {
    #[serde(rename = "kind")]
    pub kind_value: CronPayloadSystemEventKind,
    pub text: NonEmptyString,
}

/// Literal `"systemEvent"` discriminator used in `CronSystemEventPayloadSchema.kind`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CronPayloadSystemEventKind {
    #[serde(rename = "systemEvent")]
    SystemEvent,
}

/// Full cron payload for new jobs.
/// 对齐 TS: `Type.Union([systemEvent, agentTurn, command])`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum CronPayloadSchema {
    SystemEvent(CronSystemEventPayloadSchema),
    AgentTurn(CronAgentTurnPayloadSchema),
    Command(CronCommandPayloadSchema),
}

impl CronPayloadSchema {
    pub fn validate(&self) -> Result<(), String> {
        match self {
            Self::SystemEvent(p) => validate_non_empty_string("text", &p.text)?,
            Self::AgentTurn(p) => validate_non_empty_string("message", &p.message)?,
            Self::Command(p) => {
                if p.argv.is_empty() {
                    return Err("argv: minItems 1, got 0".to_string());
                }
                for (i, a) in p.argv.iter().enumerate() {
                    validate_non_empty_string(&format!("argv[{}]", i), a)?;
                }
            }
        }
        Ok(())
    }
}

// ===========================================================================
// CronPayloadPatchSchema
// ===========================================================================

/// Patch variant of `agentTurn` payload (every field becomes optional/null).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CronAgentTurnPayloadPatchSchema {
    #[serde(rename = "kind")]
    pub kind_value: CronPayloadAgentTurnKind,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message: Option<NonEmptyString>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: Option<CronPayloadPatchNullable<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fallbacks: Option<CronPayloadPatchNullable<Vec<String>>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub thinking: Option<CronPayloadPatchNullable<String>>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "timeoutSeconds"
    )]
    pub timeout_seconds: Option<f64>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "allowUnsafeExternalContent"
    )]
    pub allow_unsafe_external_content: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "lightContext")]
    pub light_context: Option<bool>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "toolsAllow"
    )]
    pub tools_allow: Option<CronPayloadPatchNullable<Vec<String>>>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "toolsAllowIsDefault"
    )]
    pub tools_allow_is_default: Option<bool>,
}

/// Patch variant of `command` payload (argv becomes optional).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CronCommandPayloadPatchSchema {
    #[serde(rename = "kind")]
    pub kind_value: CronPayloadCommandKind,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub argv: Option<Vec<NonEmptyString>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cwd: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub env: Option<std::collections::BTreeMap<String, String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub input: Option<String>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "timeoutSeconds"
    )]
    pub timeout_seconds: Option<f64>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "noOutputTimeoutSeconds"
    )]
    pub no_output_timeout_seconds: Option<f64>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "outputMaxBytes"
    )]
    pub output_max_bytes: Option<i64>,
}

/// Patch variant of `systemEvent` payload (text becomes optional).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CronSystemEventPayloadPatchSchema {
    #[serde(rename = "kind")]
    pub kind_value: CronPayloadSystemEventKind,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text: Option<NonEmptyString>,
}

/// `T | null` mirror used by patch variants to allow clearing a field.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum CronPayloadPatchNullable<T> {
    Value(T),
    Null,
}

/// Partial cron payload for job updates.
/// 对齐 TS: `Type.Union([systemEvent-patch, agentTurn-patch, command-patch])`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum CronPayloadPatchSchema {
    SystemEvent(CronSystemEventPayloadPatchSchema),
    AgentTurn(CronAgentTurnPayloadPatchSchema),
    Command(CronCommandPayloadPatchSchema),
}

impl CronPayloadPatchSchema {
    pub fn validate(&self) -> Result<(), String> {
        match self {
            Self::SystemEvent(p) => {
                if let Some(t) = &p.text {
                    validate_non_empty_string("text", t)?;
                }
            }
            Self::AgentTurn(p) => {
                if let Some(m) = &p.message {
                    validate_non_empty_string("message", m)?;
                }
            }
            Self::Command(p) => {
                if let Some(argv) = &p.argv {
                    if argv.is_empty() {
                        return Err("argv: minItems 1, got 0".to_string());
                    }
                    for (i, a) in argv.iter().enumerate() {
                        validate_non_empty_string(&format!("argv[{}]", i), a)?;
                    }
                }
            }
        }
        Ok(())
    }
}

// ===========================================================================
// CronFailureAlertSchema / CronFailureDestinationSchema
// ===========================================================================

/// Failure alert mode accepts `announce` or `webhook`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CronFailureAlertMode {
    #[serde(rename = "announce")]
    Announce,
    #[serde(rename = "webhook")]
    Webhook,
}

impl CronFailureAlertMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Announce => "announce",
            Self::Webhook => "webhook",
        }
    }
}

/// Failure alert policy for repeated cron run failures.
/// 对齐 TS: `Type.Object({...})`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CronFailureAlertSchema {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub after: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub channel: Option<CronAnnounceChannelSchema>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub to: Option<NonBlankString>,
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "cooldownMs")]
    pub cooldown_ms: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "includeSkipped")]
    pub include_skipped: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mode: Option<CronFailureAlertMode>,
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "accountId")]
    pub account_id: Option<NonEmptyString>,
}

impl CronFailureAlertSchema {
    pub fn validate(&self) -> Result<(), String> {
        if let Some(a) = self.after {
            if a < 1 {
                return Err(format!("after: expected >= 1, got {}", a));
            }
        }
        if let Some(t) = &self.to {
            validate_non_blank_string("to", t)?;
        }
        if let Some(c) = self.cooldown_ms {
            validate_non_negative_integer("cooldownMs", c)?;
        }
        if let Some(a) = &self.account_id {
            validate_non_empty_string("accountId", a)?;
        }
        Ok(())
    }
}

/// Delivery destination used when failure alerts need a separate target.
/// 对齐 TS: `Type.Object({...})`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CronFailureDestinationSchema {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub channel: Option<CronAnnounceChannelSchema>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub to: Option<NonBlankString>,
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "accountId")]
    pub account_id: Option<NonEmptyString>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mode: Option<CronFailureAlertMode>,
}

impl CronFailureDestinationSchema {
    pub fn validate(&self) -> Result<(), String> {
        if let Some(t) = &self.to {
            validate_non_blank_string("to", t)?;
        }
        if let Some(a) = &self.account_id {
            validate_non_empty_string("accountId", a)?;
        }
        Ok(())
    }
}

// Patch variant — every field accepts `null` to clear.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CronFailureDestinationPatchSchema {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub channel: Option<CronNullableChannel>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub to: Option<CronPayloadPatchNullable<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "accountId")]
    pub account_id: Option<CronPayloadPatchNullable<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mode: Option<CronNullableFailureMode>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum CronNullableChannel {
    Value(CronAnnounceChannelSchema),
    Null,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum CronNullableFailureMode {
    Value(CronFailureAlertMode),
    Null,
}

// ===========================================================================
// CronCompletionDestinationSchema
// ===========================================================================

/// Successful completion destination (webhook).
/// 对齐 TS:
///   `Type.Object({
///       mode: Type.Literal("webhook"),
///       to:   NonBlankString,
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CronCompletionDestinationSchema {
    pub mode: CronCompletionDestinationMode,
    pub to: NonBlankString,
}

/// Literal `"webhook"` discriminator for completion destinations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CronCompletionDestinationMode {
    #[serde(rename = "webhook")]
    Webhook,
}

impl CronCompletionDestinationSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_non_blank_string("to", &self.to)?;
        Ok(())
    }
}

// ===========================================================================
// CronDeliverySchema (discriminated union on `mode`)
// ===========================================================================

/// Delivery mode literal `"none"`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CronDeliveryModeNone {
    #[serde(rename = "none")]
    None,
}

/// Delivery mode literal `"announce"`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CronDeliveryModeAnnounce {
    #[serde(rename = "announce")]
    Announce,
}

/// Delivery mode literal `"webhook"`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CronDeliveryModeWebhook {
    #[serde(rename = "webhook")]
    Webhook,
}

/// `threadId` accepts a string or number.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum CronDeliveryThreadId {
    Text(String),
    Number(f64),
}

/// Shared fields for the active delivery union variants.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CronDeliverySharedProperties {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub channel: Option<CronAnnounceChannelSchema>,
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "threadId")]
    pub thread_id: Option<CronDeliveryThreadId>,
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "accountId")]
    pub account_id: Option<NonEmptyString>,
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "bestEffort")]
    pub best_effort: Option<bool>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "failureDestination"
    )]
    pub failure_destination: Option<CronFailureDestinationSchema>,
}

/// Shared fields for the patch delivery variants.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CronDeliveryPatchSharedProperties {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub channel: Option<CronNullableChannel>,
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "threadId")]
    pub thread_id: Option<CronNullableThreadId>,
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "accountId")]
    pub account_id: Option<CronPayloadPatchNullable<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "bestEffort")]
    pub best_effort: Option<bool>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "failureDestination"
    )]
    pub failure_destination: Option<CronPayloadPatchNullable<CronFailureDestinationPatchSchema>>,
}

/// `threadId | null` patch variant.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum CronNullableThreadId {
    Value(CronDeliveryThreadId),
    Null,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "mode", rename_all = "camelCase")]
pub enum CronDeliverySchema {
    None {
        #[serde(flatten)]
        shared: CronDeliverySharedProperties,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        to: Option<NonBlankString>,
    },
    Announce {
        #[serde(flatten)]
        shared: CronDeliverySharedProperties,
        #[serde(
            default,
            skip_serializing_if = "Option::is_none",
            rename = "completionDestination"
        )]
        completion_destination: Option<CronCompletionDestinationSchema>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        to: Option<NonBlankString>,
    },
    Webhook {
        #[serde(flatten)]
        shared: CronDeliverySharedProperties,
        to: NonBlankString,
    },
}

impl CronDeliverySchema {
    pub fn validate(&self) -> Result<(), String> {
        match self {
            Self::None { shared, to } => {
                shared.validate().map_err(|e| format!("shared: {}", e))?;
                if let Some(t) = to {
                    validate_non_blank_string("to", t)?;
                }
            }
            Self::Announce {
                shared,
                completion_destination,
                to,
            } => {
                shared.validate().map_err(|e| format!("shared: {}", e))?;
                if let Some(cd) = completion_destination {
                    cd.validate()
                        .map_err(|e| format!("completionDestination: {}", e))?;
                }
                if let Some(t) = to {
                    validate_non_blank_string("to", t)?;
                }
            }
            Self::Webhook { shared, to } => {
                shared.validate().map_err(|e| format!("shared: {}", e))?;
                validate_non_blank_string("to", to)?;
            }
        }
        Ok(())
    }
}

impl CronDeliverySharedProperties {
    pub fn validate(&self) -> Result<(), String> {
        if let Some(a) = &self.account_id {
            validate_non_empty_string("accountId", a)?;
        }
        if let Some(fd) = &self.failure_destination {
            fd.validate()
                .map_err(|e| format!("failureDestination: {}", e))?;
        }
        Ok(())
    }
}

/// Patch shape for cron delivery policy updates.
/// 对齐 TS: `Type.Object({...})`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CronDeliveryPatchSchema {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mode: Option<CronDeliveryPatchMode>,
    #[serde(flatten)]
    pub shared: CronDeliveryPatchSharedProperties,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "completionDestination"
    )]
    pub completion_destination: Option<CronNullableCompletionDestination>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub to: Option<CronPayloadPatchNullable<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum CronDeliveryPatchMode {
    None(CronDeliveryModeNone),
    Announce(CronDeliveryModeAnnounce),
    Webhook(CronDeliveryModeWebhook),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum CronNullableCompletionDestination {
    Value(CronCompletionDestinationSchema),
    Null,
}

// ===========================================================================
// CronFailureNotificationDeliverySchema
// ===========================================================================

/// 对齐 TS:
///   `Type.Object({
///       delivered: Type.Optional(Type.Boolean()),
///       status:    CronDeliveryStatusSchema,
///       error:     Type.Optional(Type.String()),
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CronFailureNotificationDeliverySchema {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub delivered: Option<bool>,
    pub status: CronDeliveryStatusSchema,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

// ===========================================================================
// CronJobStateSchema
// ===========================================================================

/// Scheduler-maintained state for the latest run/delivery outcome.
/// 对齐 TS: `Type.Object({...})`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CronJobStateSchema {
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "nextRunAtMs")]
    pub next_run_at_ms: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "runningAtMs")]
    pub running_at_ms: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "lastRunAtMs")]
    pub last_run_at_ms: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "lastRunStatus")]
    pub last_run_status: Option<CronRunStatusSchema>,
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "lastStatus")]
    pub last_status: Option<DeprecatedCronRunStatusSchema>,
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "lastError")]
    pub last_error: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "lastDiagnostics")]
    pub last_diagnostics: Option<CronRunDiagnosticsSchema>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "lastDiagnosticSummary"
    )]
    pub last_diagnostic_summary: Option<String>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "lastErrorReason"
    )]
    pub last_error_reason: Option<CronFailoverReasonSchema>,
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "lastDurationMs")]
    pub last_duration_ms: Option<i64>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "consecutiveErrors"
    )]
    pub consecutive_errors: Option<i64>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "consecutiveSkipped"
    )]
    pub consecutive_skipped: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "lastDelivered")]
    pub last_delivered: Option<bool>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "lastDeliveryStatus"
    )]
    pub last_delivery_status: Option<CronDeliveryStatusSchema>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "lastDeliveryError"
    )]
    pub last_delivery_error: Option<String>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "lastFailureNotificationDelivered"
    )]
    pub last_failure_notification_delivered: Option<bool>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "lastFailureNotificationDeliveryStatus"
    )]
    pub last_failure_notification_delivery_status: Option<CronDeliveryStatusSchema>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "lastFailureNotificationDeliveryError"
    )]
    pub last_failure_notification_delivery_error: Option<String>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "lastFailureAlertAtMs"
    )]
    pub last_failure_alert_at_ms: Option<i64>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "lastTriggerEvalAtMs"
    )]
    pub last_trigger_eval_at_ms: Option<i64>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "triggerEvalCount"
    )]
    pub trigger_eval_count: Option<i64>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "lastTriggerFireAtMs"
    )]
    pub last_trigger_fire_at_ms: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "triggerState")]
    pub trigger_state: Option<Value>,
}

impl CronJobStateSchema {
    pub fn validate(&self) -> Result<(), String> {
        if let Some(t) = self.next_run_at_ms {
            validate_non_negative_integer("nextRunAtMs", t)?;
        }
        if let Some(t) = self.running_at_ms {
            validate_non_negative_integer("runningAtMs", t)?;
        }
        if let Some(t) = self.last_run_at_ms {
            validate_non_negative_integer("lastRunAtMs", t)?;
        }
        if let Some(t) = self.last_duration_ms {
            validate_non_negative_integer("lastDurationMs", t)?;
        }
        if let Some(t) = self.consecutive_errors {
            validate_non_negative_integer("consecutiveErrors", t)?;
        }
        if let Some(t) = self.consecutive_skipped {
            validate_non_negative_integer("consecutiveSkipped", t)?;
        }
        if let Some(t) = self.last_failure_alert_at_ms {
            validate_non_negative_integer("lastFailureAlertAtMs", t)?;
        }
        if let Some(t) = self.last_trigger_eval_at_ms {
            validate_non_negative_integer("lastTriggerEvalAtMs", t)?;
        }
        if let Some(t) = self.trigger_eval_count {
            validate_non_negative_integer("triggerEvalCount", t)?;
        }
        if let Some(t) = self.last_trigger_fire_at_ms {
            validate_non_negative_integer("lastTriggerFireAtMs", t)?;
        }
        if let Some(d) = &self.last_diagnostics {
            d.validate().map_err(|e| format!("lastDiagnostics: {}", e))?;
        }
        Ok(())
    }
}

// ===========================================================================
// CronJobStatePatchSchema (subset of CronJobStateSchema, no diagnostics fields)
// ===========================================================================

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CronJobStatePatchSchema {
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "nextRunAtMs")]
    pub next_run_at_ms: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "runningAtMs")]
    pub running_at_ms: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "lastRunAtMs")]
    pub last_run_at_ms: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "lastRunStatus")]
    pub last_run_status: Option<CronRunStatusSchema>,
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "lastStatus")]
    pub last_status: Option<DeprecatedCronRunStatusSchema>,
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "lastError")]
    pub last_error: Option<String>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "lastErrorReason"
    )]
    pub last_error_reason: Option<CronFailoverReasonSchema>,
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "lastDurationMs")]
    pub last_duration_ms: Option<i64>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "consecutiveErrors"
    )]
    pub consecutive_errors: Option<i64>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "consecutiveSkipped"
    )]
    pub consecutive_skipped: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "lastDelivered")]
    pub last_delivered: Option<bool>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "lastDeliveryStatus"
    )]
    pub last_delivery_status: Option<CronDeliveryStatusSchema>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "lastDeliveryError"
    )]
    pub last_delivery_error: Option<String>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "lastFailureNotificationDelivered"
    )]
    pub last_failure_notification_delivered: Option<bool>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "lastFailureNotificationDeliveryStatus"
    )]
    pub last_failure_notification_delivery_status: Option<CronDeliveryStatusSchema>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "lastFailureNotificationDeliveryError"
    )]
    pub last_failure_notification_delivery_error: Option<String>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "lastFailureAlertAtMs"
    )]
    pub last_failure_alert_at_ms: Option<i64>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "lastTriggerEvalAtMs"
    )]
    pub last_trigger_eval_at_ms: Option<i64>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "triggerEvalCount"
    )]
    pub trigger_eval_count: Option<i64>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "lastTriggerFireAtMs"
    )]
    pub last_trigger_fire_at_ms: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "triggerState")]
    pub trigger_state: Option<Value>,
}

impl CronJobStatePatchSchema {
    pub fn validate(&self) -> Result<(), String> {
        if let Some(t) = self.next_run_at_ms {
            validate_non_negative_integer("nextRunAtMs", t)?;
        }
        if let Some(t) = self.last_duration_ms {
            validate_non_negative_integer("lastDurationMs", t)?;
        }
        if let Some(t) = self.consecutive_errors {
            validate_non_negative_integer("consecutiveErrors", t)?;
        }
        if let Some(t) = self.consecutive_skipped {
            validate_non_negative_integer("consecutiveSkipped", t)?;
        }
        Ok(())
    }
}

// ===========================================================================
// CronFailureAlertOption / CronJobSchema
// ===========================================================================

/// `failureAlert` field accepts `false` (cleared) or an alert schema.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum CronFailureAlertOption {
    Disabled(bool),
    Alert(CronFailureAlertSchema),
}

/// Persisted cron job definition returned by scheduler list/get APIs.
/// 对齐 TS: `Type.Object({...})`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CronJobSchema {
    pub id: NonEmptyString,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "declarationKey"
    )]
    pub declaration_key: Option<CronDeclarationKeySchema>,
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "displayName")]
    pub display_name: Option<CronDisplayNameSchema>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub owner: Option<CronOwnerSchema>,
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "agentId")]
    pub agent_id: Option<NonEmptyString>,
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "sessionKey")]
    pub session_key: Option<NonEmptyString>,
    pub name: NonEmptyString,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub enabled: bool,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "deleteAfterRun"
    )]
    pub delete_after_run: Option<bool>,
    #[serde(rename = "createdAtMs")]
    pub created_at_ms: i64,
    #[serde(rename = "updatedAtMs")]
    pub updated_at_ms: i64,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "configRevision"
    )]
    pub config_revision: Option<CronConfigRevisionSchema>,
    pub schedule: CronScheduleSchema,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub trigger: Option<CronTriggerSchema>,
    #[serde(rename = "sessionTarget")]
    pub session_target: CronSessionTargetSchema,
    #[serde(rename = "wakeMode")]
    pub wake_mode: CronWakeModeSchema,
    pub payload: CronPayloadSchema,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub delivery: Option<CronDeliverySchema>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "failureAlert"
    )]
    pub failure_alert: Option<CronFailureAlertOption>,
    pub state: CronJobStateSchema,
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "nextRunAtMs")]
    pub next_run_at_ms_flat: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "lastRunAtMs")]
    pub last_run_at_ms_flat: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "lastRunStatus")]
    pub last_run_status_flat: Option<CronRunStatusSchema>,
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "lastRunError")]
    pub last_run_error: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "lastDelivered")]
    pub last_delivered_flat: Option<bool>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "lastDeliveryStatus"
    )]
    pub last_delivery_status_flat: Option<CronDeliveryStatusSchema>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "lastDeliveryError"
    )]
    pub last_delivery_error_flat: Option<String>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "lastFailureNotificationDelivered"
    )]
    pub last_failure_notification_delivered_flat: Option<bool>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "lastFailureNotificationDeliveryStatus"
    )]
    pub last_failure_notification_delivery_status_flat: Option<CronDeliveryStatusSchema>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "lastFailureNotificationDeliveryError"
    )]
    pub last_failure_notification_delivery_error_flat: Option<String>,
}

impl CronJobSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_non_empty_string("id", &self.id)?;
        if let Some(k) = &self.declaration_key {
            if !is_valid_cron_declaration_key(k) {
                return Err(format!(
                    "declarationKey: invalid cron declaration key: {:?}",
                    k
                ));
            }
        }
        if let Some(n) = &self.display_name {
            if !is_valid_cron_display_name(n) {
                return Err(format!(
                    "displayName: invalid cron display name: {:?}",
                    n
                ));
            }
        }
        if let Some(o) = &self.owner {
            o.validate().map_err(|e| format!("owner: {}", e))?;
        }
        validate_non_empty_string("name", &self.name)?;
        validate_non_negative_integer("createdAtMs", self.created_at_ms)?;
        validate_non_negative_integer("updatedAtMs", self.updated_at_ms)?;
        if let Some(r) = &self.config_revision {
            if !is_valid_cron_config_revision(r) {
                return Err(format!(
                    "configRevision: invalid cron config revision: {:?}",
                    r
                ));
            }
        }
        self.schedule.validate().map_err(|e| format!("schedule: {}", e))?;
        if let Some(t) = &self.trigger {
            t.validate().map_err(|e| format!("trigger: {}", e))?;
        }
        self.payload.validate().map_err(|e| format!("payload: {}", e))?;
        if let Some(d) = &self.delivery {
            d.validate().map_err(|e| format!("delivery: {}", e))?;
        }
        self.state.validate().map_err(|e| format!("state: {}", e))?;
        if let Some(CronFailureAlertOption::Alert(a)) = &self.failure_alert {
            a.validate().map_err(|e| format!("failureAlert: {}", e))?;
        }
        Ok(())
    }
}

// ===========================================================================
// CronListParamsSchema
// ===========================================================================

/// Query params for listing cron jobs with filters and pagination.
/// 对齐 TS: `Type.Object({...})`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CronListParamsSchema {
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "includeDisabled"
    )]
    pub include_disabled: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub limit: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub offset: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub query: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub enabled: Option<CronJobsEnabledFilterSchema>,
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "scheduleKind")]
    pub schedule_kind: Option<CronJobsScheduleKindFilterSchema>,
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "lastRunStatus")]
    pub last_run_status: Option<CronJobsLastRunStatusFilterSchema>,
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "sortBy")]
    pub sort_by: Option<CronJobsSortBySchema>,
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "sortDir")]
    pub sort_dir: Option<CronSortDirSchema>,
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "agentId")]
    pub agent_id: Option<NonEmptyString>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub compact: Option<bool>,
}

impl CronListParamsSchema {
    pub fn validate(&self) -> Result<(), String> {
        if let Some(l) = self.limit {
            if l < CRON_LIST_LIMIT_MIN || l > CRON_LIST_LIMIT_MAX {
                return Err(format!(
                    "limit: expected [{}, {}], got {}",
                    CRON_LIST_LIMIT_MIN, CRON_LIST_LIMIT_MAX, l
                ));
            }
        }
        if let Some(o) = self.offset {
            if o < CRON_LIST_OFFSET_MIN {
                return Err(format!(
                    "offset: expected >= {}, got {}",
                    CRON_LIST_OFFSET_MIN, o
                ));
            }
        }
        if let Some(a) = &self.agent_id {
            validate_non_empty_string("agentId", a)?;
        }
        Ok(())
    }
}

// ===========================================================================
// CronStatusParamsSchema / CronGetParamsSchema / CronRemoveParamsSchema
// ===========================================================================

/// Empty request payload for scheduler status.
/// 对齐 TS: `Type.Object({}, { additionalProperties: false })`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CronStatusParamsSchema {}

impl CronStatusParamsSchema {
    pub fn validate(&self) -> Result<(), String> {
        Ok(())
    }
}

/// Identifies a job by either stable `id` or legacy `jobId` alias.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum CronIdOrJobIdParamsSchema {
    Id {
        #[serde(rename = "id")]
        id_value: NonEmptyString,
    },
    JobId {
        #[serde(rename = "jobId")]
        job_id_value: NonEmptyString,
    },
}

/// Looks up a job by stable id or legacy jobId alias.
/// 对齐 TS: `cronIdOrJobIdParams({})`.
pub type CronGetParamsSchema = CronIdOrJobIdParamsSchema;

/// Removes a cron job by id or legacy jobId alias.
/// 对齐 TS: `cronIdOrJobIdParams({})`.
pub type CronRemoveParamsSchema = CronIdOrJobIdParamsSchema;

// ===========================================================================
// CronAddParamsSchema / CronDeclarativeAddResultSchema / CronAddResultSchema
// ===========================================================================

/// Shared optional fields reused across add/patch payloads.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CronCommonOptionalFields {
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "agentId")]
    pub agent_id: Option<CronPayloadPatchNullable<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "sessionKey")]
    pub session_key: Option<CronPayloadPatchNullable<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "deleteAfterRun"
    )]
    pub delete_after_run: Option<bool>,
}

/// Creates a scheduled job with schedule, target, payload, and delivery policy.
/// 对齐 TS: `Type.Object({...})`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CronAddParamsSchema {
    pub name: NonEmptyString,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "declarationKey"
    )]
    pub declaration_key: Option<CronDeclarationKeySchema>,
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "displayName")]
    pub display_name: Option<CronDisplayNameSchema>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub owner: Option<CronOwnerSchema>,
    #[serde(flatten)]
    pub common: CronCommonOptionalFields,
    pub schedule: CronScheduleSchema,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub trigger: Option<CronTriggerSchema>,
    #[serde(rename = "sessionTarget")]
    pub session_target: CronSessionTargetSchema,
    #[serde(rename = "wakeMode")]
    pub wake_mode: CronWakeModeSchema,
    pub payload: CronPayloadSchema,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub delivery: Option<CronDeliverySchema>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "failureAlert"
    )]
    pub failure_alert: Option<CronFailureAlertOption>,
}

impl CronAddParamsSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_non_empty_string("name", &self.name)?;
        if let Some(k) = &self.declaration_key {
            if !is_valid_cron_declaration_key(k) {
                return Err(format!(
                    "declarationKey: invalid cron declaration key: {:?}",
                    k
                ));
            }
        }
        if let Some(n) = &self.display_name {
            if !is_valid_cron_display_name(n) {
                return Err(format!(
                    "displayName: invalid cron display name: {:?}",
                    n
                ));
            }
        }
        if let Some(o) = &self.owner {
            o.validate().map_err(|e| format!("owner: {}", e))?;
        }
        self.schedule.validate().map_err(|e| format!("schedule: {}", e))?;
        if let Some(t) = &self.trigger {
            t.validate().map_err(|e| format!("trigger: {}", e))?;
        }
        self.payload.validate().map_err(|e| format!("payload: {}", e))?;
        if let Some(d) = &self.delivery {
            d.validate().map_err(|e| format!("delivery: {}", e))?;
        }
        if let Some(CronFailureAlertOption::Alert(a)) = &self.failure_alert {
            a.validate().map_err(|e| format!("failureAlert: {}", e))?;
        }
        Ok(())
    }
}

/// Successful declaration-key convergence result.
/// 对齐 TS:
///   `Type.Object({
///       created: Type.Boolean(),
///       updated: Type.Optional(Type.Boolean()),
///       job:     CronJobSchema,
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CronDeclarativeAddResultSchema {
    pub created: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub updated: Option<bool>,
    pub job: CronJobSchema,
}

/// Successful result from imperative create or declaration-key convergence.
/// 对齐 TS: `Type.Union([CronJobSchema, CronDeclarativeAddResultSchema])`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum CronAddResultSchema {
    Job(CronJobSchema),
    Declarative(CronDeclarativeAddResultSchema),
}

// ===========================================================================
// CronJobPatchSchema
// ===========================================================================

/// Mutable cron job fields accepted by update APIs.
/// 对齐 TS: `Type.Object({...})`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CronJobPatchSchema {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<NonEmptyString>,
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "displayName")]
    pub display_name: Option<CronNullableDisplayName>,
    #[serde(flatten)]
    pub common: CronCommonOptionalFields,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub schedule: Option<CronScheduleSchema>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub trigger: Option<CronNullableTrigger>,
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "sessionTarget")]
    pub session_target: Option<CronSessionTargetSchema>,
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "wakeMode")]
    pub wake_mode: Option<CronWakeModeSchema>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub payload: Option<CronPayloadPatchSchema>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub delivery: Option<CronDeliveryPatchSchema>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "failureAlert"
    )]
    pub failure_alert: Option<CronFailureAlertOption>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub state: Option<CronJobStatePatchSchema>,
}

/// `displayName | null` patch variant.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum CronNullableDisplayName {
    Value(CronDisplayNameSchema),
    Null,
}

/// `trigger | null` patch variant.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum CronNullableTrigger {
    Value(CronTriggerSchema),
    Null,
}

// ===========================================================================
// CronUpdateParamsSchema / CronRunParamsSchema
// ===========================================================================

/// Updates a cron job by id or legacy jobId alias.
/// 对齐 TS: `cronIdOrJobIdParams({ patch, expectedConfigRevision })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum CronUpdateParamsSchema {
    Id {
        #[serde(rename = "id")]
        id_value: NonEmptyString,
        patch: CronJobPatchSchema,
        #[serde(
            default,
            skip_serializing_if = "Option::is_none",
            rename = "expectedConfigRevision"
        )]
        expected_config_revision: Option<CronConfigRevisionSchema>,
    },
    JobId {
        #[serde(rename = "jobId")]
        job_id_value: NonEmptyString,
        patch: CronJobPatchSchema,
        #[serde(
            default,
            skip_serializing_if = "Option::is_none",
            rename = "expectedConfigRevision"
        )]
        expected_config_revision: Option<CronConfigRevisionSchema>,
    },
}

/// Run mode accepts `due` or `force`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum CronRunMode {
    Due,
    Force,
}

/// Runs a cron job immediately or only if due.
/// 对齐 TS: `cronIdOrJobIdParams({ mode })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum CronRunParamsSchema {
    Id {
        #[serde(rename = "id")]
        id_value: NonEmptyString,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        mode: Option<CronRunMode>,
    },
    JobId {
        #[serde(rename = "jobId")]
        job_id_value: NonEmptyString,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        mode: Option<CronRunMode>,
    },
}

// ===========================================================================
// CronRunsParamsSchema
// ===========================================================================

/// Query params for cron run history.
/// 对齐 TS: `Type.Object({...})`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CronRunsParamsSchema {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scope: Option<CronRunsScope>,
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "id")]
    pub id: Option<CronRunLogJobIdSchema>,
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "jobId")]
    pub job_id: Option<CronRunLogJobIdSchema>,
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "runId")]
    pub run_id: Option<NonEmptyString>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub limit: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub offset: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub statuses: Option<Vec<CronRunsStatusValueSchema>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status: Option<CronRunsStatusFilterSchema>,
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "deliveryStatuses")]
    pub delivery_statuses: Option<Vec<CronDeliveryStatusSchema>>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "deliveryStatus"
    )]
    pub delivery_status: Option<CronDeliveryStatusSchema>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub query: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "sortDir")]
    pub sort_dir: Option<CronSortDirSchema>,
}

/// `scope` accepts `job` or `all`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum CronRunsScope {
    Job,
    All,
}

impl CronRunsParamsSchema {
    pub fn validate(&self) -> Result<(), String> {
        if let Some(l) = self.limit {
            if l < CRON_LIST_LIMIT_MIN || l > CRON_LIST_LIMIT_MAX {
                return Err(format!(
                    "limit: expected [{}, {}], got {}",
                    CRON_LIST_LIMIT_MIN, CRON_LIST_LIMIT_MAX, l
                ));
            }
        }
        if let Some(o) = self.offset {
            if o < CRON_LIST_OFFSET_MIN {
                return Err(format!(
                    "offset: expected >= {}, got {}",
                    CRON_LIST_OFFSET_MIN, o
                ));
            }
        }
        if let Some(s) = &self.statuses {
            if s.len() < CRON_RUNS_STATUS_FILTER_MIN || s.len() > CRON_RUNS_STATUS_FILTER_MAX {
                return Err(format!(
                    "statuses: expected [{}, {}] items, got {}",
                    CRON_RUNS_STATUS_FILTER_MIN, CRON_RUNS_STATUS_FILTER_MAX,
                    s.len()
                ));
            }
        }
        if let Some(s) = &self.delivery_statuses {
            if s.len() < CRON_DELIVERY_STATUS_FILTER_MIN
                || s.len() > CRON_DELIVERY_STATUS_FILTER_MAX
            {
                return Err(format!(
                    "deliveryStatuses: expected [{}, {}] items, got {}",
                    CRON_DELIVERY_STATUS_FILTER_MIN, CRON_DELIVERY_STATUS_FILTER_MAX,
                    s.len()
                ));
            }
        }
        Ok(())
    }
}

// ===========================================================================
// CronRunLogEntrySchema
// ===========================================================================

/// LLM usage block attached to a cron run log entry.
/// 对齐 TS:
///   `Type.Object({
///       input_tokens:        Type.Optional(Type.Number()),
///       output_tokens:       Type.Optional(Type.Number()),
///       total_tokens:        Type.Optional(Type.Number()),
///       cache_read_tokens:   Type.Optional(Type.Number()),
///       cache_write_tokens:  Type.Optional(Type.Number()),
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct CronRunUsageSchema {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub input_tokens: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output_tokens: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub total_tokens: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cache_read_tokens: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cache_write_tokens: Option<f64>,
}

/// One persisted cron run history entry.
/// 对齐 TS: `Type.Object({...})`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CronRunLogEntrySchema {
    pub ts: i64,
    #[serde(rename = "jobId")]
    pub job_id: NonEmptyString,
    /// Literal `"finished"` discriminator used in run log entries.
    pub action: CronRunLogAction,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status: Option<CronRunStatusSchema>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "errorReason")]
    pub error_reason: Option<CronFailoverReasonSchema>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub diagnostics: Option<CronRunDiagnosticsSchema>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub delivered: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "deliveryStatus")]
    pub delivery_status: Option<CronDeliveryStatusSchema>,
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "deliveryError")]
    pub delivery_error: Option<String>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "failureNotificationDelivery"
    )]
    pub failure_notification_delivery: Option<CronFailureNotificationDeliverySchema>,
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "sessionId")]
    pub session_id: Option<NonEmptyString>,
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "sessionKey")]
    pub session_key: Option<NonEmptyString>,
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "runId")]
    pub run_id: Option<NonEmptyString>,
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "runAtMs")]
    pub run_at_ms: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "durationMs")]
    pub duration_ms: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "nextRunAtMs")]
    pub next_run_at_ms: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "triggerFired")]
    pub trigger_fired: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provider: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub usage: Option<CronRunUsageSchema>,
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "jobName")]
    pub job_name: Option<String>,
}

/// Literal `"finished"` discriminator for run log entries.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CronRunLogAction {
    #[serde(rename = "finished")]
    Finished,
}

impl CronRunLogEntrySchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_non_negative_integer("ts", self.ts)?;
        validate_non_empty_string("jobId", &self.job_id)?;
        if let Some(d) = &self.diagnostics {
            d.validate().map_err(|e| format!("diagnostics: {}", e))?;
        }
        Ok(())
    }
}

// Wire type aliases (对标 TS `type X = Static<typeof YSchema>`)
pub type CronJob = CronJobSchema;
pub type CronListParams = CronListParamsSchema;
pub type CronStatusParams = CronStatusParamsSchema;
pub type CronGetParams = CronGetParamsSchema;
pub type CronAddParams = CronAddParamsSchema;
pub type CronAddResult = CronAddResultSchema;
pub type CronDeclarativeAddResult = CronDeclarativeAddResultSchema;
pub type CronUpdateParams = CronUpdateParamsSchema;
pub type CronRemoveParams = CronRemoveParamsSchema;
pub type CronRunParams = CronRunParamsSchema;
pub type CronRunsParams = CronRunsParamsSchema;
pub type CronRunLogEntry = CronRunLogEntrySchema;