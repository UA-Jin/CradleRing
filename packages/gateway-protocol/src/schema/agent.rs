// Gateway Protocol schema: agent.
// 翻译自 packages/gateway-protocol/src/schema/agent.ts
//
// Agent and channel-action gateway schemas.
//
// These payloads sit on the boundary between external channel adapters, gateway
// RPC callers, and the agent runtime. Keep public request fields documented
// because older CLI/channel clients may continue sending them across releases.
//
// TS 用 TypeBox 定义 schema（运行时验证 + 类型）。
// Rust 用 serde struct + 验证函数实现等价的序列化/反序列化语义。

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::primitives::{InputProvenanceSchema, SessionLabelString};

// ---------- 基础验证原语 (对齐 TypeBox: NonEmptyString / Integer) ----------

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

fn validate_optional_non_empty_string(field: &str, value: Option<&str>) -> Result<(), String> {
    if let Some(s) = value {
        validate_non_empty_string(field, s)?;
    }
    Ok(())
}

/// 对齐 TS: `Type.Integer({ minimum: 0 })`
fn validate_non_negative_integer(field: &str, n: i64) -> Result<(), String> {
    if n >= 0 {
        Ok(())
    } else {
        Err(format!(
            "{}: expected integer >= 0, got {}",
            field, n
        ))
    }
}

fn validate_optional_non_negative_integer(field: &str, n: Option<i64>) -> Result<(), String> {
    if let Some(v) = n {
        validate_non_negative_integer(field, v)?;
    }
    Ok(())
}

// ---------- Internal event constants ----------
// 对应 TS 顶层 const:
//   AGENT_INTERNAL_EVENT_TYPE_TASK_COMPLETION = "task_completion"
//   AGENT_INTERNAL_EVENT_SOURCES = [...]
//   AGENT_INTERNAL_EVENT_STATUSES = [...]

/// Discriminator literal for completion internal events.
pub const AGENT_INTERNAL_EVENT_TYPE_TASK_COMPLETION: &str = "task_completion";

/// Closed set of internal event source identifiers.
pub mod agent_internal_event_sources {
    pub const SUBAGENT: &str = "subagent";
    pub const CRON: &str = "cron";
    pub const IMAGE_GENERATION: &str = "image_generation";
    pub const VIDEO_GENERATION: &str = "video_generation";
    pub const MUSIC_GENERATION: &str = "music_generation";

    pub fn all() -> &'static [&'static str] {
        &[
            SUBAGENT,
            CRON,
            IMAGE_GENERATION,
            VIDEO_GENERATION,
            MUSIC_GENERATION,
        ]
    }

    pub fn from_str(s: &str) -> Option<&'static str> {
        match s {
            "subagent" => Some(SUBAGENT),
            "cron" => Some(CRON),
            "image_generation" => Some(IMAGE_GENERATION),
            "video_generation" => Some(VIDEO_GENERATION),
            "music_generation" => Some(MUSIC_GENERATION),
            _ => None,
        }
    }
}

/// Closed set of internal event status values.
pub mod agent_internal_event_statuses {
    pub const OK: &str = "ok";
    pub const TIMEOUT: &str = "timeout";
    pub const ERROR: &str = "error";
    pub const UNKNOWN: &str = "unknown";

    pub fn all() -> &'static [&'static str] {
        &[OK, TIMEOUT, ERROR, UNKNOWN]
    }

    pub fn from_str(s: &str) -> Option<&'static str> {
        match s {
            "ok" => Some(OK),
            "timeout" => Some(TIMEOUT),
            "error" => Some(ERROR),
            "unknown" => Some(UNKNOWN),
            _ => None,
        }
    }
}

pub fn is_valid_agent_internal_event_source(s: &str) -> bool {
    agent_internal_event_sources::from_str(s).is_some()
}

pub fn is_valid_agent_internal_event_status(s: &str) -> bool {
    agent_internal_event_statuses::from_str(s).is_some()
}

// ---------- Attachment type enum ----------

/// Closed set of generated attachment media types.
/// 对齐 TS: `Type.String({ enum: ["image", "audio", "video", "file"] })`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AgentGeneratedAttachmentType {
    Image,
    Audio,
    Video,
    File,
}

impl AgentGeneratedAttachmentType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Image => "image",
            Self::Audio => "audio",
            Self::Video => "video",
            Self::File => "file",
        }
    }
}

pub fn is_valid_agent_generated_attachment_type(s: &str) -> bool {
    matches!(
        s,
        "image" | "audio" | "video" | "file"
    )
}

// ---------- AgentGeneratedAttachmentSchema ----------

