// Gateway Protocol schema: crestodian.
// 翻译自 packages/gateway-protocol/src/schema/crestodian.ts
//
// TS 用 TypeBox 定义 schema（运行时验证 + 类型）。
// Rust 用 serde struct + 验证函数实现等价的序列化/反序列化语义。

use serde::{Deserialize, Serialize};

use super::primitives::NonEmptyString;
use super::wizard::WizardStartResult;

// ---------- 基础验证原语 (对齐 TypeBox: NonEmptyString) ----------

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

// ---------- CrestodianChatWelcomeVariant ----------

/// Optional first-run greeting variant.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CrestodianChatWelcomeVariant {
    Onboarding,
}

impl CrestodianChatWelcomeVariant {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Onboarding => "onboarding",
        }
    }
}

pub fn is_valid_crestodian_chat_welcome_variant(variant: &str) -> bool {
    matches!(variant, "onboarding")
}

// ---------- CrestodianChatAction ----------

/// 闭枚举 Crestodian chat action。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum CrestodianChatAction {
    #[serde(rename = "none")]
    None,
    #[serde(rename = "open-agent")]
    OpenAgent,
    #[serde(rename = "exit")]
    Exit,
}

impl CrestodianChatAction {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::None => "none",
            Self::OpenAgent => "open-agent",
            Self::Exit => "exit",
        }
    }
}

pub fn is_valid_crestodian_chat_action(action: &str) -> bool {
    matches!(action, "none" | "open-agent" | "exit")
}

// ---------- CrestodianChatParamsSchema ----------

/// Crestodian chat 入参。
/// 对齐 TS:
///   `CrestodianChatParamsSchema = Type.Object({
///       sessionId: NonEmptyString,
///       message: Type.Optional(Type.String()),
///       welcomeVariant: Type.Optional(Type.Union([Type.Literal("onboarding")])),
///       reset: Type.Optional(Type.Boolean()),
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CrestodianChatParams {
    pub session_id: NonEmptyString,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub welcome_variant: Option<CrestodianChatWelcomeVariant>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reset: Option<bool>,
}

impl CrestodianChatParams {
    pub fn validate(&self) -> Result<(), String> {
        validate_non_empty_string("sessionId", &self.session_id)?;
        // `message` is optional plain string; no length constraint.
        Ok(())
    }
}

// ---------- CrestodianChatResultSchema ----------

/// 单条 Crestodian chat 回复。
/// 对齐 TS:
///   `CrestodianChatResultSchema = Type.Object({
///       sessionId: NonEmptyString,
///       reply: NonEmptyString,
///       sensitive: Type.Optional(Type.Boolean()),
///       action: Type.Union([
///           Type.Literal("none"),
///           Type.Literal("open-agent"),
///           Type.Literal("exit"),
///       ]),
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CrestodianChatResult {
    pub session_id: NonEmptyString,
    pub reply: NonEmptyString,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sensitive: Option<bool>,
    pub action: CrestodianChatAction,
}

impl CrestodianChatResult {
    pub fn validate(&self) -> Result<(), String> {
        validate_non_empty_string("sessionId", &self.session_id)?;
        validate_non_empty_string("reply", &self.reply)?;
        Ok(())
    }
}

// ---------- SetupInferenceKind ----------

/// 闭枚举 setup 候选 inference 路由类型。
/// 对齐 TS:
///   `SetupInferenceKind = Type.Union([
///       Type.Literal("existing-model"),
///       Type.Literal("openai-api-key"),
///       Type.Literal("anthropic-api-key"),
///       Type.Literal("claude-cli"),
///       Type.Literal("codex-cli"),
///       Type.Literal("gemini-cli"),
///   ])`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SetupInferenceKind {
    #[serde(rename = "existing-model")]
    ExistingModel,
    #[serde(rename = "openai-api-key")]
    OpenaiApiKey,
    #[serde(rename = "anthropic-api-key")]
    AnthropicApiKey,
    #[serde(rename = "claude-cli")]
    ClaudeCli,
    #[serde(rename = "codex-cli")]
    CodexCli,
    #[serde(rename = "gemini-cli")]
    GeminiCli,
}

