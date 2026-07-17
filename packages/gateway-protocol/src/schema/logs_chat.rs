// Gateway Protocol schema: logs_chat.
// 翻译自 packages/gateway-protocol/src/schema/logs-chat.ts
//
// TS 用 TypeBox 定义 schema（运行时验证 + 类型）。
// Rust 用 serde struct + 验证函数实现等价的序列化/反序列化语义。

use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::primitives::{ChatSendSessionKeyString, InputProvenanceSchema, NonEmptyString};

// ---------- 基础验证原语 (对齐 TypeBox) ----------

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

fn validate_optional_non_empty_string(value: Option<&str>) -> Result<(), String> {
    if let Some(s) = value {
        validate_non_empty_string("optional non-empty", s)?;
    }
    Ok(())
}

// ---------- LogsTailParamsSchema ----------

/// Cursor-based request for the gateway log tail endpoint.
/// 对齐 TS:
/// ```ts
/// export const LogsTailParamsSchema = Type.Object(
///   {
///     cursor: Type.Optional(Type.Integer({ minimum: 0 })),
///     limit: Type.Optional(Type.Integer({ minimum: 1, maximum: 5000 })),
///     maxBytes: Type.Optional(Type.Integer({ minimum: 1, maximum: 1_000_000 })),
///   },
///   { additionalProperties: false },
/// );
/// ```
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LogsTailParamsSchema {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cursor: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub limit: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_bytes: Option<i64>,
}

impl LogsTailParamsSchema {
    pub fn validate(&self) -> Result<(), String> {
        if let Some(c) = self.cursor {
            if c < 0 {
                return Err(format!("cursor: expected integer >= 0, got {}", c));
            }
        }
        if let Some(l) = self.limit {
            if l < 1 || l > 5000 {
                return Err(format!(
                    "limit: expected integer in [1, 5000], got {}",
                    l
                ));
            }
        }
        if let Some(mb) = self.max_bytes {
            if mb < 1 || mb > 1_000_000 {
                return Err(format!(
                    "maxBytes: expected integer in [1, 1000000], got {}",
                    mb
                ));
            }
        }
        Ok(())
    }
}

// ---------- LogsTailResultSchema ----------

/// Gateway log tail payload returned to dashboard clients.
/// 对齐 TS:
/// ```ts
/// export const LogsTailResultSchema = Type.Object(
///   {
///     file: NonEmptyString,
///     cursor: Type.Integer({ minimum: 0 }),
///     size: Type.Integer({ minimum: 0 }),
///     lines: Type.Array(Type.String()),
///     truncated: Type.Optional(Type.Boolean()),
///     reset: Type.Optional(Type.Boolean()),
///   },
///   { additionalProperties: false },
/// );
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LogsTailResultSchema {
    pub file: NonEmptyString,
    pub cursor: i64,
    pub size: i64,
    pub lines: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub truncated: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reset: Option<bool>,
}

impl LogsTailResultSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_non_empty_string("file", &self.file)?;
        if self.cursor < 0 {
            return Err(format!("cursor: expected integer >= 0, got {}", self.cursor));
        }
        if self.size < 0 {
            return Err(format!("size: expected integer >= 0, got {}", self.size));
        }
        Ok(())
    }
}

// ---------- ChatHistoryParamsSchema ----------

/// Session-scoped history request used by WebChat and native WebSocket clients.
/// 对齐 TS:
/// ```ts
/// export const ChatHistoryParamsSchema = Type.Object(
///   {
///     sessionKey: NonEmptyString,
///     agentId: Type.Optional(NonEmptyString),
///     limit: Type.Optional(Type.Integer({ minimum: 1, maximum: 1000 })),
///     offset: Type.Optional(Type.Integer({ minimum: 0 })),
///     maxChars: Type.Optional(Type.Integer({ minimum: 1, maximum: 500_000 })),
///   },
///   { additionalProperties: false },
/// );
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatHistoryParamsSchema {
    pub session_key: NonEmptyString,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub agent_id: Option<NonEmptyString>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub limit: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub offset: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_chars: Option<i64>,
}