/// Generated media/file attachment metadata carried by internal agent events.
/// 对齐 TS:
///   `Type.Object({
///      type: Type.Optional(Type.String({ enum: ["image", "audio", "video", "file"] })),
///      path: Type.Optional(Type.String()),
///      url: Type.Optional(Type.String()),
///      mediaUrl: Type.Optional(Type.String()),
///      filePath: Type.Optional(Type.String()),
///      mimeType: Type.Optional(Type.String()),
///      name: Type.Optional(Type.String()),
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentGeneratedAttachmentSchema {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub r#type: Option<AgentGeneratedAttachmentType>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub media_url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file_path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mime_type: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

impl AgentGeneratedAttachmentSchema {
    pub fn validate(&self) -> Result<(), String> {
        // Optional fields; type is enum-checked at deserialize time via serde.
        // No nested objects to recurse.
        Ok(())
    }
}

// ---------- AgentInternalEventSchema ----------

/// Internal completion event surfaced when child automation reports back to a parent run.
/// 对齐 TS:
///   `Type.Object({
///      type: Type.Literal(AGENT_INTERNAL_EVENT_TYPE_TASK_COMPLETION),
///      source: Type.String({ enum: [...AGENT_INTERNAL_EVENT_SOURCES] }),
///      childSessionKey: Type.String(),
///      childSessionId: Type.Optional(Type.String()),
///      announceType: Type.String(),
///      taskLabel: Type.String(),
///      status: Type.String({ enum: [...AGENT_INTERNAL_EVENT_STATUSES] }),
///      statusLabel: Type.String(),
///      result: Type.String(),
///      attachments: Type.Optional(Type.Array(AgentGeneratedAttachmentSchema)),
///      mediaUrls: Type.Optional(Type.Array(Type.String())),
///      statsLine: Type.Optional(Type.String()),
///      replyInstruction: Type.String(),
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentInternalEventSchema {
    /// Discriminator literal: must equal `AGENT_INTERNAL_EVENT_TYPE_TASK_COMPLETION`.
    /// 对齐 TS: `Type.Literal(AGENT_INTERNAL_EVENT_TYPE_TASK_COMPLETION)`.
    #[serde(rename = "type")]
    pub r#type: String,
    pub source: String,
    pub child_session_key: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub child_session_id: Option<String>,
    pub announce_type: String,
    pub task_label: String,
    pub status: String,
    pub status_label: String,
    pub result: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub attachments: Option<Vec<AgentGeneratedAttachmentSchema>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub media_urls: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stats_line: Option<String>,
    pub reply_instruction: String,
}

impl AgentInternalEventSchema {
    pub fn validate(&self) -> Result<(), String> {
        // type discriminator
        if self.r#type != AGENT_INTERNAL_EVENT_TYPE_TASK_COMPLETION {
            return Err(format!(
                "type: expected {:?}, got {:?}",
                AGENT_INTERNAL_EVENT_TYPE_TASK_COMPLETION, self.r#type
            ));
        }
        if !is_valid_agent_internal_event_source(&self.source) {
            return Err(format!("source: invalid value {:?}", self.source));
        }
        if !is_valid_agent_internal_event_status(&self.status) {
            return Err(format!("status: invalid value {:?}", self.status));
        }
        validate_non_empty_string("childSessionKey", &self.child_session_key)?;
        validate_optional_non_empty_string("childSessionId", self.child_session_id.as_deref())?;
        validate_non_empty_string("announceType", &self.announce_type)?;
        validate_non_empty_string("taskLabel", &self.task_label)?;
        validate_non_empty_string("statusLabel", &self.status_label)?;
        validate_non_empty_string("result", &self.result)?;
        validate_non_empty_string("replyInstruction", &self.reply_instruction)?;
        if let Some(attachments) = &self.attachments {
            for (i, a) in attachments.iter().enumerate() {
                a.validate()
                    .map_err(|e| format!("attachments[{}]: {}", i, e))?;
            }
        }
        Ok(())
    }
}

// ---------- AgentEventSchema ----------

/// Stream event emitted by the agent runtime over the gateway protocol.
/// 对齐 TS:
///   `Type.Object({
///      runId: NonEmptyString,
///      seq: Type.Integer({ minimum: 0 }),
///      stream: NonEmptyString,
///      ts: Type.Integer({ minimum: 0 }),
///      spawnedBy: Type.Optional(NonEmptyString),
///      isHeartbeat: Type.Optional(Type.Boolean()),
///      data: Type.Record(Type.String(), Type.Unknown()),
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentEventSchema {
    pub run_id: String,
    pub seq: i64,
    pub stream: String,
    pub ts: i64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub spawned_by: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub is_heartbeat: Option<bool>,
    /// 对齐 TS: `Type.Record(Type.String(), Type.Unknown())` — opaque JSON object.
    pub data: BTreeMap<String, Value>,
}