impl SetupInferenceKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::ExistingModel => "existing-model",
            Self::OpenaiApiKey => "openai-api-key",
            Self::AnthropicApiKey => "anthropic-api-key",
            Self::ClaudeCli => "claude-cli",
            Self::CodexCli => "codex-cli",
            Self::GeminiCli => "gemini-cli",
        }
    }
}

pub fn is_valid_setup_inference_kind(kind: &str) -> bool {
    matches!(
        kind,
        "existing-model"
            | "openai-api-key"
            | "anthropic-api-key"
            | "claude-cli"
            | "codex-cli"
            | "gemini-cli"
    )
}

// ---------- SetupInferenceStatus ----------

/// 闭枚举 inference 路由状态（activate 成功/失败合集）。
/// 对齐 TS:
///   `SetupInferenceStatus = Type.Union([
///       Type.Literal("ok"), Type.Literal("auth"), Type.Literal("rate_limit"),
///       Type.Literal("billing"), Type.Literal("timeout"), Type.Literal("format"),
///       Type.Literal("unavailable"), Type.Literal("unknown"),
///   ])`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SetupInferenceStatus {
    #[serde(rename = "ok")]
    Ok,
    #[serde(rename = "auth")]
    Auth,
    #[serde(rename = "rate_limit")]
    RateLimit,
    #[serde(rename = "billing")]
    Billing,
    #[serde(rename = "timeout")]
    Timeout,
    #[serde(rename = "format")]
    Format,
    #[serde(rename = "unavailable")]
    Unavailable,
    #[serde(rename = "unknown")]
    Unknown,
}

impl SetupInferenceStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Ok => "ok",
            Self::Auth => "auth",
            Self::RateLimit => "rate_limit",
            Self::Billing => "billing",
            Self::Timeout => "timeout",
            Self::Format => "format",
            Self::Unavailable => "unavailable",
            Self::Unknown => "unknown",
        }
    }
}

pub fn is_valid_setup_inference_status(status: &str) -> bool {
    matches!(
        status,
        "ok"
            | "auth"
            | "rate_limit"
            | "billing"
            | "timeout"
            | "format"
            | "unavailable"
            | "unknown"
    )
}

// ---------- SetupInferenceFailureStatus ----------

/// 闭枚举 inference 路由失败状态（不含 ok）。
/// 对齐 TS:
///   `SetupInferenceFailureStatus = Type.Union([
///       Type.Literal("auth"), Type.Literal("rate_limit"),
///       Type.Literal("billing"), Type.Literal("timeout"), Type.Literal("format"),
///       Type.Literal("unavailable"), Type.Literal("unknown"),
///   ])`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SetupInferenceFailureStatus {
    #[serde(rename = "auth")]
    Auth,
    #[serde(rename = "rate_limit")]
    RateLimit,
    #[serde(rename = "billing")]
    Billing,
    #[serde(rename = "timeout")]
    Timeout,
    #[serde(rename = "format")]
    Format,
    #[serde(rename = "unavailable")]
    Unavailable,
    #[serde(rename = "unknown")]
    Unknown,
}

impl SetupInferenceFailureStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Auth => "auth",
            Self::RateLimit => "rate_limit",
            Self::Billing => "billing",
            Self::Timeout => "timeout",
            Self::Format => "format",
            Self::Unavailable => "unavailable",
            Self::Unknown => "unknown",
        }
    }
}

pub fn is_valid_setup_inference_failure_status(status: &str) -> bool {
    matches!(
        status,
        "auth"
            | "rate_limit"
            | "billing"
            | "timeout"
            | "format"
            | "unavailable"
            | "unknown"
    )
}

// ---------- CrestodianSetupDetectParamsSchema ----------

/// 空入参 detect 调用。
/// 对齐 TS:
///   `CrestodianSetupDetectParamsSchema = Type.Object({}, { additionalProperties: false })`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CrestodianSetupDetectParams {}

impl CrestodianSetupDetectParams {
    pub fn validate(&self) -> Result<(), String> {
        Ok(())
    }
}

// ---------- CrestodianSetupCandidate / Provider / AuthOption ----------