impl ChatHistoryParamsSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_non_empty_string("sessionKey", &self.session_key)?;
        validate_optional_non_empty_string(self.agent_id.as_deref())?;
        if let Some(l) = self.limit {
            if l < 1 || l > 1000 {
                return Err(format!(
                    "limit: expected integer in [1, 1000], got {}",
                    l
                ));
            }
        }
        if let Some(o) = self.offset {
            if o < 0 {
                return Err(format!("offset: expected integer >= 0, got {}", o));
            }
        }
        if let Some(mc) = self.max_chars {
            if mc < 1 || mc > 500_000 {
                return Err(format!(
                    "maxChars: expected integer in [1, 500000], got {}",
                    mc
                ));
            }
        }
        Ok(())
    }
}

// ---------- ChatMetadataParamsSchema ----------

/// Lightweight chat metadata request; optional agent scope keeps selector state explicit.
/// 对齐 TS:
/// ```ts
/// export const ChatMetadataParamsSchema = Type.Object(
///   {
///     agentId: Type.Optional(NonEmptyString),
///   },
///   { additionalProperties: false },
/// );
/// ```
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatMetadataParamsSchema {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub agent_id: Option<NonEmptyString>,
}

impl ChatMetadataParamsSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_optional_non_empty_string(self.agent_id.as_deref())?;
        Ok(())
    }
}

// ---------- ChatToolTitlesParamsSchema ----------

/// Batched purpose-title request for tool calls rendered in the Control UI.
/// 对齐 TS:
/// ```ts
/// export const ChatToolTitlesParamsSchema = Type.Object(
///   {
///     sessionKey: NonEmptyString,
///     agentId: Type.Optional(NonEmptyString),
///     items: Type.Array(
///       Type.Object(
///         {
///           id: Type.String({ minLength: 1, maxLength: 64 }),
///           name: Type.String({ minLength: 1, maxLength: 200 }),
///           input: Type.String({ minLength: 1, maxLength: 4_000 }),
///         },
///         { additionalProperties: false },
///       ),
///       { minItems: 1, maxItems: 24 },
///     ),
///   },
///   { additionalProperties: false },
/// );
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatToolTitlesParamsSchema {
    pub session_key: NonEmptyString,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub agent_id: Option<NonEmptyString>,
    pub items: Vec<ChatToolTitlesParamsItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatToolTitlesParamsItem {
    pub id: String,
    pub name: String,
    pub input: String,
}

impl ChatToolTitlesParamsSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_non_empty_string("sessionKey", &self.session_key)?;
        validate_optional_non_empty_string(self.agent_id.as_deref())?;
        if self.items.is_empty() || self.items.len() > 24 {
            return Err(format!(
                "items: expected array length in [1, 24], got {}",
                self.items.len()
            ));
        }
        for (i, item) in self.items.iter().enumerate() {
            let id_len = item.id.chars().count();
            if id_len < 1 || id_len > 64 {
                return Err(format!(
                    "items[{}].id: expected length in [1, 64], got {}",
                    i, id_len
                ));
            }
            let name_len = item.name.chars().count();
            if name_len < 1 || name_len > 200 {
                return Err(format!(
                    "items[{}].name: expected length in [1, 200], got {}",
                    i, name_len
                ));
            }
            let input_len = item.input.chars().count();
            if input_len < 1 || input_len > 4_000 {
                return Err(format!(
                    "items[{}].input: expected length in [1, 4000], got {}",
                    i, input_len
                ));
            }
        }
        Ok(())
    }
}

// ---------- ChatToolTitlesResultSchema ----------

/**
 * Titles keyed by the caller-provided item id; missing ids mean no title.
 * `disabled: true` tells clients the gateway has tool titles switched off so
 * they stop requesting for the rest of the session.
 * 对齐 TS:
 * ```ts
 * export const ChatToolTitlesResultSchema = Type.Object(
 *   {
 *     titles: Type.Record(Type.String(), Type.String()),
 *     disabled: Type.Optional(Type.Boolean()),
 *   },
 *   { additionalProperties: false },
 * );
 * ```
 */
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatToolTitlesResultSchema {
    pub titles: std::collections::BTreeMap<String, String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub disabled: Option<bool>,
}

