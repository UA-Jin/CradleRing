// Gateway Protocol schema: wizard.
// 翻译自 packages/gateway-protocol/src/schema/wizard.ts
//
// TS 用 TypeBox 定义 schema（运行时验证 + 类型）。
// Rust 用 serde struct + 验证函数实现等价的序列化/反序列化语义。

use serde::{Deserialize, Serialize};
use serde_json::Value;

// ---------- 基础验证原语 (对齐 TypeBox: NonEmptyString) ----------

/// 对齐 TS: `NonEmptyString = Type.String({ minLength: 1 })`.
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

// ---------- WizardRunStatus ----------

/// 闭枚举 wizard 会话运行时状态。
/// 对齐 TS:
/// ```ts
/// const WizardRunStatusSchema = Type.Union([
///   Type.Literal("running"),
///   Type.Literal("done"),
///   Type.Literal("cancelled"),
///   Type.Literal("error"),
/// ]);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum WizardRunStatus {
    Running,
    Done,
    Cancelled,
    Error,
}

impl WizardRunStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            WizardRunStatus::Running => "running",
            WizardRunStatus::Done => "done",
            WizardRunStatus::Cancelled => "cancelled",
            WizardRunStatus::Error => "error",
        }
    }
}

/// Returns true when `status` parses as a wizard run status literal.
pub fn is_valid_wizard_run_status(status: &str) -> bool {
    matches!(status, "running" | "done" | "cancelled" | "error")
}

// ---------- WizardStartParams ----------

/// 启动 setup wizard（可选 local/remote 工作区）。
/// 对齐 TS:
/// ```ts
/// export const WizardStartParamsSchema = Type.Object({
///   mode: Type.Optional(Type.Union([Type.Literal("local"), Type.Literal("remote")])),
///   workspace: Type.Optional(Type.String()),
/// }, { additionalProperties: false });
/// ```
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WizardStartParams {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mode: Option<WizardStartMode>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub workspace: Option<String>,
}

/// Optional wizard start mode.
/// 对齐 TS: `Type.Union([Type.Literal("local"), Type.Literal("remote")])`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum WizardStartMode {
    Local,
    Remote,
}

impl WizardStartMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            WizardStartMode::Local => "local",
            WizardStartMode::Remote => "remote",
        }
    }
}

/// Returns true when `mode` is one of the wizard start mode literals.
pub fn is_valid_wizard_start_mode(mode: &str) -> bool {
    matches!(mode, "local" | "remote")
}

impl WizardStartParams {
    pub fn validate(&self) -> Result<(), String> {
        // mode / workspace 均为可选字段，无需运行时约束。
        Ok(())
    }
}

// ---------- WizardAnswer ----------

/// 客户端对当前 wizard step 的回答载荷。
/// 对齐 TS:
/// ```ts
/// export const WizardAnswerSchema = Type.Object({
///   stepId: NonEmptyString,
///   value: Type.Optional(Type.Unknown()),
/// }, { additionalProperties: false });
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WizardAnswer {
    pub step_id: String,
    /// `serde_json::Value` 承载 TS `Type.Unknown()` 的任意 JSON 负载。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub value: Option<Value>,
}

impl WizardAnswer {
    pub fn validate(&self) -> Result<(), String> {
        validate_non_empty_string("stepId", &self.step_id)?;
        Ok(())
    }
}

// ---------- WizardNextParams ----------

/// 推进 wizard 会话，可选附带回答（若上一步要求输入）。
/// 对齐 TS:
/// ```ts
/// export const WizardNextParamsSchema = Type.Object({
///   sessionId: NonEmptyString,
///   answer: Type.Optional(WizardAnswerSchema),
/// }, { additionalProperties: false });
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WizardNextParams {
    pub session_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub answer: Option<WizardAnswer>,
}

impl WizardNextParams {
    pub fn validate(&self) -> Result<(), String> {
        validate_non_empty_string("sessionId", &self.session_id)?;
        if let Some(answer) = &self.answer {
            answer.validate().map_err(|e| format!("answer: {}", e))?;
        }
        Ok(())
    }
}

// ---------- WizardSessionIdParams（共享基类） ----------

/// cancel 与 status 共享的 sessionId 参数。
/// 对齐 TS:
/// ```ts
/// const WizardSessionIdParamsSchema = Type.Object({
///   sessionId: NonEmptyString,
/// }, { additionalProperties: false });
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WizardSessionIdParams {
    pub session_id: String,
}

impl WizardSessionIdParams {
    pub fn validate(&self) -> Result<(), String> {
        validate_non_empty_string("sessionId", &self.session_id)?;
        Ok(())
    }
}

// ---------- WizardCancelParams / WizardStatusParams ----------

/// 取消一个活动 wizard 会话。
/// 对齐 TS: `export const WizardCancelParamsSchema = WizardSessionIdParamsSchema;`.
pub type WizardCancelParams = WizardSessionIdParams;