/// 一条 detect 出来的 inference 候选。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CrestodianSetupCandidate {
    pub kind: SetupInferenceKind,
    pub label: NonEmptyString,
    pub detail: String,
    pub model_ref: NonEmptyString,
    pub recommended: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub credentials: Option<bool>,
}

impl CrestodianSetupCandidate {
    pub fn validate(&self) -> Result<(), String> {
        validate_non_empty_string("label", &self.label)?;
        validate_non_empty_string("modelRef", &self.model_ref)?;
        Ok(())
    }
}

/// Gateway provider registry 暴露的文本推理 key/token 选项。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CrestodianSetupManualProvider {
    pub id: NonEmptyString,
    pub label: NonEmptyString,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hint: Option<String>,
}

impl CrestodianSetupManualProvider {
    pub fn validate(&self) -> Result<(), String> {
        validate_non_empty_string("id", &self.id)?;
        validate_non_empty_string("label", &self.label)?;
        Ok(())
    }
}

/// Auth-option kind discriminator.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum CrestodianSetupAuthOptionKind {
    #[serde(rename = "oauth")]
    Oauth,
    #[serde(rename = "device-code")]
    DeviceCode,
}

impl CrestodianSetupAuthOptionKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Oauth => "oauth",
            Self::DeviceCode => "device-code",
        }
    }
}

pub fn is_valid_crestodian_setup_auth_option_kind(kind: &str) -> bool {
    matches!(kind, "oauth" | "device-code")
}

/// Provider 自带的浏览器/device-code 登录方式。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CrestodianSetupAuthOption {
    pub id: NonEmptyString,
    pub label: NonEmptyString,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hint: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub group_label: Option<String>,
    pub kind: CrestodianSetupAuthOptionKind,
    pub featured: bool,
}

impl CrestodianSetupAuthOption {
    pub fn validate(&self) -> Result<(), String> {
        validate_non_empty_string("id", &self.id)?;
        validate_non_empty_string("label", &self.label)?;
        Ok(())
    }
}

// ---------- CrestodianSetupDetectResultSchema ----------

/// Detect 响应：候选 + 手动 provider + 可选 auth-options + 工作区上下文。
/// 对齐 TS:
///   `CrestodianSetupDetectResultSchema = Type.Object({
///       candidates: Type.Array(...),
///       manualProviders: Type.Array(...),
///       authOptions: Type.Optional(Type.Array(...)),
///       workspace: NonEmptyString,
///       codexAppServerDetected: Type.Optional(Type.Boolean()),
///       configuredModel: Type.Optional(Type.String()),
///       setupComplete: Type.Boolean(),
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CrestodianSetupDetectResult {
    pub candidates: Vec<CrestodianSetupCandidate>,
    pub manual_providers: Vec<CrestodianSetupManualProvider>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub auth_options: Option<Vec<CrestodianSetupAuthOption>>,
    pub workspace: NonEmptyString,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub codex_app_server_detected: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub configured_model: Option<String>,
    pub setup_complete: bool,
}

impl CrestodianSetupDetectResult {
    pub fn validate(&self) -> Result<(), String> {
        validate_non_empty_string("workspace", &self.workspace)?;
        for (i, c) in self.candidates.iter().enumerate() {
            c.validate().map_err(|e| format!("candidates[{}]: {}", i, e))?;
        }
        for (i, p) in self.manual_providers.iter().enumerate() {
            p.validate()
                .map_err(|e| format!("manualProviders[{}]: {}", i, e))?;
        }
        if let Some(opts) = &self.auth_options {
            for (i, opt) in opts.iter().enumerate() {
                opt.validate()
                    .map_err(|e| format!("authOptions[{}]: {}", i, e))?;
            }
        }
        Ok(())
    }
}

// ---------- CrestodianSetupVerifyParamsSchema ----------

/// 空入参 verify 调用。
/// 对齐 TS:
///   `CrestodianSetupVerifyParamsSchema = Type.Object({}, { additionalProperties: false })`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CrestodianSetupVerifyParams {}

impl CrestodianSetupVerifyParams {
    pub fn validate(&self) -> Result<(), String> {
        Ok(())
    }
}

// ---------- CrestodianSetupVerifyResultSchema ----------