impl ChatToolTitlesResultSchema {
    pub fn validate(&self) -> Result<(), String> {
        // titles is a free-form string → string map; nothing to constrain.
        Ok(())
    }
}

// ---------- ChatMessageGetParamsSchema ----------

/// Fetches one stored chat message without forcing history callers to request huge payloads.
/// 对齐 TS:
/// ```ts
/// export const ChatMessageGetParamsSchema = Type.Object(
///   {
///     sessionKey: NonEmptyString,
///     agentId: Type.Optional(NonEmptyString),
///     messageId: NonEmptyString,
///     maxChars: Type.Optional(Type.Integer({ minimum: 1, maximum: 2_000_000 })),
///   },
///   { additionalProperties: false },
/// );
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatMessageGetParamsSchema {
    pub session_key: NonEmptyString,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub agent_id: Option<NonEmptyString>,
    pub message_id: NonEmptyString,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_chars: Option<i64>,
}

impl ChatMessageGetParamsSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_non_empty_string("sessionKey", &self.session_key)?;
        validate_optional_non_empty_string(self.agent_id.as_deref())?;
        validate_non_empty_string("messageId", &self.message_id)?;
        if let Some(mc) = self.max_chars {
            if mc < 1 || mc > 2_000_000 {
                return Err(format!(
                    "maxChars: expected integer in [1, 2000000], got {}",
                    mc
                ));
            }
        }
        Ok(())
    }
}

// ---------- ChatMessageGetUnavailableReasonSchema ----------

/// Reason string returned when a single chat message lookup misses.
/// 对齐 TS:
///   `Type.Union([
///     Type.Literal("not_found"),
///     Type.Literal("oversized"),
///     Type.Literal("not_visible"),
///   ])`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ChatMessageGetUnavailableReasonSchema {
    #[serde(rename = "not_found")]
    NotFound,
    #[serde(rename = "oversized")]
    Oversized,
    #[serde(rename = "not_visible")]
    NotVisible,
}

impl ChatMessageGetUnavailableReasonSchema {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::NotFound => "not_found",
            Self::Oversized => "oversized",
            Self::NotVisible => "not_visible",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "not_found" => Some(Self::NotFound),
            "oversized" => Some(Self::Oversized),
            "not_visible" => Some(Self::NotVisible),
            _ => None,
        }
    }

    pub fn all() -> &'static [ChatMessageGetUnavailableReasonSchema] {
        &[Self::NotFound, Self::Oversized, Self::NotVisible]
    }
}

pub fn is_valid_chat_message_get_unavailable_reason(s: &str) -> bool {
    ChatMessageGetUnavailableReasonSchema::from_str(s).is_some()
}

// ---------- ChatMessageGetResultSchema ----------

/// Result envelope for single-message lookup, including the stable miss/visibility reason.
/// 对齐 TS:
/// ```ts
/// export const ChatMessageGetResultSchema = Type.Object(
///   {
///     ok: Type.Boolean(),
///     message: Type.Optional(Type.Unknown()),
///     unavailableReason: Type.Optional(
///       Type.Union([
///         Type.Literal("not_found"),
///         Type.Literal("oversized"),
///         Type.Literal("not_visible"),
///       ]),
///     ),
///   },
///   { additionalProperties: false },
/// );
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatMessageGetResultSchema {
    pub ok: bool,
    /// `serde_json::Value` 承载 TS `Type.Unknown()` 的任意 JSON 负载。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message: Option<Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub unavailable_reason: Option<ChatMessageGetUnavailableReasonSchema>,
}

impl ChatMessageGetResultSchema {
    pub fn validate(&self) -> Result<(), String> {
        if let Some(reason) = self.unavailable_reason {
            // Closed enum; serde rejects unknown values at deserialize time.
            if ChatMessageGetUnavailableReasonSchema::from_str(reason.as_str()).is_none() {
                return Err(format!(
                    "unavailableReason: invalid value: {}",
                    reason.as_str()
                ));
            }
        }
        Ok(())
    }
}