impl AgentEventSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_non_empty_string("runId", &self.run_id)?;
        validate_non_empty_string("stream", &self.stream)?;
        validate_optional_non_empty_string("spawnedBy", self.spawned_by.as_deref())?;
        validate_non_negative_integer("seq", self.seq)?;
        validate_non_negative_integer("ts", self.ts)?;
        Ok(())
    }
}

// ---------- MessageActionToolContextSchema ----------

/// Caller-supplied routing hints. Authorization must use trusted runtime context.
/// 对齐 TS:
///   `Type.Object({
///      currentChannelId: Type.Optional(Type.String()),
///      currentMessagingTarget: Type.Optional(Type.String()),
///      currentGraphChannelId: Type.Optional(Type.String()),
///      currentChannelProvider: Type.Optional(Type.String()),
///      currentThreadTs: Type.Optional(Type.String()),
///      currentMessageId: Type.Optional(Type.Union([Type.String(), Type.Number()])),
///      replyToMode: Type.Optional(Type.Union([
///        Type.Literal("off"), Type.Literal("first"),
///        Type.Literal("all"), Type.Literal("batched"),
///      ])),
///      hasRepliedRef: Type.Optional(Type.Object({ value: Type.Boolean() }, { additionalProperties: false })),
///      sameChannelThreadRequired: Type.Optional(Type.Boolean()),
///      skipCrossContextDecoration: Type.Optional(Type.Boolean()),
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HasRepliedRef {
    pub value: bool,
}

impl HasRepliedRef {
    pub fn validate(&self) -> Result<(), String> {
        Ok(())
    }
}

/// Union literal for reply-to behavior in tool context.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ReplyToMode {
    #[serde(rename = "off")]
    Off,
    #[serde(rename = "first")]
    First,
    #[serde(rename = "all")]
    All,
    #[serde(rename = "batched")]
    Batched,
}

impl ReplyToMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Off => "off",
            Self::First => "first",
            Self::All => "all",
            Self::Batched => "batched",
        }
    }
}

pub fn is_valid_reply_to_mode(s: &str) -> bool {
    matches!(s, "off" | "first" | "all" | "batched")
}

/// `currentMessageId` accepts either a string or a number.
/// 对齐 TS: `Type.Union([Type.String(), Type.Number()])`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum CurrentMessageId {
    Text(String),
    Number(f64),
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MessageActionToolContextSchema {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub current_channel_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub current_messaging_target: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub current_graph_channel_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub current_channel_provider: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub current_thread_ts: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub current_message_id: Option<CurrentMessageId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reply_to_mode: Option<ReplyToMode>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub has_replied_ref: Option<HasRepliedRef>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub same_channel_thread_required: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub skip_cross_context_decoration: Option<bool>,
}

impl MessageActionToolContextSchema {
    pub fn validate(&self) -> Result<(), String> {
        if let Some(has_replied_ref) = &self.has_replied_ref {
            has_replied_ref.validate()?;
        }
        Ok(())
    }
}

// ---------- InboundTurnKind enum ----------

/// Inbound turn kinds accepted by `MessageActionParamsSchema`.
/// 对齐 TS: `Type.String({ enum: ["user_request", "room_event"] })`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InboundTurnKind {
    UserRequest,
    RoomEvent,
}

impl InboundTurnKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::UserRequest => "user_request",
            Self::RoomEvent => "room_event",
        }
    }
}

pub fn is_valid_inbound_turn_kind(s: &str) -> bool {
    matches!(s, "user_request" | "room_event")
}

// ---------- ConversationReadOrigin enum ----------

/// Explicit operation-local marker for an authenticated direct operator.
/// 对齐 TS: `Type.Literal("direct-operator")`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ConversationReadOrigin {
    #[serde(rename = "direct-operator")]
    DirectOperator,
}

impl ConversationReadOrigin {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::DirectOperator => "direct-operator",
        }
    }
}

pub fn is_valid_conversation_read_origin(s: &str) -> bool {
    s == "direct-operator"
}

// ---------- MessageActionParamsSchema ----------