/// 读取活动或近期完成 wizard 会话的状态。
/// 对齐 TS: `export const WizardStatusParamsSchema = WizardSessionIdParamsSchema;`.
pub type WizardStatusParams = WizardSessionIdParams;

// ---------- WizardStepOption ----------

/// choice-based wizard step 中可选项的值/标签。
/// 对齐 TS:
/// ```ts
/// export const WizardStepOptionSchema = Type.Object({
///   value: Type.Unknown(),
///   label: NonEmptyString,
///   hint: Type.Optional(Type.String()),
/// }, { additionalProperties: false });
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WizardStepOption {
    /// `serde_json::Value` 承载 TS `Type.Unknown()` 的任意 JSON 负载。
    pub value: Value,
    pub label: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hint: Option<String>,
}

impl WizardStepOption {
    pub fn validate(&self) -> Result<(), String> {
        validate_non_empty_string("label", &self.label)?;
        Ok(())
    }
}

// ---------- WizardDeviceCode ----------

/// wizard 步骤可能携带的设备码信息（OAuth 风格外部授权）。
/// 对齐 TS:
/// ```ts
/// const WizardDeviceCodeSchema = Type.Object({
///   code: NonEmptyString,
///   expiresInMinutes: Type.Optional(Type.Integer({ minimum: 1, maximum: 1440 })),
///   message: Type.Optional(Type.String()),
/// }, { additionalProperties: false });
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WizardDeviceCode {
    pub code: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expires_in_minutes: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

impl WizardDeviceCode {
    pub fn validate(&self) -> Result<(), String> {
        validate_non_empty_string("code", &self.code)?;
        if let Some(mins) = self.expires_in_minutes {
            if !(1..=1440).contains(&mins) {
                return Err(format!(
                    "expiresInMinutes: expected 1..=1440, got {}",
                    mins
                ));
            }
        }
        Ok(())
    }
}

// ---------- WizardStep ----------

/// wizard 步骤 UI 契约（gateway 客户端渲染单步时遵循）。
/// 对齐 TS:
/// ```ts
/// export const WizardStepSchema = Type.Object({
///   id: NonEmptyString,
///   type: Type.Union([
///     Type.Literal("note"),
///     Type.Literal("select"),
///     Type.Literal("text"),
///     Type.Literal("confirm"),
///     Type.Literal("multiselect"),
///     Type.Literal("progress"),
///     Type.Literal("action"),
///   ]),
///   title: Type.Optional(Type.String()),
///   message: Type.Optional(Type.String()),
///   format: Type.Optional(Type.Union([Type.Literal("plain")])),
///   options: Type.Optional(Type.Array(WizardStepOptionSchema)),
///   initialValue: Type.Optional(Type.Unknown()),
///   placeholder: Type.Optional(Type.String()),
///   sensitive: Type.Optional(Type.Boolean()),
///   executor: Type.Optional(Type.Union([Type.Literal("gateway"), Type.Literal("client")])),
///   externalUrl: Type.Optional(Type.String()),
///   deviceCode: Type.Optional(WizardDeviceCodeSchema),
/// }, { additionalProperties: false });
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WizardStep {
    pub id: String,
    #[serde(rename = "type")]
    pub step_type: WizardStepType,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub format: Option<WizardStepFormat>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub options: Option<Vec<WizardStepOption>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub initial_value: Option<Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub placeholder: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sensitive: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub executor: Option<WizardStepExecutor>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub external_url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub device_code: Option<WizardDeviceCode>,
}

/// 闭枚举 wizard 步骤类型。
/// 对齐 TS:
/// ```ts
/// Type.Union([
///   Type.Literal("note"),
///   Type.Literal("select"),
///   Type.Literal("text"),
///   Type.Literal("confirm"),
///   Type.Literal("multiselect"),
///   Type.Literal("progress"),
///   Type.Literal("action"),
/// ])
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum WizardStepType {
    Note,
    Select,
    Text,
    Confirm,
    Multiselect,
    Progress,
    Action,
}

impl WizardStepType {
    pub fn as_str(&self) -> &'static str {
        match self {
            WizardStepType::Note => "note",
            WizardStepType::Select => "select",
            WizardStepType::Text => "text",
            WizardStepType::Confirm => "confirm",
            WizardStepType::Multiselect => "multiselect",
            WizardStepType::Progress => "progress",
            WizardStepType::Action => "action",
        }
    }
}

/// Returns true when `kind` parses as a wizard step type literal.
pub fn is_valid_wizard_step_type(kind: &str) -> bool {
    matches!(
        kind,
        "note" | "select" | "text" | "confirm" | "multiselect" | "progress" | "action"
    )
}

/// wizard 步骤的可选内容格式。
/// 对齐 TS: `Type.Optional(Type.Union([Type.Literal("plain")]))`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum WizardStepFormat {
    Plain,
}