/// Live 验证结果：`ok=true` 时携带 model/latency；`ok=false` 时携带 status/error。
/// 对齐 TS:
///   `CrestodianSetupVerifyResultSchema = Type.Union([
///       Type.Object({ ok: Type.Literal(true), modelRef: NonEmptyString, latencyMs: Type.Number() },
///                   { additionalProperties: false }),
///       Type.Object({ ok: Type.Literal(false), status: SetupInferenceFailureStatus, error: NonEmptyString },
///                   { additionalProperties: false }),
///   ])`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum CrestodianSetupVerifyResult {
    Success {
        ok: bool, // 序列化时为 true
        model_ref: NonEmptyString,
        latency_ms: f64,
    },
    Failure {
        ok: bool, // 序列化时为 false
        status: SetupInferenceFailureStatus,
        error: NonEmptyString,
    },
}

impl CrestodianSetupVerifyResult {
    pub fn validate(&self) -> Result<(), String> {
        match self {
            Self::Success {
                ok,
                model_ref,
                latency_ms,
            } => {
                if !*ok {
                    return Err("success variant requires ok=true".to_string());
                }
                validate_non_empty_string("modelRef", model_ref)?;
                if !latency_ms.is_finite() {
                    return Err(format!(
                        "latencyMs: expected finite number, got {}",
                        latency_ms
                    ));
                }
                Ok(())
            }
            Self::Failure { ok, status, error } => {
                if *ok {
                    return Err("failure variant requires ok=false".to_string());
                }
                validate_non_empty_string("error", error)?;
                let _ = status; // 闭合枚举由 serde 反序列化保证
                Ok(())
            }
        }
    }
}

// ---------- CrestodianSetupActivateKind ----------

/// 闭枚举 activate 入参 kind（额外包含 `api-key`）。
/// 对齐 TS:
///   `Type.Union([
///       Type.Literal("existing-model"),
///       Type.Literal("openai-api-key"),
///       Type.Literal("anthropic-api-key"),
///       Type.Literal("claude-cli"),
///       Type.Literal("codex-cli"),
///       Type.Literal("gemini-cli"),
///       Type.Literal("api-key"),
///   ])`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CrestodianSetupActivateKind {
    #[serde(rename = "existing-model")]
    ExistingModel,
    #[serde(rename = "openai-api-key")]
    OpenaiApiKey,
    #[serde(rename = "anthropic-api-key")]
    AnthropicApiKey,
    #[serde(rename = "claude-cli")]
    ClaudeCli,
    #[serde(rename = "codex-cli")]
    CodexCli,
    #[serde(rename = "gemini-cli")]
    GeminiCli,
    #[serde(rename = "api-key")]
    ApiKey,
}

impl CrestodianSetupActivateKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::ExistingModel => "existing-model",
            Self::OpenaiApiKey => "openai-api-key",
            Self::AnthropicApiKey => "anthropic-api-key",
            Self::ClaudeCli => "claude-cli",
            Self::CodexCli => "codex-cli",
            Self::GeminiCli => "gemini-cli",
            Self::ApiKey => "api-key",
        }
    }
}

pub fn is_valid_crestodian_setup_activate_kind(kind: &str) -> bool {
    matches!(
        kind,
        "existing-model"
            | "openai-api-key"
            | "anthropic-api-key"
            | "claude-cli"
            | "codex-cli"
            | "gemini-cli"
            | "api-key"
    )
}

// ---------- CrestodianSetupActivateParamsSchema ----------

/// Activate 入参。
/// 对齐 TS:
///   `CrestodianSetupActivateParamsSchema = Type.Object({
///       kind: ...,
///       modelRef: Type.Optional(NonEmptyString),
///       authChoice: Type.Optional(Type.String()),
///       apiKey: Type.Optional(Type.String()),
///       workspace: Type.Optional(Type.String()),
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CrestodianSetupActivateParams {
    pub kind: CrestodianSetupActivateKind,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model_ref: Option<NonEmptyString>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub auth_choice: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub api_key: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub workspace: Option<String>,
}

impl CrestodianSetupActivateParams {
    pub fn validate(&self) -> Result<(), String> {
        // `modelRef` if present must be non-empty; others are free-form strings.
        if let Some(m) = &self.model_ref {
            validate_non_empty_string("modelRef", m)?;
        }
        Ok(())
    }
}