/// Request to execute a channel message action through a configured adapter.
/// 对齐 TS:
///   `Type.Object({
///      channel: NonEmptyString,
///      action: NonEmptyString,
///      params: Type.Record(Type.String(), Type.Unknown()),
///      accountId: Type.Optional(Type.String()),
///      requesterAccountId: Type.Optional(Type.String()),
///      requesterSenderId: Type.Optional(Type.String()),
///      senderIsOwner: Type.Optional(Type.Boolean()),
///      sessionKey: Type.Optional(Type.String()),
///      sessionId: Type.Optional(Type.String()),
///      inboundTurnKind: Type.Optional(Type.String({ enum: ["user_request", "room_event"] })),
///      agentId: Type.Optional(Type.String()),
///      toolContext: Type.Optional(MessageActionToolContextSchema),
///      conversationReadOrigin: Type.Optional(Type.Literal("direct-operator")),
///      idempotencyKey: NonEmptyString,
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MessageActionParamsSchema {
    pub channel: String,
    pub action: String,
    /// 对齐 TS: `Type.Record(Type.String(), Type.Unknown())` — opaque JSON object.
    pub params: BTreeMap<String, Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub account_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub requester_account_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub requester_sender_id: Option<String>,
    /// Honored only when the RPC caller has the full operator scope set
    /// (shared-secret bearer or `operator.admin`). For narrowly-scoped
    /// callers (e.g. `operator.write`-only) the gateway forces this to
    /// `false` regardless of the value sent here.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sender_is_owner: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub session_key: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub inbound_turn_kind: Option<InboundTurnKind>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub agent_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool_context: Option<MessageActionToolContextSchema>,
    /// Explicit operation-local marker for an authenticated direct operator.
    /// Missing values remain delegated, and agent runtime identity wins server-side.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub conversation_read_origin: Option<ConversationReadOrigin>,
    pub idempotency_key: String,
}

impl MessageActionParamsSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_non_empty_string("channel", &self.channel)?;
        validate_non_empty_string("action", &self.action)?;
        validate_non_empty_string("idempotencyKey", &self.idempotency_key)?;
        if let Some(tool_context) = &self.tool_context {
            tool_context.validate()?;
        }
        Ok(())
    }
}

// ---------- SendParamsSchema ----------

/// Outbound send request shared by channel adapters.
/// 对齐 TS:
///   `Type.Object({
///      to: NonEmptyString,
///      message: Type.Optional(Type.String()),
///      mediaUrl: Type.Optional(Type.String()),
///      mediaUrls: Type.Optional(Type.Array(Type.String())),
///      buffer: Type.Optional(Type.String()),
///      filename: Type.Optional(Type.String()),
///      contentType: Type.Optional(Type.String()),
///      asVoice: Type.Optional(Type.Boolean()),
///      gifPlayback: Type.Optional(Type.Boolean()),
///      channel: Type.Optional(Type.String()),
///      accountId: Type.Optional(Type.String()),
///      agentId: Type.Optional(Type.String()),
///      replyToId: Type.Optional(Type.String()),
///      threadId: Type.Optional(Type.String()),
///      forceDocument: Type.Optional(Type.Boolean()),
///      silent: Type.Optional(Type.Boolean()),
///      parseMode: Type.Optional(Type.Literal("HTML")),
///      sessionKey: Type.Optional(Type.String()),
///      idempotencyKey: NonEmptyString,
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SendParamsSchema {
    pub to: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub media_url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub media_urls: Option<Vec<String>>,
    /// Base64 attachment payload for gateway-local media materialization.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub buffer: Option<String>,
    /// Optional filename for a base64 attachment payload.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub filename: Option<String>,
    /// Optional MIME type for a base64 attachment payload.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content_type: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub as_voice: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gif_playback: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub channel: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub account_id: Option<String>,
    /// Optional agent id for per-agent media root resolution on gateway sends.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub agent_id: Option<String>,
    /// Reply target message id for native quoted/threaded sends where supported.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reply_to_id: Option<String>,
    /// Thread id (channel-specific meaning, e.g. Telegram forum topic id).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub thread_id: Option<String>,
    /// Force document-style media sends where supported.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub force_document: Option<bool>,
    /// Send silently (no notification) where supported.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub silent: Option<bool>,
    /// Channel-specific parse mode for formatted text.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parse_mode: Option<ParseMode>,
    /// Optional session key for mirroring delivered output back into the transcript.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub session_key: Option<String>,
    pub idempotency_key: String,
}

/// Channel-specific parse mode for formatted text.
/// 对齐 TS: `Type.Literal("HTML")`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ParseMode {
    #[serde(rename = "HTML")]
    Html,
}

impl ParseMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Html => "HTML",
        }
    }
}