// ---------- ChatSendFastMode ----------

/// `fastMode` accepts either a boolean or the literal string `"auto"`.
/// 对齐 TS:
///   `Type.Optional(Type.Union([Type.Boolean(), Type.Literal("auto")]))`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ChatSendFastMode {
    Bool(bool),
    Auto,
}

impl ChatSendFastMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            ChatSendFastMode::Bool(b) => {
                if *b {
                    "true"
                } else {
                    "false"
                }
            }
            ChatSendFastMode::Auto => "auto",
        }
    }
}

// ---------- ChatSendParamsSchema ----------

/// User-to-agent send request; idempotency key lets clients safely retry transport failures.
/// 对齐 TS:
/// ```ts
/// export const ChatSendParamsSchema = Type.Object(
///   {
///     sessionKey: ChatSendSessionKeyString,
///     agentId: Type.Optional(NonEmptyString),
///     sessionId: Type.Optional(NonEmptyString),
///     message: Type.String(),
///     thinking: Type.Optional(Type.String()),
///     fastMode: Type.Optional(Type.Union([Type.Boolean(), Type.Literal("auto")])),
///     // One-turn override for auto fast-mode cutoff seconds.
///     fastAutoOnSeconds: Type.Optional(Type.Integer({ minimum: 1 })),
///     deliver: Type.Optional(Type.Boolean()),
///     originatingChannel: Type.Optional(Type.String()),
///     originatingTo: Type.Optional(Type.String()),
///     originatingAccountId: Type.Optional(Type.String()),
///     originatingThreadId: Type.Optional(Type.String()),
///     attachments: Type.Optional(Type.Array(Type.Unknown())),
///     timeoutMs: Type.Optional(Type.Integer({ minimum: 0 })),
///     systemInputProvenance: Type.Optional(InputProvenanceSchema),
///     systemProvenanceReceipt: Type.Optional(Type.String()),
///     suppressCommandInterpretation: Type.Optional(Type.Boolean()),
///     expectedSessionRoutingContract: Type.Optional(NonEmptyString),
///     idempotencyKey: NonEmptyString,
///   },
///   { additionalProperties: false },
/// );
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatSendParamsSchema {
    pub session_key: ChatSendSessionKeyString,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub agent_id: Option<NonEmptyString>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub session_id: Option<NonEmptyString>,
    pub message: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub thinking: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fast_mode: Option<ChatSendFastMode>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fast_auto_on_seconds: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub deliver: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub originating_channel: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub originating_to: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub originating_account_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub originating_thread_id: Option<String>,
    /// `Vec<serde_json::Value>` 承载 TS `Type.Array(Type.Unknown())` 的任意 JSON 负载。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub attachments: Option<Vec<Value>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub timeout_ms: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub system_input_provenance: Option<InputProvenanceSchema>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub system_provenance_receipt: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub suppress_command_interpretation: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expected_session_routing_contract: Option<NonEmptyString>,
    pub idempotency_key: NonEmptyString,
}

impl ChatSendParamsSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_non_empty_string("sessionKey", &self.session_key)?;
        validate_optional_non_empty_string(self.agent_id.as_deref())?;
        validate_optional_non_empty_string(self.session_id.as_deref())?;
        if let Some(faos) = self.fast_auto_on_seconds {
            if faos < 1 {
                return Err(format!(
                    "fastAutoOnSeconds: expected integer >= 1, got {}",
                    faos
                ));
            }
        }
        if let Some(t) = self.timeout_ms {
            if t < 0 {
                return Err(format!(
                    "timeoutMs: expected integer >= 0, got {}",
                    t
                ));
            }
        }
        if let Some(ip) = &self.system_input_provenance {
            ip.validate()
                .map_err(|e| format!("systemInputProvenance: {}", e))?;
        }
        validate_optional_non_empty_string(self.expected_session_routing_contract.as_deref())?;
        validate_non_empty_string("idempotencyKey", &self.idempotency_key)?;
        Ok(())
    }
}

// ---------- ChatAbortParamsSchema ----------