// ---------- CrestodianSetupActivateResultSchema ----------

/// Activate 响应：成功路径带 modelRef/latency/lines；失败路径带 status/error。
/// 对齐 TS:
///   `CrestodianSetupActivateResultSchema = Type.Object({
///       ok: Type.Boolean(),
///       modelRef: Type.Optional(Type.String()),
///       latencyMs: Type.Optional(Type.Number()),
///       lines: Type.Optional(Type.Array(Type.String())),
///       status: Type.Optional(SetupInferenceStatus),
///       error: Type.Optional(Type.String()),
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CrestodianSetupActivateResult {
    pub ok: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model_ref: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub latency_ms: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lines: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status: Option<SetupInferenceStatus>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl CrestodianSetupActivateResult {
    pub fn validate(&self) -> Result<(), String> {
        if let Some(ms) = self.latency_ms {
            if !ms.is_finite() {
                return Err(format!("latencyMs: expected finite number, got {}", ms));
            }
        }
        Ok(())
    }
}

// ---------- CrestodianSetupAuthStartParamsSchema ----------

/// 启动一个 provider 拥有的交互式登录 wizard。
/// 对齐 TS:
///   `CrestodianSetupAuthStartParamsSchema = Type.Object({
///       sessionId: NonEmptyString,
///       authChoice: NonEmptyString,
///       workspace: Type.Optional(Type.String()),
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CrestodianSetupAuthStartParams {
    pub session_id: NonEmptyString,
    pub auth_choice: NonEmptyString,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub workspace: Option<String>,
}

impl CrestodianSetupAuthStartParams {
    pub fn validate(&self) -> Result<(), String> {
        validate_non_empty_string("sessionId", &self.session_id)?;
        validate_non_empty_string("authChoice", &self.auth_choice)?;
        Ok(())
    }
}

// ---------- CrestodianSetupAuthStartResultSchema ----------

/// `WizardStartResultSchema` 复用。
/// 对齐 TS: `CrestodianSetupAuthStartResultSchema = WizardStartResultSchema`.
pub type CrestodianSetupAuthStartResult = WizardStartResult;

// Wire types derive directly from local schema consts so public d.ts graphs
// never pull in the ProtocolSchemas registry.
// 对应 TS:
//   export type CrestodianChatParams = Static<typeof CrestodianChatParamsSchema>;
//   export type CrestodianChatResult = Static<typeof CrestodianChatResultSchema>;
//   export type CrestodianSetupDetectParams = Static<typeof CrestodianSetupDetectParamsSchema>;
//   export type CrestodianSetupDetectResult = Static<typeof CrestodianSetupDetectResultSchema>;
//   export type CrestodianSetupActivateParams = Static<typeof CrestodianSetupActivateParamsSchema>;
//   export type CrestodianSetupActivateResult = Static<typeof CrestodianSetupActivateResultSchema>;
//   export type CrestodianSetupVerifyParams = Static<typeof CrestodianSetupVerifyParamsSchema>;
//   export type CrestodianSetupVerifyResult = Static<typeof CrestodianSetupVerifyResultSchema>;
//   export type CrestodianSetupAuthStartParams = Static<typeof CrestodianSetupAuthStartParamsSchema>;
//   export type CrestodianSetupAuthStartResult = Static<typeof CrestodianSetupAuthStartResultSchema>;
pub type CrestodianChatParamsType = CrestodianChatParams;
pub type CrestodianChatResultType = CrestodianChatResult;
pub type CrestodianSetupDetectParamsType = CrestodianSetupDetectParams;
pub type CrestodianSetupDetectResultType = CrestodianSetupDetectResult;
pub type CrestodianSetupActivateParamsType = CrestodianSetupActivateParams;
pub type CrestodianSetupActivateResultType = CrestodianSetupActivateResult;
pub type CrestodianSetupVerifyParamsType = CrestodianSetupVerifyParams;
pub type CrestodianSetupVerifyResultType = CrestodianSetupVerifyResult;
pub type CrestodianSetupAuthStartParamsType = CrestodianSetupAuthStartParams;
pub type CrestodianSetupAuthStartResultType = CrestodianSetupAuthStartResult;