pub fn is_valid_parse_mode(s: &str) -> bool {
    s == "HTML"
}

impl SendParamsSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_non_empty_string("to", &self.to)?;
        validate_non_empty_string("idempotencyKey", &self.idempotency_key)?;
        Ok(())
    }
}

// ---------- PollParamsSchema ----------

/// Poll creation request for adapters that support native polls.
/// 对齐 TS:
///   `Type.Object({
///      to: NonEmptyString,
///      question: NonEmptyString,
///      options: Type.Array(NonEmptyString, { minItems: 2, maxItems: 12 }),
///      maxSelections: Type.Optional(Type.Integer({ minimum: 1, maximum: 12 })),
///      durationSeconds: Type.Optional(Type.Integer({ minimum: 1, maximum: 604_800 })),
///      durationHours: Type.Optional(Type.Integer({ minimum: 1 })),
///      silent: Type.Optional(Type.Boolean()),
///      isAnonymous: Type.Optional(Type.Boolean()),
///      threadId: Type.Optional(Type.String()),
///      channel: Type.Optional(Type.String()),
///      accountId: Type.Optional(Type.String()),
///      idempotencyKey: NonEmptyString,
///   }, { additionalProperties: false })`.
pub const POLL_OPTIONS_MIN_ITEMS: usize = 2;
pub const POLL_OPTIONS_MAX_ITEMS: usize = 12;
pub const POLL_MAX_SELECTIONS_MAX: i64 = 12;
pub const POLL_DURATION_SECONDS_MAX: i64 = 604_800;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PollParamsSchema {
    pub to: String,
    pub question: String,
    pub options: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_selections: Option<i64>,
    /// Poll duration in seconds (channel-specific limits may apply).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub duration_seconds: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub duration_hours: Option<i64>,
    /// Send silently (no notification) where supported.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub silent: Option<bool>,
    /// Poll anonymity where supported (e.g. Telegram polls default to anonymous).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub is_anonymous: Option<bool>,
    /// Thread id (channel-specific meaning, e.g. Telegram forum topic id).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub thread_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub channel: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub account_id: Option<String>,
    pub idempotency_key: String,
}

impl PollParamsSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_non_empty_string("to", &self.to)?;
        validate_non_empty_string("question", &self.question)?;
        validate_non_empty_string("idempotencyKey", &self.idempotency_key)?;
        if self.options.len() < POLL_OPTIONS_MIN_ITEMS || self.options.len() > POLL_OPTIONS_MAX_ITEMS {
            return Err(format!(
                "options: expected {}..={} items, got {}",
                POLL_OPTIONS_MIN_ITEMS,
                POLL_OPTIONS_MAX_ITEMS,
                self.options.len()
            ));
        }
        for (i, opt) in self.options.iter().enumerate() {
            validate_non_empty_string(&format!("options[{}]", i), opt)?;
        }
        if let Some(max) = self.max_selections {
            if !(1..=POLL_MAX_SELECTIONS_MAX).contains(&max) {
                return Err(format!(
                    "maxSelections: expected 1..={}, got {}",
                    POLL_MAX_SELECTIONS_MAX, max
                ));
            }
        }
        if let Some(d) = self.duration_seconds {
            if !(1..=POLL_DURATION_SECONDS_MAX).contains(&d) {
                return Err(format!(
                    "durationSeconds: expected 1..={}, got {}",
                    POLL_DURATION_SECONDS_MAX, d
                ));
            }
        }
        if let Some(d) = self.duration_hours {
            if d < 1 {
                return Err(format!("durationHours: expected >= 1, got {}", d));
            }
        }
        Ok(())
    }
}

// ---------- AgentParamsSchema enums ----------

/// Prompt mode accepted by `AgentParamsSchema`.
/// 对齐 TS: `Type.Union([Type.Literal("full"), Type.Literal("minimal"), Type.Literal("none")])`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AgentPromptMode {
    Full,
    Minimal,
    None,
}

impl AgentPromptMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Full => "full",
            Self::Minimal => "minimal",
            Self::None => "none",
        }
    }
}

pub fn is_valid_agent_prompt_mode(s: &str) -> bool {
    matches!(s, "full" | "minimal" | "none")
}

/// Bootstrap context mode accepted by `AgentParamsSchema`.
/// 对齐 TS: `Type.Union([Type.Literal("full"), Type.Literal("lightweight")])`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BootstrapContextMode {
    Full,
    Lightweight,
}

impl BootstrapContextMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Full => "full",
            Self::Lightweight => "lightweight",
        }
    }
}