/// Cancels the active or named run for a chat session.
/// 对齐 TS:
/// ```ts
/// export const ChatAbortParamsSchema = Type.Object(
///   {
///     sessionKey: NonEmptyString,
///     agentId: Type.Optional(NonEmptyString),
///     runId: Type.Optional(NonEmptyString),
///     preserveSideRuns: Type.Optional(Type.Boolean()),
///   },
///   { additionalProperties: false },
/// );
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatAbortParamsSchema {
    pub session_key: NonEmptyString,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub agent_id: Option<NonEmptyString>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub run_id: Option<NonEmptyString>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub preserve_side_runs: Option<bool>,
}

impl ChatAbortParamsSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_non_empty_string("sessionKey", &self.session_key)?;
        validate_optional_non_empty_string(self.agent_id.as_deref())?;
        validate_optional_non_empty_string(self.run_id.as_deref())?;
        Ok(())
    }
}

// ---------- ChatInjectParamsSchema ----------

/// Inserts an operator-visible synthetic message into an existing chat transcript.
/// 对齐 TS:
/// ```ts
/// export const ChatInjectParamsSchema = Type.Object(
///   {
///     sessionKey: NonEmptyString,
///     agentId: Type.Optional(NonEmptyString),
///     message: NonEmptyString,
///     label: Type.Optional(Type.String({ maxLength: 100 })),
///   },
///   { additionalProperties: false },
/// );
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatInjectParamsSchema {
    pub session_key: NonEmptyString,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub agent_id: Option<NonEmptyString>,
    pub message: NonEmptyString,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
}

impl ChatInjectParamsSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_non_empty_string("sessionKey", &self.session_key)?;
        validate_optional_non_empty_string(self.agent_id.as_deref())?;
        validate_non_empty_string("message", &self.message)?;
        if let Some(l) = self.label.as_deref() {
            if l.chars().count() > 100 {
                return Err(format!(
                    "label: expected max length 100, got {}",
                    l.chars().count()
                ));
            }
        }
        Ok(())
    }
}

// ---------- ChatEventBaseSchema (private) ----------

/// Shared event fields preserve stream ordering and route events to the right session.
/// 对齐 TS:
/// ```ts
/// const ChatEventBaseSchema = {
///   runId: NonEmptyString,
///   sessionKey: NonEmptyString,
///   agentId: Type.Optional(NonEmptyString),
///   spawnedBy: Type.Optional(NonEmptyString),
///   seq: Type.Integer({ minimum: 0 }),
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatEventBaseSchema {
    pub run_id: NonEmptyString,
    pub session_key: NonEmptyString,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub agent_id: Option<NonEmptyString>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub spawned_by: Option<NonEmptyString>,
    pub seq: i64,
}

impl ChatEventBaseSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_non_empty_string("runId", &self.run_id)?;
        validate_non_empty_string("sessionKey", &self.session_key)?;
        validate_optional_non_empty_string(self.agent_id.as_deref())?;
        validate_optional_non_empty_string(self.spawned_by.as_deref())?;
        if self.seq < 0 {
            return Err(format!("seq: expected integer >= 0, got {}", self.seq));
        }
        Ok(())
    }
}

// ---------- ChatEventErrorKindSchema (private) ----------

/// Stable error categories exposed over the chat stream.
/// 对齐 TS:
///   `Type.Union([
///     Type.Literal("refusal"),
///     Type.Literal("timeout"),
///     Type.Literal("rate_limit"),
///     Type.Literal("context_length"),
///     Type.Literal("unknown"),
///   ])`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ChatEventErrorKindSchema {
    #[serde(rename = "refusal")]
    Refusal,
    #[serde(rename = "timeout")]
    Timeout,
    #[serde(rename = "rate_limit")]
    RateLimit,
    #[serde(rename = "context_length")]
    ContextLength,
    #[serde(rename = "unknown")]
    Unknown,
}