impl WizardStepFormat {
    pub fn as_str(&self) -> &'static str {
        match self {
            WizardStepFormat::Plain => "plain",
        }
    }
}

/// Returns true when `format` parses as a wizard step format literal.
pub fn is_valid_wizard_step_format(format: &str) -> bool {
    matches!(format, "plain")
}

/// wizard 步骤的执行者归属（gateway 端跑 vs. 客户端本地执行）。
/// 对齐 TS: `Type.Optional(Type.Union([Type.Literal("gateway"), Type.Literal("client")]))`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum WizardStepExecutor {
    Gateway,
    Client,
}

impl WizardStepExecutor {
    pub fn as_str(&self) -> &'static str {
        match self {
            WizardStepExecutor::Gateway => "gateway",
            WizardStepExecutor::Client => "client",
        }
    }
}

/// Returns true when `executor` parses as a wizard step executor literal.
pub fn is_valid_wizard_step_executor(executor: &str) -> bool {
    matches!(executor, "gateway" | "client")
}

impl WizardStep {
    pub fn validate(&self) -> Result<(), String> {
        validate_non_empty_string("id", &self.id)?;
        if let Some(options) = &self.options {
            for (i, opt) in options.iter().enumerate() {
                opt.validate().map_err(|e| format!("options[{}]: {}", i, e))?;
            }
        }
        if let Some(device_code) = &self.device_code {
            device_code
                .validate()
                .map_err(|e| format!("deviceCode: {}", e))?;
        }
        Ok(())
    }
}

// ---------- WizardNextResult / WizardStartResult ----------

/// start / next 共用的响应字段。
/// 对齐 TS:
/// ```ts
/// const WizardResultFields = {
///   done: Type.Boolean(),
///   step: Type.Optional(WizardStepSchema),
///   status: Type.Optional(WizardRunStatusSchema),
///   error: Type.Optional(Type.String()),
/// };
/// ```
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WizardResultFields {
    pub done: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub step: Option<WizardStep>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status: Option<WizardRunStatus>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl WizardResultFields {
    pub fn validate(&self) -> Result<(), String> {
        if let Some(step) = &self.step {
            step.validate().map_err(|e| format!("step: {}", e))?;
        }
        Ok(())
    }
}

/// 推进 wizard 会话后的响应。
/// 对齐 TS:
/// ```ts
/// export const WizardNextResultSchema = Type.Object(WizardResultFields, {
///   additionalProperties: false,
/// });
/// ```
pub type WizardNextResult = WizardResultFields;

/// wizard 会话创建时的响应（附加 sessionId）。
/// 对齐 TS:
/// ```ts
/// export const WizardStartResultSchema = Type.Object({
///   sessionId: NonEmptyString,
///   ...WizardResultFields,
/// }, { additionalProperties: false });
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WizardStartResult {
    pub session_id: String,
    #[serde(flatten)]
    pub fields: WizardResultFields,
}

impl WizardStartResult {
    pub fn validate(&self) -> Result<(), String> {
        validate_non_empty_string("sessionId", &self.session_id)?;
        self.fields.validate()?;
        Ok(())
    }
}

// ---------- WizardStatusResult ----------

/// 极简状态轮询响应（客户端无需下一步 step 时使用）。
/// 对齐 TS:
/// ```ts
/// export const WizardStatusResultSchema = Type.Object({
///   status: WizardRunStatusSchema,
///   error: Type.Optional(Type.String()),
/// }, { additionalProperties: false });
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WizardStatusResult {
    pub status: WizardRunStatus,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl WizardStatusResult {
    pub fn validate(&self) -> Result<(), String> {
        // status 是闭合枚举，serde 反序列化失败即非法；error 是可选字符串。
        Ok(())
    }
}

// Wire types derive directly from local schema consts so public d.ts graphs
// never pull in the ProtocolSchemas registry.
// 对应 TS:
//   export type WizardStartParams = Static<typeof WizardStartParamsSchema>;
//   export type WizardNextParams  = Static<typeof WizardNextParamsSchema>;
//   export type WizardCancelParams = Static<typeof WizardCancelParamsSchema>;
//   export type WizardStatusParams = Static<typeof WizardStatusParamsSchema>;
//   export type WizardStep        = Static<typeof WizardStepSchema>;
//   export type WizardNextResult  = Static<typeof WizardNextResultSchema>;
//   export type WizardStartResult = Static<typeof WizardStartResultSchema>;
//   export type WizardStatusResult = Static<typeof WizardStatusResultSchema>;
pub type WizardStartParamsType = WizardStartParams;
pub type WizardNextParamsType = WizardNextParams;
pub type WizardCancelParamsType = WizardCancelParams;
pub type WizardStatusParamsType = WizardStatusParams;
pub type WizardStepTypeAlias = WizardStep;
pub type WizardNextResultType = WizardNextResult;
pub type WizardStartResultType = WizardStartResult;
pub type WizardStatusResultType = WizardStatusResult;