pub fn is_valid_bootstrap_context_mode(s: &str) -> bool {
    matches!(s, "full" | "lightweight")
}

/// Bootstrap context run kind accepted by `AgentParamsSchema`.
/// 对齐 TS: `Type.Union([Type.Literal("default"), Type.Literal("heartbeat"), Type.Literal("cron")])`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BootstrapContextRunKind {
    Default,
    Heartbeat,
    Cron,
}

impl BootstrapContextRunKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Default => "default",
            Self::Heartbeat => "heartbeat",
            Self::Cron => "cron",
        }
    }
}

pub fn is_valid_bootstrap_context_run_kind(s: &str) -> bool {
    matches!(s, "default" | "heartbeat" | "cron")
}

/// ACP turn source accepted by `AgentParamsSchema`.
/// 对齐 TS: `Type.Literal("manual_spawn")`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AcpTurnSource {
    ManualSpawn,
}

impl AcpTurnSource {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::ManualSpawn => "manual_spawn",
        }
    }
}

pub fn is_valid_acp_turn_source(s: &str) -> bool {
    s == "manual_spawn"
}

/// Session effect mode accepted by `AgentParamsSchema`.
/// 对齐 TS: `Type.Union([Type.Literal("visible"), Type.Literal("internal")])`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SessionEffectsMode {
    Visible,
    Internal,
}

impl SessionEffectsMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Visible => "visible",
            Self::Internal => "internal",
        }
    }
}

pub fn is_valid_session_effects_mode(s: &str) -> bool {
    matches!(s, "visible" | "internal")
}

/// Source reply delivery mode accepted by `AgentParamsSchema`.
/// 对齐 TS: `Type.Union([Type.Literal("automatic"), Type.Literal("message_tool_only")])`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceReplyDeliveryMode {
    Automatic,
    MessageToolOnly,
}

impl SourceReplyDeliveryMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Automatic => "automatic",
            Self::MessageToolOnly => "message_tool_only",
        }
    }
}

pub fn is_valid_source_reply_delivery_mode(s: &str) -> bool {
    matches!(s, "automatic" | "message_tool_only")
}

// ---------- AgentParamsSchema ----------

/// Main agent-run request accepted by the gateway.
/// 对齐 TS:
///   `Type.Object({
///      message: NonEmptyString,
///      agentId: Type.Optional(NonEmptyString),
///      provider: Type.Optional(Type.String()),
///      model: Type.Optional(Type.String()),
///      to: Type.Optional(Type.String()),
///      replyTo: Type.Optional(Type.String()),
///      sessionId: Type.Optional(Type.String()),
///      sessionKey: Type.Optional(Type.String()),
///      thinking: Type.Optional(Type.String()),
///      deliver: Type.Optional(Type.Boolean()),
///      attachments: Type.Optional(Type.Array(Type.Unknown())),
///      channel: Type.Optional(Type.String()),
///      replyChannel: Type.Optional(Type.String()),
///      accountId: Type.Optional(Type.String()),
///      replyAccountId: Type.Optional(Type.String()),
///      threadId: Type.Optional(Type.String()),
///      groupId: Type.Optional(Type.String()),
///      groupChannel: Type.Optional(Type.String()),
///      groupSpace: Type.Optional(Type.String()),
///      timeout: Type.Optional(Type.Integer({ minimum: 0 })),
///      bestEffortDeliver: Type.Optional(Type.Boolean()),
///      lane: Type.Optional(Type.String()),
///      cwd: Type.Optional(NonEmptyString),
///      cleanupBundleMcpOnRunEnd: Type.Optional(Type.Boolean()),
///      modelRun: Type.Optional(Type.Boolean()),
///      promptMode: Type.Optional(Type.Union([Type.Literal("full"), Type.Literal("minimal"), Type.Literal("none")])),
///      extraSystemPrompt: Type.Optional(Type.String()),
///      bootstrapContextMode: Type.Optional(Type.Union([Type.Literal("full"), Type.Literal("lightweight")])),
///      bootstrapContextRunKind: Type.Optional(Type.Union([Type.Literal("default"), Type.Literal("heartbeat"), Type.Literal("cron")])),
///      acpTurnSource: Type.Optional(Type.Literal("manual_spawn")),
///      internalRuntimeHandoffId: Type.Optional(NonEmptyString),
///      execApprovalFollowupExpectedSessionId: Type.Optional(NonEmptyString),
///      internalEvents: Type.Optional(Type.Array(AgentInternalEventSchema)),
///      inputProvenance: Type.Optional(InputProvenanceSchema),
///      suppressPromptPersistence: Type.Optional(Type.Boolean()),
///      sessionEffects: Type.Optional(Type.Union([Type.Literal("visible"), Type.Literal("internal")])),
///      sourceReplyDeliveryMode: Type.Optional(Type.Union([Type.Literal("automatic"), Type.Literal("message_tool_only")])),
///      disableMessageTool: Type.Optional(Type.Boolean()),
///      forceRestartSafeTools: Type.Optional(Type.Boolean()),
///      voiceWakeTrigger: Type.Optional(Type.String()),
///      idempotencyKey: NonEmptyString,
///      label: Type.Optional(SessionLabelString),
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentParamsSchema {
    pub message: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub agent_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provider: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub to: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reply_to: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub session_key: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub thinking: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub deliver: Option<bool>,
    /// 对齐 TS: `Type.Array(Type.Unknown())` — opaque JSON array.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub attachments: Option<Vec<Value>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub channel: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reply_channel: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub account_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reply_account_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub thread_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub group_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub group_channel: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub group_space: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub timeout: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub best_effort_deliver: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lane: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cwd: Option<String>,
    /// One-shot CLI gateway requests can ask the gateway to close process-wide
    /// bundle MCP resources after the run instead of keeping them warm.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cleanup_bundle_mcp_on_run_end: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model_run: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prompt_mode: Option<AgentPromptMode>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub extra_system_prompt: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bootstrap_context_mode: Option<BootstrapContextMode>,
    /// Commitment fan-out scope is scheduler-internal and cannot be selected over Gateway RPC.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bootstrap_context_run_kind: Option<BootstrapContextRunKind>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub acp_turn_source: Option<AcpTurnSource>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub internal_runtime_handoff_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub exec_approval_followup_expected_session_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub internal_events: Option<Vec<AgentInternalEventSchema>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub input_provenance: Option<InputProvenanceSchema>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub suppress_prompt_persistence: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub session_effects: Option<SessionEffectsMode>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_reply_delivery_mode: Option<SourceReplyDeliveryMode>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub disable_message_tool: Option<bool>,
    /// Host-owned recovery turns can force every Code Mode exec onto the
    /// restart-safe path even if the model omits or clears the tool argument.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub force_restart_safe_tools: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub voice_wake_trigger: Option<String>,
    pub idempotency_key: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<SessionLabelString>,
}