impl ChatEventErrorKindSchema {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Refusal => "refusal",
            Self::Timeout => "timeout",
            Self::RateLimit => "rate_limit",
            Self::ContextLength => "context_length",
            Self::Unknown => "unknown",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "refusal" => Some(Self::Refusal),
            "timeout" => Some(Self::Timeout),
            "rate_limit" => Some(Self::RateLimit),
            "context_length" => Some(Self::ContextLength),
            "unknown" => Some(Self::Unknown),
            _ => None,
        }
    }

    pub fn all() -> &'static [ChatEventErrorKindSchema] {
        &[
            Self::Refusal,
            Self::Timeout,
            Self::RateLimit,
            Self::ContextLength,
            Self::Unknown,
        ]
    }
}

pub fn is_valid_chat_event_error_kind(s: &str) -> bool {
    ChatEventErrorKindSchema::from_str(s).is_some()
}

// ---------- ChatDeltaEventSchema ----------

/// Incremental assistant output event; `replace` marks full-content refresh deltas.
/// 对齐 TS:
/// ```ts
/// export const ChatDeltaEventSchema = Type.Object(
///   {
///     ...ChatEventBaseSchema,
///     state: Type.Literal("delta"),
///     message: Type.Optional(Type.Unknown()),
///     deltaText: Type.String(),
///     replace: Type.Optional(Type.Boolean()),
///     usage: Type.Optional(Type.Unknown()),
///   },
///   { additionalProperties: false },
/// );
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatDeltaEventSchema {
    #[serde(flatten)]
    pub base: ChatEventBaseSchema,
    pub state: ChatDeltaEventState,
    /// `serde_json::Value` 承载 TS `Type.Unknown()` 的任意 JSON 负载。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message: Option<Value>,
    pub delta_text: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub replace: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub usage: Option<Value>,
}

/// Literal "delta" marker for `ChatDeltaEventSchema.state`.
/// 对齐 TS: `state: Type.Literal("delta")`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ChatDeltaEventState {
    #[serde(rename = "delta")]
    Delta,
}

impl Default for ChatDeltaEventState {
    fn default() -> Self {
        Self::Delta
    }
}

impl ChatDeltaEventSchema {
    pub fn validate(&self) -> Result<(), String> {
        self.base.validate()?;
        // state is a closed enum literal; serde rejects other values at deserialize time.
        Ok(())
    }
}

// ---------- ChatFinalEventSchema ----------

/// Successful terminal event for a completed chat run.
/// 对齐 TS:
/// ```ts
/// export const ChatFinalEventSchema = Type.Object(
///   {
///     ...ChatEventBaseSchema,
///     state: Type.Literal("final"),
///     message: Type.Optional(Type.Unknown()),
///     usage: Type.Optional(Type.Unknown()),
///     stopReason: Type.Optional(Type.String()),
///   },
///   { additionalProperties: false },
/// );
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatFinalEventSchema {
    #[serde(flatten)]
    pub base: ChatEventBaseSchema,
    pub state: ChatFinalEventState,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message: Option<Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub usage: Option<Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stop_reason: Option<String>,
}

/// Literal "final" marker for `ChatFinalEventSchema.state`.
/// 对齐 TS: `state: Type.Literal("final")`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ChatFinalEventState {
    #[serde(rename = "final")]
    Final,
}

impl Default for ChatFinalEventState {
    fn default() -> Self {
        Self::Final
    }
}

impl ChatFinalEventSchema {
    pub fn validate(&self) -> Result<(), String> {
        self.base.validate()
    }
}

// ---------- ChatAbortedEventSchema ----------

/// Terminal event for user-initiated or coordinator-initiated cancellation.
/// 对齐 TS:
/// ```ts
/// export const ChatAbortedEventSchema = Type.Object(
///   {
///     ...ChatEventBaseSchema,
///     state: Type.Literal("aborted"),
///     message: Type.Optional(Type.Unknown()),
///     errorMessage: Type.Optional(Type.String()),
///     stopReason: Type.Optional(Type.String()),
///   },
///   { additionalProperties: false },
/// );
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatAbortedEventSchema {
    #[serde(flatten)]
    pub base: ChatEventBaseSchema,
    pub state: ChatAbortedEventState,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message: Option<Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error_message: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stop_reason: Option<String>,
}