impl AgentParamsSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_non_empty_string("message", &self.message)?;
        validate_non_empty_string("idempotencyKey", &self.idempotency_key)?;
        validate_optional_non_empty_string("agentId", self.agent_id.as_deref())?;
        validate_optional_non_empty_string("cwd", self.cwd.as_deref())?;
        validate_optional_non_empty_string("internalRuntimeHandoffId", self.internal_runtime_handoff_id.as_deref())?;
        validate_optional_non_empty_string(
            "execApprovalFollowupExpectedSessionId",
            self.exec_approval_followup_expected_session_id.as_deref(),
        )?;
        validate_optional_non_negative_integer("timeout", self.timeout)?;
        if let Some(events) = &self.internal_events {
            for (i, e) in events.iter().enumerate() {
                e.validate()
                    .map_err(|e| format!("internalEvents[{}]: {}", i, e))?;
            }
        }
        if let Some(input_provenance) = &self.input_provenance {
            input_provenance
                .validate()
                .map_err(|e| format!("inputProvenance: {}", e))?;
        }
        if let Some(label) = &self.label {
            if label.is_empty() {
                return Err("label: expected non-empty session label".to_string());
            }
        }
        Ok(())
    }
}

// ---------- AgentIdentityParamsSchema ----------

/// Identity lookup request for the current or selected agent/session.
/// 对齐 TS:
///   `Type.Object({
///      agentId: Type.Optional(NonEmptyString),
///      sessionKey: Type.Optional(Type.String()),
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentIdentityParamsSchema {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub agent_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub session_key: Option<String>,
}

impl AgentIdentityParamsSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_optional_non_empty_string("agentId", self.agent_id.as_deref())?;
        Ok(())
    }
}

// ---------- AgentIdentityResultSchema ----------

/// Avatar status enum returned by `AgentIdentityResultSchema`.
/// 对齐 TS: `Type.String({ enum: ["none", "local", "remote", "data"] })`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AgentAvatarStatus {
    None,
    Local,
    Remote,
    Data,
}

impl AgentAvatarStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Local => "local",
            Self::Remote => "remote",
            Self::Data => "data",
        }
    }
}

pub fn is_valid_agent_avatar_status(s: &str) -> bool {
    matches!(s, "none" | "local" | "remote" | "data")
}

/// Public display identity returned for an agent.
/// 对齐 TS:
///   `Type.Object({
///      agentId: NonEmptyString,
///      name: Type.Optional(NonEmptyString),
///      avatar: Type.Optional(NonEmptyString),
///      avatarSource: Type.Optional(NonEmptyString),
///      avatarStatus: Type.Optional(Type.String({ enum: ["none", "local", "remote", "data"] })),
///      avatarReason: Type.Optional(NonEmptyString),
///      emoji: Type.Optional(NonEmptyString),
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentIdentityResultSchema {
    pub agent_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub avatar: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub avatar_source: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub avatar_status: Option<AgentAvatarStatus>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub avatar_reason: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub emoji: Option<String>,
}

impl AgentIdentityResultSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_non_empty_string("agentId", &self.agent_id)?;
        Ok(())
    }
}

// ---------- AgentWaitParamsSchema ----------

/// Waits for a submitted agent run to complete or time out.
/// 对齐 TS:
///   `Type.Object({
///      runId: NonEmptyString,
///      timeoutMs: Type.Optional(Type.Integer({ minimum: 0 })),
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentWaitParamsSchema {
    pub run_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub timeout_ms: Option<i64>,
}

impl AgentWaitParamsSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_non_empty_string("runId", &self.run_id)?;
        validate_optional_non_negative_integer("timeoutMs", self.timeout_ms)?;
        Ok(())
    }
}

// ---------- WakeParamsSchema ----------

/// Wake mode accepted by `WakeParamsSchema`.
/// 对齐 TS: `Type.Union([Type.Literal("now"), Type.Literal("next-heartbeat")])`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum WakeMode {
    Now,
    #[serde(rename = "next-heartbeat")]
    NextHeartbeat,
}

impl WakeMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Now => "now",
            Self::NextHeartbeat => "next-heartbeat",
        }
    }
}

pub fn is_valid_wake_mode(s: &str) -> bool {
    matches!(s, "now" | "next-heartbeat")
}

/// Wake request from external schedulers or devices into an agent session.
/// 对齐 TS:
///   `Type.Object({
///      mode: Type.Union([Type.Literal("now"), Type.Literal("next-heartbeat")]),
///      text: NonEmptyString,
///      sessionKey: Type.Optional(NonEmptyString),
///      agentId: Type.Optional(NonEmptyString),
///   }, { additionalProperties: true })` — external wake senders may attach opaque metadata.
///
/// Typed field; misspelled variants remain opaque metadata because wake senders
/// already rely on additionalProperties.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WakeParamsSchema {
    pub mode: WakeMode,
    pub text: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub session_key: Option<String>,
    /// Optional agent id paired with `sessionKey`. Routes multi-agent setups
    /// to the agent that owns the targeted session — closes the related half
    /// of #46886 ("always routes to default agent").
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub agent_id: Option<String>,
    /// 对齐 TS `additionalProperties: true` —— external wake senders may attach opaque metadata.
    #[serde(flatten)]
    pub additional_properties: BTreeMap<String, Value>,
}

impl WakeParamsSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_non_empty_string("text", &self.text)?;
        validate_optional_non_empty_string("sessionKey", self.session_key.as_deref())?;
        validate_optional_non_empty_string("agentId", self.agent_id.as_deref())?;
        Ok(())
    }
}

// ============================================================
// Wire types
// 对应 TS:
//   export type AgentEvent = Static<typeof AgentEventSchema>;
//   export type AgentIdentityParams = Static<typeof AgentIdentityParamsSchema>;
//   export type AgentIdentityResult = Static<typeof AgentIdentityResultSchema>;
//   export type MessageActionParams = Static<typeof MessageActionParamsSchema>;
//   export type PollParams = Static<typeof PollParamsSchema>;
//   export type AgentWaitParams = Static<typeof AgentWaitParamsSchema>;
//   export type WakeParams = Static<typeof WakeParamsSchema>;
// Wire types derive directly from local schema consts so public d.ts graphs never
// pull in the ProtocolSchemas registry.
// ============================================================

pub type AgentEvent = AgentEventSchema;
pub type AgentIdentityParams = AgentIdentityParamsSchema;
pub type AgentIdentityResult = AgentIdentityResultSchema;
pub type MessageActionParams = MessageActionParamsSchema;
pub type PollParams = PollParamsSchema;
pub type AgentWaitParams = AgentWaitParamsSchema;
pub type WakeParams = WakeParamsSchema;