/// Literal "aborted" marker for `ChatAbortedEventSchema.state`.
/// 对齐 TS: `state: Type.Literal("aborted")`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ChatAbortedEventState {
    #[serde(rename = "aborted")]
    Aborted,
}

impl Default for ChatAbortedEventState {
    fn default() -> Self {
        Self::Aborted
    }
}

impl ChatAbortedEventSchema {
    pub fn validate(&self) -> Result<(), String> {
        self.base.validate()
    }
}

// ---------- ChatErrorEventSchema ----------

/// Terminal event for failed chat runs with an optional normalized failure kind.
/// 对齐 TS:
/// ```ts
/// export const ChatErrorEventSchema = Type.Object(
///   {
///     ...ChatEventBaseSchema,
///     state: Type.Literal("error"),
///     message: Type.Optional(Type.Unknown()),
///     errorMessage: Type.Optional(Type.String()),
///     errorKind: Type.Optional(ChatEventErrorKindSchema),
///     usage: Type.Optional(Type.Unknown()),
///     stopReason: Type.Optional(Type.String()),
///   },
///   { additionalProperties: false },
/// );
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatErrorEventSchema {
    #[serde(flatten)]
    pub base: ChatEventBaseSchema,
    pub state: ChatErrorEventState,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message: Option<Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error_message: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error_kind: Option<ChatEventErrorKindSchema>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub usage: Option<Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stop_reason: Option<String>,
}

/// Literal "error" marker for `ChatErrorEventSchema.state`.
/// 对齐 TS: `state: Type.Literal("error")`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ChatErrorEventState {
    #[serde(rename = "error")]
    Error,
}

impl Default for ChatErrorEventState {
    fn default() -> Self {
        Self::Error
    }
}

impl ChatErrorEventSchema {
    pub fn validate(&self) -> Result<(), String> {
        self.base.validate()
    }
}

// ---------- ChatEventSchema ----------

/// Public chat stream event union consumed by gateway protocol validators.
/// 对齐 TS:
///   `export const ChatEventSchema = Type.Union([
///     ChatDeltaEventSchema,
///     ChatFinalEventSchema,
///     ChatAbortedEventSchema,
///     ChatErrorEventSchema,
///   ])`.
///
/// Each variant carries its own `state` literal; serde can disambiguate via
/// the inner `state` field. Use `#[serde(untagged)]` to mirror the TS union's
/// untagged JSON-Schema shape.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ChatEventSchema {
    Delta(ChatDeltaEventSchema),
    Final(ChatFinalEventSchema),
    Aborted(ChatAbortedEventSchema),
    Error(ChatErrorEventSchema),
}

impl ChatEventSchema {
    pub fn validate(&self) -> Result<(), String> {
        match self {
            ChatEventSchema::Delta(e) => e.validate(),
            ChatEventSchema::Final(e) => e.validate(),
            ChatEventSchema::Aborted(e) => e.validate(),
            ChatEventSchema::Error(e) => e.validate(),
        }
    }
}

// Wire types derive directly from local schema consts so public d.ts graphs never
// pull in the ProtocolSchemas registry.
// 对应 TS:
//   export type ChatMetadataParams  = Static<typeof ChatMetadataParamsSchema>;
//   export type ChatToolTitlesParams = Static<typeof ChatToolTitlesParamsSchema>;
//   export type LogsTailParams      = Static<typeof LogsTailParamsSchema>;
//   export type LogsTailResult      = Static<typeof LogsTailResultSchema>;
//   export type ChatAbortParams     = Static<typeof ChatAbortParamsSchema>;
//   export type ChatInjectParams    = Static<typeof ChatInjectParamsSchema>;
//   export type ChatEvent           = Static<typeof ChatEventSchema>;
pub type ChatMetadataParams = ChatMetadataParamsSchema;
pub type ChatToolTitlesParams = ChatToolTitlesParamsSchema;
pub type LogsTailParams = LogsTailParamsSchema;
pub type LogsTailResult = LogsTailResultSchema;
pub type ChatAbortParams = ChatAbortParamsSchema;
pub type ChatInjectParams = ChatInjectParamsSchema;
pub type ChatEvent = ChatEventSchema;