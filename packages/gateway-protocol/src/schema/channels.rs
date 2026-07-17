// Gateway Protocol schema: channels.
// 翻译自 packages/gateway-protocol/src/schema/channels.ts
//
// Channel and Talk protocol schemas.
//
// Talk schemas are consumed by browser realtime clients, gateway relay sessions,
// and channel adapters, so the mode/transport/brain unions below are shared
// API vocabulary rather than provider-local implementation details.
//
// TS 用 TypeBox 定义 schema（运行时验证 + 类型）。
// Rust 用 serde struct + 验证函数实现等价的序列化/反序列化语义。

use std::collections::{BTreeMap, HashMap};

use serde::{Deserialize, Serialize};

use super::primitives::SecretInputSchema;

// ============================================================================
// 基础验证原语
// ============================================================================

/// 对齐 TS: `NonEmptyString = Type.String({ minLength: 1 })`.
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

/// 对齐 TS: `Type.Integer({ minimum: 0 })`.
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

/// 对齐 TS: `Type.Integer({ minimum: 1 })`.
fn validate_positive_integer(field: &str, n: i64) -> Result<(), String> {
    if n >= 1 {
        Ok(())
    } else {
        Err(format!(
            "{}: expected integer >= 1, got {}",
            field, n
        ))
    }
}

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

/// 对齐 TS: `Type.Number({ minimum: 0, maximum: 1 })`.
fn validate_number_in_range(field: &str, n: f64, min: f64, max: f64) -> Result<(), String> {
    if n.is_nan() || n < min || n > max {
        return Err(format!(
            "{}: expected {}..={}, got {}",
            field, min, max, n
        ));
    }
    Ok(())
}

fn validate_optional_number_in_range(
    field: &str,
    n: Option<f64>,
    min: f64,
    max: f64,
) -> Result<(), String> {
    if let Some(v) = n {
        validate_number_in_range(field, v, min, max)?;
    }
    Ok(())
}

fn validate_optional_string_min_length(
    field: &str,
    value: Option<&str>,
    min: usize,
) -> Result<(), String> {
    if let Some(s) = value {
        let len = s.chars().count();
        if len < min {
            return Err(format!(
                "{}: expected length >= {}, got {}",
                field, min, len
            ));
        }
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

fn regex(pattern: &str) -> regex::Regex {
    regex::Regex::new(pattern).expect("invalid regex pattern compiled into channels")
}

// ============================================================================
// Module-private constants
// ============================================================================

/// QR data URL length bound: `Type.String({ maxLength: 16_384, pattern: "^data:image/png;base64," })`.
const QR_DATA_URL_MAX_LENGTH: usize = 16_384;
const QR_DATA_URL_PATTERN: &str = r"^data:image/png;base64,";

/// `TalkSessionCreateParamsSchema.ttlMs` integer range.
const TALK_SESSION_TTL_MS_MIN: i64 = 1000;
const TALK_SESSION_TTL_MS_MAX: i64 = 3_600_000;

/// `TalkRealtimeConfigSchema.vadThreshold` number range (`Type.Number({ minimum: 0, maximum: 1 })`).
const TALK_VAD_THRESHOLD_MIN: f64 = 0.0;
const TALK_VAD_THRESHOLD_MAX: f64 = 1.0;

// ============================================================================
// Enums (private schema unions)
// ============================================================================

// ---------- TalkModeSchema ----------

/// Talk session shape discriminator.
/// 对齐 TS: `Type.Union([
///   Type.Literal("realtime"),
///   Type.Literal("stt-tts"),
///   Type.Literal("transcription"),
/// ])`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TalkModeSchema {
    #[serde(rename = "realtime")]
    Realtime,
    #[serde(rename = "stt-tts")]
    SttTts,
    #[serde(rename = "transcription")]
    Transcription,
}

impl TalkModeSchema {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Realtime => "realtime",
            Self::SttTts => "stt-tts",
            Self::Transcription => "transcription",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "realtime" => Some(Self::Realtime),
            "stt-tts" => Some(Self::SttTts),
            "transcription" => Some(Self::Transcription),
            _ => None,
        }
    }

    pub fn all() -> &'static [TalkModeSchema] {
        &[Self::Realtime, Self::SttTts, Self::Transcription]
    }
}

pub fn is_valid_talk_mode(s: &str) -> bool {
    TalkModeSchema::from_str(s).is_some()
}

// ---------- TalkTransportSchema ----------

/// Transport family discriminator; browser clients branch on this value.
/// 对齐 TS: `Type.Union([
///   Type.Literal("webrtc"),
///   Type.Literal("provider-websocket"),
///   Type.Literal("gateway-relay"),
///   Type.Literal("managed-room"),
/// ])`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TalkTransportSchema {
    #[serde(rename = "webrtc")]
    Webrtc,
    #[serde(rename = "provider-websocket")]
    ProviderWebsocket,
    #[serde(rename = "gateway-relay")]
    GatewayRelay,
    #[serde(rename = "managed-room")]
    ManagedRoom,
}

impl TalkTransportSchema {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Webrtc => "webrtc",
            Self::ProviderWebsocket => "provider-websocket",
            Self::GatewayRelay => "gateway-relay",
            Self::ManagedRoom => "managed-room",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "webrtc" => Some(Self::Webrtc),
            "provider-websocket" => Some(Self::ProviderWebsocket),
            "gateway-relay" => Some(Self::GatewayRelay),
            "managed-room" => Some(Self::ManagedRoom),
            _ => None,
        }
    }

    pub fn all() -> &'static [TalkTransportSchema] {
        &[
            Self::Webrtc,
            Self::ProviderWebsocket,
            Self::GatewayRelay,
            Self::ManagedRoom,
        ]
    }
}

pub fn is_valid_talk_transport(s: &str) -> bool {
    TalkTransportSchema::from_str(s).is_some()
}

// ---------- TalkBrainSchema ----------

/// How a Talk session delegates reasoning/tool use to the agent runtime.
/// 对齐 TS: `Type.Union([
///   Type.Literal("agent-consult"),
///   Type.Literal("direct-tools"),
///   Type.Literal("none"),
/// ])`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TalkBrainSchema {
    #[serde(rename = "agent-consult")]
    AgentConsult,
    #[serde(rename = "direct-tools")]
    DirectTools,
    #[serde(rename = "none")]
    None,
}

impl TalkBrainSchema {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::AgentConsult => "agent-consult",
            Self::DirectTools => "direct-tools",
            Self::None => "none",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "agent-consult" => Some(Self::AgentConsult),
            "direct-tools" => Some(Self::DirectTools),
            "none" => Some(Self::None),
            _ => None,
        }
    }

    pub fn all() -> &'static [TalkBrainSchema] {
        &[Self::AgentConsult, Self::DirectTools, Self::None]
    }
}

pub fn is_valid_talk_brain(s: &str) -> bool {
    TalkBrainSchema::from_str(s).is_some()
}

// ---------- TalkAgentControlModeSchema ----------

/// Agent control actions accepted from Talk clients and managed rooms.
/// 对齐 TS: `Type.Union([
///   Type.Literal("status"),
///   Type.Literal("steer"),
///   Type.Literal("cancel"),
///   Type.Literal("followup"),
/// ])`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TalkAgentControlModeSchema {
    #[serde(rename = "status")]
    Status,
    #[serde(rename = "steer")]
    Steer,
    #[serde(rename = "cancel")]
    Cancel,
    #[serde(rename = "followup")]
    Followup,
}

impl TalkAgentControlModeSchema {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Status => "status",
            Self::Steer => "steer",
            Self::Cancel => "cancel",
            Self::Followup => "followup",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "status" => Some(Self::Status),
            "steer" => Some(Self::Steer),
            "cancel" => Some(Self::Cancel),
            "followup" => Some(Self::Followup),
            _ => None,
        }
    }

    pub fn all() -> &'static [TalkAgentControlModeSchema] {
        &[Self::Status, Self::Steer, Self::Cancel, Self::Followup]
    }
}

pub fn is_valid_talk_agent_control_mode(s: &str) -> bool {
    TalkAgentControlModeSchema::from_str(s).is_some()
}

// ---------- TalkEventTypeSchema ----------

/// Stable event names emitted by Talk sessions across providers/transports.
/// 对齐 TS: `Type.Union([...])` (26 literals).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TalkEventTypeSchema {
    #[serde(rename = "session.started")]
    SessionStarted,
    #[serde(rename = "session.ready")]
    SessionReady,
    #[serde(rename = "session.closed")]
    SessionClosed,
    #[serde(rename = "session.error")]
    SessionError,
    #[serde(rename = "session.replaced")]
    SessionReplaced,
    #[serde(rename = "turn.started")]
    TurnStarted,
    #[serde(rename = "turn.ended")]
    TurnEnded,
    #[serde(rename = "turn.cancelled")]
    TurnCancelled,
    #[serde(rename = "capture.started")]
    CaptureStarted,
    #[serde(rename = "capture.stopped")]
    CaptureStopped,
    #[serde(rename = "capture.cancelled")]
    CaptureCancelled,
    #[serde(rename = "capture.once")]
    CaptureOnce,
    #[serde(rename = "input.audio.delta")]
    InputAudioDelta,
    #[serde(rename = "input.audio.committed")]
    InputAudioCommitted,
    #[serde(rename = "transcript.delta")]
    TranscriptDelta,
    #[serde(rename = "transcript.done")]
    TranscriptDone,
    #[serde(rename = "output.text.delta")]
    OutputTextDelta,
    #[serde(rename = "output.text.done")]
    OutputTextDone,
    #[serde(rename = "output.audio.started")]
    OutputAudioStarted,
    #[serde(rename = "output.audio.delta")]
    OutputAudioDelta,
    #[serde(rename = "output.audio.done")]
    OutputAudioDone,
    #[serde(rename = "tool.call")]
    ToolCall,
    #[serde(rename = "tool.progress")]
    ToolProgress,
    #[serde(rename = "tool.result")]
    ToolResult,
    #[serde(rename = "tool.error")]
    ToolError,
    #[serde(rename = "usage.metrics")]
    UsageMetrics,
    #[serde(rename = "latency.metrics")]
    LatencyMetrics,
    #[serde(rename = "health.changed")]
    HealthChanged,
}

impl TalkEventTypeSchema {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::SessionStarted => "session.started",
            Self::SessionReady => "session.ready",
            Self::SessionClosed => "session.closed",
            Self::SessionError => "session.error",
            Self::SessionReplaced => "session.replaced",
            Self::TurnStarted => "turn.started",
            Self::TurnEnded => "turn.ended",
            Self::TurnCancelled => "turn.cancelled",
            Self::CaptureStarted => "capture.started",
            Self::CaptureStopped => "capture.stopped",
            Self::CaptureCancelled => "capture.cancelled",
            Self::CaptureOnce => "capture.once",
            Self::InputAudioDelta => "input.audio.delta",
            Self::InputAudioCommitted => "input.audio.committed",
            Self::TranscriptDelta => "transcript.delta",
            Self::TranscriptDone => "transcript.done",
            Self::OutputTextDelta => "output.text.delta",
            Self::OutputTextDone => "output.text.done",
            Self::OutputAudioStarted => "output.audio.started",
            Self::OutputAudioDelta => "output.audio.delta",
            Self::OutputAudioDone => "output.audio.done",
            Self::ToolCall => "tool.call",
            Self::ToolProgress => "tool.progress",
            Self::ToolResult => "tool.result",
            Self::ToolError => "tool.error",
            Self::UsageMetrics => "usage.metrics",
            Self::LatencyMetrics => "latency.metrics",
            Self::HealthChanged => "health.changed",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        Some(match s {
            "session.started" => Self::SessionStarted,
            "session.ready" => Self::SessionReady,
            "session.closed" => Self::SessionClosed,
            "session.error" => Self::SessionError,
            "session.replaced" => Self::SessionReplaced,
            "turn.started" => Self::TurnStarted,
            "turn.ended" => Self::TurnEnded,
            "turn.cancelled" => Self::TurnCancelled,
            "capture.started" => Self::CaptureStarted,
            "capture.stopped" => Self::CaptureStopped,
            "capture.cancelled" => Self::CaptureCancelled,
            "capture.once" => Self::CaptureOnce,
            "input.audio.delta" => Self::InputAudioDelta,
            "input.audio.committed" => Self::InputAudioCommitted,
            "transcript.delta" => Self::TranscriptDelta,
            "transcript.done" => Self::TranscriptDone,
            "output.text.delta" => Self::OutputTextDelta,
            "output.text.done" => Self::OutputTextDone,
            "output.audio.started" => Self::OutputAudioStarted,
            "output.audio.delta" => Self::OutputAudioDelta,
            "output.audio.done" => Self::OutputAudioDone,
            "tool.call" => Self::ToolCall,
            "tool.progress" => Self::ToolProgress,
            "tool.result" => Self::ToolResult,
            "tool.error" => Self::ToolError,
            "usage.metrics" => Self::UsageMetrics,
            "latency.metrics" => Self::LatencyMetrics,
            "health.changed" => Self::HealthChanged,
            _ => return None,
        })
    }

    pub fn all() -> &'static [TalkEventTypeSchema] {
        &[
            Self::SessionStarted,
            Self::SessionReady,
            Self::SessionClosed,
            Self::SessionError,
            Self::SessionReplaced,
            Self::TurnStarted,
            Self::TurnEnded,
            Self::TurnCancelled,
            Self::CaptureStarted,
            Self::CaptureStopped,
            Self::CaptureCancelled,
            Self::CaptureOnce,
            Self::InputAudioDelta,
            Self::InputAudioCommitted,
            Self::TranscriptDelta,
            Self::TranscriptDone,
            Self::OutputTextDelta,
            Self::OutputTextDone,
            Self::OutputAudioStarted,
            Self::OutputAudioDelta,
            Self::OutputAudioDone,
            Self::ToolCall,
            Self::ToolProgress,
            Self::ToolResult,
            Self::ToolError,
            Self::UsageMetrics,
            Self::LatencyMetrics,
            Self::HealthChanged,
        ]
    }
}

pub fn is_valid_talk_event_type(s: &str) -> bool {
    TalkEventTypeSchema::from_str(s).is_some()
}

/// Event types that must carry a turn id for client-side stream correlation.
/// 对齐 TS: `TURN_SCOPED_TALK_EVENT_TYPES`.
pub const TURN_SCOPED_TALK_EVENT_TYPES: &[TalkEventTypeSchema] = &[
    TalkEventTypeSchema::TurnStarted,
    TalkEventTypeSchema::TurnEnded,
    TalkEventTypeSchema::TurnCancelled,
    TalkEventTypeSchema::InputAudioDelta,
    TalkEventTypeSchema::InputAudioCommitted,
    TalkEventTypeSchema::TranscriptDelta,
    TalkEventTypeSchema::TranscriptDone,
    TalkEventTypeSchema::OutputTextDelta,
    TalkEventTypeSchema::OutputTextDone,
    TalkEventTypeSchema::OutputAudioStarted,
    TalkEventTypeSchema::OutputAudioDelta,
    TalkEventTypeSchema::OutputAudioDone,
    TalkEventTypeSchema::ToolCall,
    TalkEventTypeSchema::ToolProgress,
    TalkEventTypeSchema::ToolResult,
    TalkEventTypeSchema::ToolError,
];

/// Capture lifecycle events must include capture id to avoid cross-turn ambiguity.
/// 对齐 TS: `CAPTURE_SCOPED_TALK_EVENT_TYPES`.
pub const CAPTURE_SCOPED_TALK_EVENT_TYPES: &[TalkEventTypeSchema] = &[
    TalkEventTypeSchema::CaptureStarted,
    TalkEventTypeSchema::CaptureStopped,
    TalkEventTypeSchema::CaptureCancelled,
    TalkEventTypeSchema::CaptureOnce,
];

pub fn is_turn_scoped_talk_event_type(t: TalkEventTypeSchema) -> bool {
    TURN_SCOPED_TALK_EVENT_TYPES.contains(&t)
}

pub fn is_capture_scoped_talk_event_type(t: TalkEventTypeSchema) -> bool {
    CAPTURE_SCOPED_TALK_EVENT_TYPES.contains(&t)
}

// ---------- TalkAgentControlTargetSchema ----------

/// Embedded vs reply-bound run target discriminator for agent control results.
/// 对齐 TS: `Type.Union([Type.Literal("embedded_run"), Type.Literal("reply_run")])`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TalkAgentControlTargetSchema {
    #[serde(rename = "embedded_run")]
    EmbeddedRun,
    #[serde(rename = "reply_run")]
    ReplyRun,
}

impl TalkAgentControlTargetSchema {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::EmbeddedRun => "embedded_run",
            Self::ReplyRun => "reply_run",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "embedded_run" => Some(Self::EmbeddedRun),
            "reply_run" => Some(Self::ReplyRun),
            _ => None,
        }
    }
}

pub fn is_valid_talk_agent_control_target(s: &str) -> bool {
    TalkAgentControlTargetSchema::from_str(s).is_some()
}

// ---------- TalkProviderCancelledStatusSchema ----------

/// `TalkAgentControlResultSchema.providerResult.status` is a closed literal.
/// 对齐 TS: `Type.Literal("cancelled")`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TalkProviderCancelledStatusSchema {
    #[serde(rename = "cancelled")]
    Cancelled,
}

impl TalkProviderCancelledStatusSchema {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Cancelled => "cancelled",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "cancelled" => Some(Self::Cancelled),
            _ => None,
        }
    }
}

pub fn is_valid_talk_provider_cancelled_status(s: &str) -> bool {
    TalkProviderCancelledStatusSchema::from_str(s).is_some()
}

// ---------- BrowserRealtimeEncodingSchema ----------

/// Audio encoding literal accepted by browser realtime audio contracts.
/// 对齐 TS: `Type.Union([Type.Literal("pcm16"), Type.Literal("g711_ulaw")])`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BrowserRealtimeEncodingSchema {
    #[serde(rename = "pcm16")]
    Pcm16,
    #[serde(rename = "g711_ulaw")]
    G711Ulaw,
}

impl BrowserRealtimeEncodingSchema {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Pcm16 => "pcm16",
            Self::G711Ulaw => "g711_ulaw",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "pcm16" => Some(Self::Pcm16),
            "g711_ulaw" => Some(Self::G711Ulaw),
            _ => None,
        }
    }
}

pub fn is_valid_browser_realtime_encoding(s: &str) -> bool {
    BrowserRealtimeEncodingSchema::from_str(s).is_some()
}

// ---------- TalkConsultRoutingSchema ----------

/// Routing discriminator for `agent-consult` brain mode.
/// 对齐 TS: `Type.Union([Type.Literal("provider-direct"), Type.Literal("force-agent-consult")])`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TalkConsultRoutingSchema {
    #[serde(rename = "provider-direct")]
    ProviderDirect,
    #[serde(rename = "force-agent-consult")]
    ForceAgentConsult,
}

impl TalkConsultRoutingSchema {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::ProviderDirect => "provider-direct",
            Self::ForceAgentConsult => "force-agent-consult",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "provider-direct" => Some(Self::ProviderDirect),
            "force-agent-consult" => Some(Self::ForceAgentConsult),
            _ => None,
        }
    }
}

pub fn is_valid_talk_consult_routing(s: &str) -> bool {
    TalkConsultRoutingSchema::from_str(s).is_some()
}

// ---------- ChannelEventLoopReasonSchema ----------

/// Reason the event loop is considered degraded.
/// 对齐 TS: `Type.Union([
///   Type.Literal("event_loop_delay"),
///   Type.Literal("event_loop_utilization"),
///   Type.Literal("cpu"),
/// ])`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChannelEventLoopReasonSchema {
    EventLoopDelay,
    EventLoopUtilization,
    Cpu,
}

impl ChannelEventLoopReasonSchema {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::EventLoopDelay => "event_loop_delay",
            Self::EventLoopUtilization => "event_loop_utilization",
            Self::Cpu => "cpu",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "event_loop_delay" => Some(Self::EventLoopDelay),
            "event_loop_utilization" => Some(Self::EventLoopUtilization),
            "cpu" => Some(Self::Cpu),
            _ => None,
        }
    }
}

pub fn is_valid_channel_event_loop_reason(s: &str) -> bool {
    ChannelEventLoopReasonSchema::from_str(s).is_some()
}

// ============================================================================
// Talk configuration / control request schemas
// ============================================================================

// ---------- TalkModeParamsSchema ----------

/// Toggles Talk mode for the gateway, with an optional rollout phase marker.
/// 对齐 TS:
///   `Type.Object({
///      enabled: Type.Boolean(),
///      phase:   Type.Optional(Type.String()),
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TalkModeParamsSchema {
    pub enabled: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub phase: Option<String>,
}

impl TalkModeParamsSchema {
    pub fn validate(&self) -> Result<(), String> {
        Ok(())
    }
}

// ---------- TalkConfigParamsSchema ----------

/// Reads Talk configuration; secrets are included only for trusted callers.
/// 对齐 TS:
///   `Type.Object({
///      includeSecrets: Type.Optional(Type.Boolean()),
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TalkConfigParamsSchema {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub include_secrets: Option<bool>,
}

impl TalkConfigParamsSchema {
    pub fn validate(&self) -> Result<(), String> {
        Ok(())
    }
}

// ---------- TalkSpeakParamsSchema ----------

/// One-shot text-to-speech request with provider-specific voice tuning knobs.
/// 对齐 TS:
///   `Type.Object({
///      text:          NonEmptyString,
///      voiceId:       Type.Optional(Type.String()),
///      modelId:       Type.Optional(Type.String()),
///      outputFormat:  Type.Optional(Type.String()),
///      speed:         Type.Optional(Type.Number()),
///      rateWpm:       Type.Optional(Type.Integer({ minimum: 1 })),
///      stability:     Type.Optional(Type.Number()),
///      similarity:    Type.Optional(Type.Number()),
///      style:         Type.Optional(Type.Number()),
///      speakerBoost:  Type.Optional(Type.Boolean()),
///      seed:          Type.Optional(Type.Integer({ minimum: 0 })),
///      normalize:     Type.Optional(Type.String()),
///      language:      Type.Optional(Type.String()),
///      latencyTier:   Type.Optional(Type.Integer({ minimum: 0 })),
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TalkSpeakParamsSchema {
    pub text: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub voice_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output_format: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub speed: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rate_wpm: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stability: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub similarity: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub style: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub speaker_boost: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub seed: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub normalize: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub latency_tier: Option<i64>,
}

impl TalkSpeakParamsSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_non_empty_string("text", &self.text)?;
        if let Some(n) = self.rate_wpm {
            validate_positive_integer("rateWpm", n)?;
        }
        validate_optional_non_negative_integer("seed", self.seed)?;
        validate_optional_non_negative_integer("latencyTier", self.latency_tier)?;
        Ok(())
    }
}

// ---------- TtsSpeakParamsSchema ----------

/// One-shot text-to-speech request rendered with the configured TTS provider
/// chain (unlike `talk.speak`, which pins the Talk-mode provider).
/// 对齐 TS: `Type.Object({ text: NonEmptyString }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TtsSpeakParamsSchema {
    pub text: String,
}

impl TtsSpeakParamsSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_non_empty_string("text", &self.text)?;
        Ok(())
    }
}

// ============================================================================
// Talk event schema and conditional-requirement helpers
// ============================================================================

/// Canonical Talk event envelope emitted to browser, relay, and channel consumers.
/// 对齐 TS:
///   `Type.Object({
///      id:        NonEmptyString,
///      type:      TalkEventTypeSchema,
///      sessionId: NonEmptyString,
///      turnId:    Type.Optional(Type.String()),
///      captureId: Type.Optional(Type.String()),
///      seq:       Type.Integer({ minimum: 1 }),
///      timestamp: NonEmptyString,
///      mode:      TalkModeSchema,
///      transport: TalkTransportSchema,
///      brain:     TalkBrainSchema,
///      provider:  Type.Optional(Type.String()),
///      final:     Type.Optional(Type.Boolean()),
///      callId:    Type.Optional(Type.String()),
///      itemId:    Type.Optional(Type.String()),
///      parentId:  Type.Optional(Type.String()),
///      payload:   Type.Unknown(),
///   }, {
///      additionalProperties: false,
///      allOf: [
///        { if: { properties: { type: { enum: TURN_SCOPED_TALK_EVENT_TYPES } },
///            required: ["type"] },
///          ...requireJsonSchemaProperties(["turnId"]) },
///        { if: { properties: { type: { enum: CAPTURE_SCOPED_TALK_EVENT_TYPES } },
///            required: ["type"] },
///          ...requireJsonSchemaProperties(["captureId"]) },
///      ],
///   })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TalkEventSchema {
    pub id: String,
    #[serde(rename = "type")]
    pub event_type: TalkEventTypeSchema,
    pub session_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub turn_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub capture_id: Option<String>,
    pub seq: i64,
    pub timestamp: String,
    pub mode: TalkModeSchema,
    pub transport: TalkTransportSchema,
    pub brain: TalkBrainSchema,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provider: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "final")]
    pub is_final: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub call_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub item_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent_id: Option<String>,
    pub payload: serde_json::Value,
}

impl TalkEventSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_non_empty_string("id", &self.id)?;
        validate_non_empty_string("sessionId", &self.session_id)?;
        validate_optional_non_empty_string("turnId", self.turn_id.as_deref())?;
        validate_optional_non_empty_string("captureId", self.capture_id.as_deref())?;
        if self.seq < 1 {
            return Err(format!(
                "seq: expected integer >= 1, got {}",
                self.seq
            ));
        }
        validate_non_empty_string("timestamp", &self.timestamp)?;
        validate_optional_non_empty_string("provider", self.provider.as_deref())?;
        validate_optional_non_empty_string("callId", self.call_id.as_deref())?;
        validate_optional_non_empty_string("itemId", self.item_id.as_deref())?;
        validate_optional_non_empty_string("parentId", self.parent_id.as_deref())?;
        // TS `allOf` conditional requirements: turn-id is required for
        // TURN_SCOPED_TALK_EVENT_TYPES, capture-id is required for
        // CAPTURE_SCOPED_TALK_EVENT_TYPES.
        if is_turn_scoped_talk_event_type(self.event_type) && self.turn_id.is_none() {
            return Err(format!(
                "turnId: required for event type {:?}",
                self.event_type.as_str()
            ));
        }
        if is_capture_scoped_talk_event_type(self.event_type) && self.capture_id.is_none() {
            return Err(format!(
                "captureId: required for event type {:?}",
                self.event_type.as_str()
            ));
        }
        Ok(())
    }
}

// ---------- TalkClientCreateParamsSchema ----------

/// Creates a browser-facing Talk client session.
/// 对齐 TS:
///   `Type.Object({
///      sessionKey:        Type.Optional(Type.String()),
///      provider:          Type.Optional(Type.String()),
///      model:             Type.Optional(Type.String()),
///      voice:             Type.Optional(Type.String()),
///      vadThreshold:      Type.Optional(Type.Number()),
///      silenceDurationMs: Type.Optional(Type.Integer({ minimum: 1 })),
///      prefixPaddingMs:   Type.Optional(Type.Integer({ minimum: 0 })),
///      reasoningEffort:   Type.Optional(Type.String()),
///      mode:              Type.Optional(TalkModeSchema),
///      transport:         Type.Optional(TalkTransportSchema),
///      brain:             Type.Optional(TalkBrainSchema),
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TalkClientCreateParamsSchema {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub session_key: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provider: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub voice: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub vad_threshold: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub silence_duration_ms: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prefix_padding_ms: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reasoning_effort: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mode: Option<TalkModeSchema>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub transport: Option<TalkTransportSchema>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub brain: Option<TalkBrainSchema>,
}

impl TalkClientCreateParamsSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_optional_positive_integer("silenceDurationMs", self.silence_duration_ms)?;
        validate_optional_non_negative_integer("prefixPaddingMs", self.prefix_padding_ms)?;
        Ok(())
    }
}

fn validate_optional_positive_integer(field: &str, n: Option<i64>) -> Result<(), String> {
    if let Some(v) = n {
        validate_positive_integer(field, v)?;
    }
    Ok(())
}

// ---------- TalkClientToolCallParamsSchema / ResultSchema ----------

/// Tool-call request from a browser/client session back into the agent runtime.
/// 对齐 TS:
///   `Type.Object({
///      sessionKey:      NonEmptyString,
///      callId:          NonEmptyString,
///      name:            NonEmptyString,
///      args:            Type.Optional(Type.Unknown()),
///      relaySessionId:  Type.Optional(NonEmptyString),
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TalkClientToolCallParamsSchema {
    pub session_key: String,
    pub call_id: String,
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub args: Option<serde_json::Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub relay_session_id: Option<String>,
}

impl TalkClientToolCallParamsSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_non_empty_string("sessionKey", &self.session_key)?;
        validate_non_empty_string("callId", &self.call_id)?;
        validate_non_empty_string("name", &self.name)?;
        validate_optional_non_empty_string("relaySessionId", self.relay_session_id.as_deref())?;
        Ok(())
    }
}

/// Agent run identity returned after accepting a Talk client tool call.
/// 对齐 TS:
///   `Type.Object({
///      runId:           NonEmptyString,
///      idempotencyKey:  NonEmptyString,
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TalkClientToolCallResultSchema {
    pub run_id: String,
    pub idempotency_key: String,
}

impl TalkClientToolCallResultSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_non_empty_string("runId", &self.run_id)?;
        validate_non_empty_string("idempotencyKey", &self.idempotency_key)?;
        Ok(())
    }
}

// ---------- TalkClientSteerParamsSchema ----------

/// Text steering request for a Talk session bound to an agent turn.
/// 对齐 TS:
///   `Type.Object({
///      sessionKey: NonEmptyString,
///      text:       NonEmptyString,
///      mode:       Type.Optional(TalkAgentControlModeSchema),
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TalkClientSteerParamsSchema {
    pub session_key: String,
    pub text: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mode: Option<TalkAgentControlModeSchema>,
}

impl TalkClientSteerParamsSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_non_empty_string("sessionKey", &self.session_key)?;
        validate_non_empty_string("text", &self.text)?;
        Ok(())
    }
}

// ---------- TalkProviderCancelledResultSchema (nested under providerResult) ----------

/// Closed `providerResult` payload for cancelled agent-control replies.
/// 对齐 TS:
///   `Type.Object({
///      status:  Type.Literal("cancelled"),
///      message: Type.String(),
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TalkProviderCancelledResultSchema {
    pub status: TalkProviderCancelledStatusSchema,
    pub message: String,
}

impl TalkProviderCancelledResultSchema {
    pub fn validate(&self) -> Result<(), String> {
        Ok(())
    }
}

// ---------- TalkAgentControlResultSchema ----------

/// Result of applying agent control to an embedded or reply-backed Talk run.
/// 对齐 TS:
///   `Type.Object({
///      ok:             Type.Boolean(),
///      mode:           TalkAgentControlModeSchema,
///      sessionKey:     NonEmptyString,
///      sessionId:      Type.Optional(Type.String()),
///      active:         Type.Boolean(),
///      queued:         Type.Optional(Type.Boolean()),
///      aborted:        Type.Optional(Type.Boolean()),
///      target:         Type.Optional(Type.Union([...embedded_run/reply_run...])),
///      reason:         Type.Optional(Type.String()),
///      message:        Type.String(),
///      speak:          Type.Boolean(),
///      show:           Type.Boolean(),
///      suppress:       Type.Boolean(),
///      providerResult: Type.Optional(TalkProviderCancelledResultSchema),
///      enqueuedAtMs:   Type.Optional(Type.Number()),
///      deliveredAtMs:  Type.Optional(Type.Number()),
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TalkAgentControlResultSchema {
    pub ok: bool,
    pub mode: TalkAgentControlModeSchema,
    pub session_key: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
    pub active: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub queued: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub aborted: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target: Option<TalkAgentControlTargetSchema>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    pub message: String,
    pub speak: bool,
    pub show: bool,
    pub suppress: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provider_result: Option<TalkProviderCancelledResultSchema>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub enqueued_at_ms: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub delivered_at_ms: Option<f64>,
}

impl TalkAgentControlResultSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_non_empty_string("sessionKey", &self.session_key)?;
        validate_optional_non_empty_string("sessionId", self.session_id.as_deref())?;
        validate_optional_non_empty_string("reason", self.reason.as_deref())?;
        if let Some(pr) = &self.provider_result {
            pr.validate().map_err(|e| format!("providerResult: {}", e))?;
        }
        Ok(())
    }
}

// ---------- TalkSessionJoinParamsSchema ----------

/// Joins an existing managed-room Talk session.
/// 对齐 TS:
///   `Type.Object({
///      sessionId: NonEmptyString,
///      token:     NonEmptyString,
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TalkSessionJoinParamsSchema {
    pub session_id: String,
    pub token: String,
}

impl TalkSessionJoinParamsSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_non_empty_string("sessionId", &self.session_id)?;
        validate_non_empty_string("token", &self.token)?;
        Ok(())
    }
}

// ---------- TalkSessionCreateParamsSchema ----------

/// Creates a gateway-managed Talk session for realtime, transcription, or relay use.
/// 对齐 TS:
///   `Type.Object({
///      sessionKey:        Type.Optional(Type.String()),
///      spawnedBy:         Type.Optional(NonEmptyString),
///      provider:          Type.Optional(Type.String()),
///      model:             Type.Optional(Type.String()),
///      voice:             Type.Optional(Type.String()),
///      vadThreshold:      Type.Optional(Type.Number()),
///      silenceDurationMs: Type.Optional(Type.Integer({ minimum: 1 })),
///      prefixPaddingMs:   Type.Optional(Type.Integer({ minimum: 0 })),
///      reasoningEffort:   Type.Optional(Type.String()),
///      mode:              Type.Optional(TalkModeSchema),
///      transport:         Type.Optional(TalkTransportSchema),
///      brain:             Type.Optional(TalkBrainSchema),
///      ttlMs:             Type.Optional(Type.Integer({ minimum: 1000, maximum: 3600000 })),
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TalkSessionCreateParamsSchema {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub session_key: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub spawned_by: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provider: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub voice: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub vad_threshold: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub silence_duration_ms: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prefix_padding_ms: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reasoning_effort: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mode: Option<TalkModeSchema>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub transport: Option<TalkTransportSchema>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub brain: Option<TalkBrainSchema>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ttl_ms: Option<i64>,
}

impl TalkSessionCreateParamsSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_optional_non_empty_string("spawnedBy", self.spawned_by.as_deref())?;
        validate_optional_positive_integer("silenceDurationMs", self.silence_duration_ms)?;
        validate_optional_non_negative_integer("prefixPaddingMs", self.prefix_padding_ms)?;
        if let Some(t) = self.ttl_ms {
            validate_integer_in_range(
                "ttlMs",
                t,
                TALK_SESSION_TTL_MS_MIN,
                TALK_SESSION_TTL_MS_MAX,
            )?;
        }
        Ok(())
    }
}

// ---------- TalkSessionAppendAudioParamsSchema ----------

/// Appends base64 audio to an active Talk session.
/// 对齐 TS:
///   `Type.Object({
///      sessionId:    NonEmptyString,
///      audioBase64:  NonEmptyString,
///      timestamp:    Type.Optional(Type.Number()),
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TalkSessionAppendAudioParamsSchema {
    pub session_id: String,
    pub audio_base64: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<f64>,
}

impl TalkSessionAppendAudioParamsSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_non_empty_string("sessionId", &self.session_id)?;
        validate_non_empty_string("audioBase64", &self.audio_base64)?;
        Ok(())
    }
}

// ---------- TalkSessionTurnParamsSchema ----------

/// Starts or advances a Talk turn within a session.
/// 对齐 TS:
///   `Type.Object({
///      sessionId: NonEmptyString,
///      turnId:    Type.Optional(Type.String()),
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TalkSessionTurnParamsSchema {
    pub session_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub turn_id: Option<String>,
}

impl TalkSessionTurnParamsSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_non_empty_string("sessionId", &self.session_id)?;
        validate_optional_non_empty_string("turnId", self.turn_id.as_deref())?;
        Ok(())
    }
}

// ---------- TalkSessionCancelTurnParamsSchema ----------

/// Cancels the active or named Talk turn.
/// 对齐 TS:
///   `Type.Object({
///      sessionId: NonEmptyString,
///      turnId:    Type.Optional(Type.String()),
///      reason:    Type.Optional(Type.String()),
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TalkSessionCancelTurnParamsSchema {
    pub session_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub turn_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

impl TalkSessionCancelTurnParamsSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_non_empty_string("sessionId", &self.session_id)?;
        validate_optional_non_empty_string("turnId", self.turn_id.as_deref())?;
        Ok(())
    }
}

// ---------- TalkSessionCancelOutputParamsSchema ----------

/// Cancels currently streaming Talk output without necessarily ending the turn.
/// 对齐 TS:
///   `Type.Object({
///      sessionId: NonEmptyString,
///      turnId:    Type.Optional(Type.String()),
///      reason:    Type.Optional(Type.String()),
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TalkSessionCancelOutputParamsSchema {
    pub session_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub turn_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

impl TalkSessionCancelOutputParamsSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_non_empty_string("sessionId", &self.session_id)?;
        validate_optional_non_empty_string("turnId", self.turn_id.as_deref())?;
        Ok(())
    }
}

// ---------- TalkSessionSubmitToolResultOptionsSchema ----------

/// `TalkSessionSubmitToolResultParamsSchema.options` payload.
/// 对齐 TS:
///   `Type.Object({
///      suppressResponse: Type.Optional(Type.Boolean()),
///      willContinue:     Type.Optional(Type.Boolean()),
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TalkSessionSubmitToolResultOptionsSchema {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub suppress_response: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub will_continue: Option<bool>,
}

impl TalkSessionSubmitToolResultOptionsSchema {
    pub fn validate(&self) -> Result<(), String> {
        Ok(())
    }
}

// ---------- TalkSessionSubmitToolResultParamsSchema ----------

/// Submits a tool result back to a Talk provider session.
/// 对齐 TS:
///   `Type.Object({
///      sessionId: NonEmptyString,
///      callId:    NonEmptyString,
///      result:    Type.Unknown(),
///      options:   Type.Optional(TalkSessionSubmitToolResultOptionsSchema),
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TalkSessionSubmitToolResultParamsSchema {
    pub session_id: String,
    pub call_id: String,
    pub result: serde_json::Value,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub options: Option<TalkSessionSubmitToolResultOptionsSchema>,
}

impl TalkSessionSubmitToolResultParamsSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_non_empty_string("sessionId", &self.session_id)?;
        validate_non_empty_string("callId", &self.call_id)?;
        if let Some(opts) = &self.options {
            opts.validate().map_err(|e| format!("options: {}", e))?;
        }
        Ok(())
    }
}

// ---------- TalkSessionSteerParamsSchema ----------

/// Steers a managed Talk session by session id rather than transcript key.
/// 对齐 TS:
///   `Type.Object({
///      sessionId:  NonEmptyString,
///      sessionKey: Type.Optional(NonEmptyString),
///      text:       NonEmptyString,
///      mode:       Type.Optional(TalkAgentControlModeSchema),
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TalkSessionSteerParamsSchema {
    pub session_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub session_key: Option<String>,
    pub text: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mode: Option<TalkAgentControlModeSchema>,
}

impl TalkSessionSteerParamsSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_non_empty_string("sessionId", &self.session_id)?;
        validate_optional_non_empty_string("sessionKey", self.session_key.as_deref())?;
        validate_non_empty_string("text", &self.text)?;
        Ok(())
    }
}

// ---------- TalkSessionCloseParamsSchema ----------

/// Closes a gateway-managed Talk session.
/// 对齐 TS: `Type.Object({ sessionId: NonEmptyString }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TalkSessionCloseParamsSchema {
    pub session_id: String,
}

impl TalkSessionCloseParamsSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_non_empty_string("sessionId", &self.session_id)?;
        Ok(())
    }
}

// ============================================================================
// Talk catalog schemas
// ============================================================================

// ---------- BrowserRealtimeAudioFormatSpecSchema ----------

/// One audio format description for a Talk catalog provider.
/// 对齐 TS:
///   `Type.Object({
///      encoding:      Type.Union([Type.Literal("pcm16"), Type.Literal("g711_ulaw")]),
///      sampleRateHz:  Type.Integer({ minimum: 1 }),
///      channels:      Type.Integer({ minimum: 1 }),
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BrowserRealtimeAudioFormatSpecSchema {
    pub encoding: BrowserRealtimeEncodingSchema,
    pub sample_rate_hz: i64,
    pub channels: i64,
}

impl BrowserRealtimeAudioFormatSpecSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_positive_integer("sampleRateHz", self.sample_rate_hz)?;
        validate_positive_integer("channels", self.channels)?;
        Ok(())
    }
}

// ---------- TalkCatalogProviderSchema ----------

/// One provider entry in the Talk capability catalog.
/// 对齐 TS:
///   `Type.Object({
///      id:          NonEmptyString,
///      label:       NonEmptyString,
///      configured:  Type.Boolean(),
///      aliases:     Type.Optional(Type.Array(NonEmptyString)),
///      models:      Type.Optional(Type.Array(Type.String())),
///      voices:      Type.Optional(Type.Array(Type.String())),
///      defaultModel:Type.Optional(Type.String()),
///      modes:       Type.Optional(Type.Array(TalkModeSchema)),
///      transports:  Type.Optional(Type.Array(TalkTransportSchema)),
///      brains:      Type.Optional(Type.Array(TalkBrainSchema)),
///      inputAudioFormats:  Type.Optional(Type.Array(BrowserRealtimeAudioFormatSpecSchema)),
///      outputAudioFormats: Type.Optional(Type.Array(BrowserRealtimeAudioFormatSpecSchema)),
///      supportsBrowserSession:    Type.Optional(Type.Boolean()),
///      supportsBargeIn:           Type.Optional(Type.Boolean()),
///      supportsToolCalls:         Type.Optional(Type.Boolean()),
///      supportsVideoFrames:       Type.Optional(Type.Boolean()),
///      supportsSessionResumption: Type.Optional(Type.Boolean()),
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TalkCatalogProviderSchema {
    pub id: String,
    pub label: String,
    pub configured: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub aliases: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub models: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub voices: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_model: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub modes: Option<Vec<TalkModeSchema>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub transports: Option<Vec<TalkTransportSchema>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub brains: Option<Vec<TalkBrainSchema>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub input_audio_formats: Option<Vec<BrowserRealtimeAudioFormatSpecSchema>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output_audio_formats: Option<Vec<BrowserRealtimeAudioFormatSpecSchema>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub supports_browser_session: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub supports_barge_in: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub supports_tool_calls: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub supports_video_frames: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub supports_session_resumption: Option<bool>,
}

impl TalkCatalogProviderSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_non_empty_string("id", &self.id)?;
        validate_non_empty_string("label", &self.label)?;
        validate_optional_non_empty_string_list("aliases", self.aliases.as_ref())?;
        Ok(())
    }
}

// ---------- TalkCatalogProviderGroupSchema ----------

/// Active provider plus all candidates for a Talk capability family.
/// 对齐 TS:
///   `Type.Object({
///      ready:          Type.Optional(Type.Boolean()),
///      activeProvider: Type.Optional(Type.String()),
///      providers:      Type.Array(TalkCatalogProviderSchema),
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TalkCatalogProviderGroupSchema {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ready: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub active_provider: Option<String>,
    pub providers: Vec<TalkCatalogProviderSchema>,
}

impl TalkCatalogProviderGroupSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_optional_non_empty_string("activeProvider", self.active_provider.as_deref())?;
        for (i, p) in self.providers.iter().enumerate() {
            p.validate()
                .map_err(|e| format!("providers[{}]: {}", i, e))?;
        }
        Ok(())
    }
}

// ---------- TalkCatalogParamsSchema ----------

/// Empty request payload for reading configured Talk provider capabilities.
/// 对齐 TS: `Type.Object({}, { additionalProperties: false })`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TalkCatalogParamsSchema {}

impl TalkCatalogParamsSchema {
    pub fn validate(&self) -> Result<(), String> {
        Ok(())
    }
}

// ---------- TalkCatalogResultSchema ----------

/// Provider, mode, transport, and audio-format catalog returned to clients.
/// 对齐 TS:
///   `Type.Object({
///      modes:         Type.Array(TalkModeSchema),
///      transports:    Type.Array(TalkTransportSchema),
///      brains:        Type.Array(TalkBrainSchema),
///      speech:        TalkCatalogProviderGroupSchema,
///      transcription: TalkCatalogProviderGroupSchema,
///      realtime:      TalkCatalogProviderGroupSchema,
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TalkCatalogResultSchema {
    pub modes: Vec<TalkModeSchema>,
    pub transports: Vec<TalkTransportSchema>,
    pub brains: Vec<TalkBrainSchema>,
    pub speech: TalkCatalogProviderGroupSchema,
    pub transcription: TalkCatalogProviderGroupSchema,
    pub realtime: TalkCatalogProviderGroupSchema,
}

impl TalkCatalogResultSchema {
    pub fn validate(&self) -> Result<(), String> {
        self.speech.validate().map_err(|e| format!("speech: {}", e))?;
        self.transcription
            .validate()
            .map_err(|e| format!("transcription: {}", e))?;
        self.realtime
            .validate()
            .map_err(|e| format!("realtime: {}", e))?;
        Ok(())
    }
}

// ============================================================================
// Talk session result schemas (create/turn/join/close)
// ============================================================================

// ---------- BrowserRealtimeAudioContractSchema ----------

/// Audio format contract for realtime browser sessions.
/// 对齐 TS:
///   `Type.Object({
///      inputEncoding:      Type.Union([Type.Literal("pcm16"), Type.Literal("g711_ulaw")]),
///      inputSampleRateHz:  Type.Integer({ minimum: 1 }),
///      outputEncoding:     Type.Union([Type.Literal("pcm16"), Type.Literal("g711_ulaw")]),
///      outputSampleRateHz: Type.Integer({ minimum: 1 }),
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BrowserRealtimeAudioContractSchema {
    pub input_encoding: BrowserRealtimeEncodingSchema,
    pub input_sample_rate_hz: i64,
    pub output_encoding: BrowserRealtimeEncodingSchema,
    pub output_sample_rate_hz: i64,
}

impl BrowserRealtimeAudioContractSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_positive_integer("inputSampleRateHz", self.input_sample_rate_hz)?;
        validate_positive_integer("outputSampleRateHz", self.output_sample_rate_hz)?;
        Ok(())
    }
}

// ---------- TalkSessionCreateResultSchema ----------

/// Session creation result with transport-specific ids and credentials.
/// 对齐 TS:
///   `Type.Object({
///      sessionId:             NonEmptyString,
///      provider:              Type.Optional(Type.String()),
///      mode:                  TalkModeSchema,
///      transport:             TalkTransportSchema,
///      brain:                 TalkBrainSchema,
///      relaySessionId:        Type.Optional(NonEmptyString),
///      transcriptionSessionId:Type.Optional(NonEmptyString),
///      handoffId:             Type.Optional(NonEmptyString),
///      roomId:                Type.Optional(NonEmptyString),
///      roomUrl:               Type.Optional(NonEmptyString),
///      token:                 Type.Optional(NonEmptyString),
///      audio:                 Type.Optional(Type.Unknown()),
///      model:                 Type.Optional(Type.String()),
///      voice:                 Type.Optional(Type.String()),
///      expiresAt:             Type.Optional(Type.Number()),
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TalkSessionCreateResultSchema {
    pub session_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provider: Option<String>,
    pub mode: TalkModeSchema,
    pub transport: TalkTransportSchema,
    pub brain: TalkBrainSchema,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub relay_session_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub transcription_session_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub handoff_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub room_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub room_url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub token: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub audio: Option<serde_json::Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub voice: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<f64>,
}

impl TalkSessionCreateResultSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_non_empty_string("sessionId", &self.session_id)?;
        validate_optional_non_empty_string("provider", self.provider.as_deref())?;
        validate_optional_non_empty_string("relaySessionId", self.relay_session_id.as_deref())?;
        validate_optional_non_empty_string(
            "transcriptionSessionId",
            self.transcription_session_id.as_deref(),
        )?;
        validate_optional_non_empty_string("handoffId", self.handoff_id.as_deref())?;
        validate_optional_non_empty_string("roomId", self.room_id.as_deref())?;
        validate_optional_non_empty_string("roomUrl", self.room_url.as_deref())?;
        validate_optional_non_empty_string("token", self.token.as_deref())?;
        Ok(())
    }
}

// ---------- TalkSessionTurnResultSchema ----------

/// Result for a Talk turn request, optionally including emitted events.
/// 对齐 TS:
///   `Type.Object({
///      ok:     Type.Boolean(),
///      turnId: Type.Optional(Type.String()),
///      events: Type.Optional(Type.Array(TalkEventSchema)),
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TalkSessionTurnResultSchema {
    pub ok: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub turn_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub events: Option<Vec<TalkEventSchema>>,
}

impl TalkSessionTurnResultSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_optional_non_empty_string("turnId", self.turn_id.as_deref())?;
        if let Some(events) = &self.events {
            for (i, ev) in events.iter().enumerate() {
                ev.validate()
                    .map_err(|e| format!("events[{}]: {}", i, e))?;
            }
        }
        Ok(())
    }
}

// ---------- TalkSessionManagedRoomStateSchema ----------

/// Mutable room state returned when a client joins a managed Talk room.
/// 对齐 TS:
///   `Type.Object({
///      activeClientId:    Type.Optional(Type.String()),
///      activeTurnId:      Type.Optional(Type.String()),
///      recentTalkEvents:  Type.Array(TalkEventSchema),
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TalkSessionManagedRoomStateSchema {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub active_client_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub active_turn_id: Option<String>,
    pub recent_talk_events: Vec<TalkEventSchema>,
}

impl TalkSessionManagedRoomStateSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_optional_non_empty_string("activeClientId", self.active_client_id.as_deref())?;
        validate_optional_non_empty_string("activeTurnId", self.active_turn_id.as_deref())?;
        for (i, ev) in self.recent_talk_events.iter().enumerate() {
            ev.validate()
                .map_err(|e| format!("recentTalkEvents[{}]: {}", i, e))?;
        }
        Ok(())
    }
}

// ---------- TalkSessionManagedRoomRecordSchema ----------

/// Managed-room session record shared with browser clients.
/// 对齐 TS:
///   `Type.Object({
///      id:         NonEmptyString,
///      roomId:     NonEmptyString,
///      roomUrl:    NonEmptyString,
///      sessionKey: NonEmptyString,
///      sessionId:  Type.Optional(Type.String()),
///      channel:    Type.Optional(Type.String()),
///      target:     Type.Optional(Type.String()),
///      provider:   Type.Optional(Type.String()),
///      model:      Type.Optional(Type.String()),
///      voice:      Type.Optional(Type.String()),
///      mode:       TalkModeSchema,
///      transport:  TalkTransportSchema,
///      brain:      TalkBrainSchema,
///      createdAt:  Type.Number(),
///      expiresAt:  Type.Number(),
///      room:       TalkSessionManagedRoomStateSchema,
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TalkSessionManagedRoomRecordSchema {
    pub id: String,
    pub room_id: String,
    pub room_url: String,
    pub session_key: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub channel: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provider: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub voice: Option<String>,
    pub mode: TalkModeSchema,
    pub transport: TalkTransportSchema,
    pub brain: TalkBrainSchema,
    pub created_at: f64,
    pub expires_at: f64,
    pub room: TalkSessionManagedRoomStateSchema,
}

impl TalkSessionManagedRoomRecordSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_non_empty_string("id", &self.id)?;
        validate_non_empty_string("roomId", &self.room_id)?;
        validate_non_empty_string("roomUrl", &self.room_url)?;
        validate_non_empty_string("sessionKey", &self.session_key)?;
        validate_optional_non_empty_string("sessionId", self.session_id.as_deref())?;
        validate_optional_non_empty_string("channel", self.channel.as_deref())?;
        validate_optional_non_empty_string("target", self.target.as_deref())?;
        validate_optional_non_empty_string("provider", self.provider.as_deref())?;
        self.room.validate().map_err(|e| format!("room: {}", e))?;
        Ok(())
    }
}

/// Managed-room record returned to clients after joining an existing Talk session.
/// 对齐 TS: `TalkSessionJoinResultSchema = TalkSessionManagedRoomRecordSchema`.
pub type TalkSessionJoinResultSchema = TalkSessionManagedRoomRecordSchema;

// ---------- TalkSessionOkResultSchema ----------

/// Generic success result for Talk session lifecycle calls.
/// 对齐 TS: `Type.Object({ ok: Type.Boolean() }, { additionalProperties: false })`.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct TalkSessionOkResultSchema {
    pub ok: bool,
}

impl TalkSessionOkResultSchema {
    pub fn validate(&self) -> Result<(), String> {
        Ok(())
    }
}

// ============================================================================
// Browser realtime session setup union
// ============================================================================

// ---------- BrowserRealtimeWebRtcSdpSessionSchema ----------

/// Browser WebRTC setup payload using provider SDP exchange.
/// 对齐 TS:
///   `Type.Object({
///      provider:      NonEmptyString,
///      transport:     Type.Literal("webrtc"),
///      clientSecret:  NonEmptyString,
///      offerUrl:      Type.Optional(Type.String()),
///      offerHeaders:  Type.Optional(Type.Record(Type.String(), Type.String())),
///      model:         Type.Optional(Type.String()),
///      voice:         Type.Optional(Type.String()),
///      expiresAt:     Type.Optional(Type.Number()),
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BrowserRealtimeWebRtcSdpSessionSchema {
    pub provider: String,
    pub transport: TalkTransportWebrtcLiteral,
    pub client_secret: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub offer_url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub offer_headers: Option<HashMap<String, String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub voice: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<f64>,
}

impl BrowserRealtimeWebRtcSdpSessionSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_non_empty_string("provider", &self.provider)?;
        validate_non_empty_string("clientSecret", &self.client_secret)?;
        Ok(())
    }
}

/// 对齐 TS: `Type.Literal("webrtc")`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TalkTransportWebrtcLiteral {
    #[serde(rename = "webrtc")]
    Webrtc,
}

// ---------- BrowserRealtimeJsonPcmWebSocketSessionSchema ----------

/// Browser websocket setup payload with JSON/PCM audio contract.
/// 对齐 TS:
///   `Type.Object({
///      provider:       NonEmptyString,
///      transport:      Type.Literal("provider-websocket"),
///      protocol:       NonEmptyString,
///      clientSecret:   NonEmptyString,
///      websocketUrl:   NonEmptyString,
///      audio:          BrowserRealtimeAudioContractSchema,
///      initialMessage: Type.Optional(Type.Unknown()),
///      model:          Type.Optional(Type.String()),
///      voice:          Type.Optional(Type.String()),
///      expiresAt:      Type.Optional(Type.Number()),
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BrowserRealtimeJsonPcmWebSocketSessionSchema {
    pub provider: String,
    pub transport: TalkTransportProviderWebsocketLiteral,
    pub protocol: String,
    pub client_secret: String,
    pub websocket_url: String,
    pub audio: BrowserRealtimeAudioContractSchema,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub initial_message: Option<serde_json::Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub voice: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<f64>,
}

impl BrowserRealtimeJsonPcmWebSocketSessionSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_non_empty_string("provider", &self.provider)?;
        validate_non_empty_string("protocol", &self.protocol)?;
        validate_non_empty_string("clientSecret", &self.client_secret)?;
        validate_non_empty_string("websocketUrl", &self.websocket_url)?;
        self.audio.validate().map_err(|e| format!("audio: {}", e))?;
        Ok(())
    }
}

/// 对齐 TS: `Type.Literal("provider-websocket")`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TalkTransportProviderWebsocketLiteral {
    #[serde(rename = "provider-websocket")]
    ProviderWebsocket,
}

// ---------- BrowserRealtimeGatewayRelaySessionSchema ----------

/// Browser setup payload for gateway-relayed realtime audio.
/// 对齐 TS:
///   `Type.Object({
///      provider:        NonEmptyString,
///      transport:       Type.Literal("gateway-relay"),
///      relaySessionId:  NonEmptyString,
///      audio:           BrowserRealtimeAudioContractSchema,
///      model:           Type.Optional(Type.String()),
///      voice:           Type.Optional(Type.String()),
///      expiresAt:       Type.Optional(Type.Number()),
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BrowserRealtimeGatewayRelaySessionSchema {
    pub provider: String,
    pub transport: TalkTransportGatewayRelayLiteral,
    pub relay_session_id: String,
    pub audio: BrowserRealtimeAudioContractSchema,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub voice: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<f64>,
}

impl BrowserRealtimeGatewayRelaySessionSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_non_empty_string("provider", &self.provider)?;
        validate_non_empty_string("relaySessionId", &self.relay_session_id)?;
        self.audio.validate().map_err(|e| format!("audio: {}", e))?;
        Ok(())
    }
}

/// 对齐 TS: `Type.Literal("gateway-relay")`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TalkTransportGatewayRelayLiteral {
    #[serde(rename = "gateway-relay")]
    GatewayRelay,
}

// ---------- BrowserRealtimeManagedRoomSessionSchema ----------

/// Browser setup payload for managed-room Talk sessions.
/// 对齐 TS:
///   `Type.Object({
///      provider:   NonEmptyString,
///      transport:  Type.Literal("managed-room"),
///      roomUrl:    NonEmptyString,
///      token:      Type.Optional(Type.String()),
///      model:      Type.Optional(Type.String()),
///      voice:      Type.Optional(Type.String()),
///      expiresAt:  Type.Optional(Type.Number()),
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BrowserRealtimeManagedRoomSessionSchema {
    pub provider: String,
    pub transport: TalkTransportManagedRoomLiteral,
    pub room_url: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub token: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub voice: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<f64>,
}

impl BrowserRealtimeManagedRoomSessionSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_non_empty_string("provider", &self.provider)?;
        validate_non_empty_string("roomUrl", &self.room_url)?;
        Ok(())
    }
}

/// 对齐 TS: `Type.Literal("managed-room")`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TalkTransportManagedRoomLiteral {
    #[serde(rename = "managed-room")]
    ManagedRoom,
}

// ---------- TalkClientCreateResultSchema (union) ----------

/// Union of all browser Talk session setup payloads.
/// 对齐 TS: `Type.Union([
///   BrowserRealtimeWebRtcSdpSessionSchema,
///   BrowserRealtimeJsonPcmWebSocketSessionSchema,
///   BrowserRealtimeGatewayRelaySessionSchema,
///   BrowserRealtimeManagedRoomSessionSchema,
/// ])`.
///
/// 在 Rust 中我们通过外层 wrapper 把四种 variant 重新暴露（每个内层已固定
/// `transport` 字段），由调用方根据当前结构字段选择具体取值。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum TalkClientCreateResultSchema {
    Webrtc(Box<BrowserRealtimeWebRtcSdpSessionSchema>),
    ProviderWebsocket(Box<BrowserRealtimeJsonPcmWebSocketSessionSchema>),
    GatewayRelay(Box<BrowserRealtimeGatewayRelaySessionSchema>),
    ManagedRoom(Box<BrowserRealtimeManagedRoomSessionSchema>),
}

impl TalkClientCreateResultSchema {
    pub fn validate(&self) -> Result<(), String> {
        match self {
            Self::Webrtc(v) => v.validate(),
            Self::ProviderWebsocket(v) => v.validate(),
            Self::GatewayRelay(v) => v.validate(),
            Self::ManagedRoom(v) => v.validate(),
        }
    }
}

// ============================================================================
// Talk config storage / read result schemas
// ============================================================================

// ---------- TalkProviderConfigSchema (additionalProperties: true) ----------

/// Per-provider Talk config bag; unknown fields are passthrough for the
/// provider adapter. 对齐 TS:
///   `Type.Object({ apiKey: Type.Optional(SecretInputSchema) },
///                { additionalProperties: true })`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TalkProviderConfigSchema {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub api_key: Option<SecretInputSchema>,
    #[serde(default, flatten)]
    pub extra: BTreeMap<String, serde_json::Value>,
}

impl TalkProviderConfigSchema {
    pub fn validate(&self) -> Result<(), String> {
        Ok(())
    }
}

// ---------- TalkRealtimeConfigSchema ----------

/// Realtime Talk defaults and provider selection stored in config.
/// 对齐 TS:
///   `Type.Object({
///      provider:          Type.Optional(Type.String()),
///      providers:         Type.Optional(Type.Record(Type.String(), TalkProviderConfigSchema)),
///      model:             Type.Optional(Type.String()),
///      speakerVoice:      Type.Optional(Type.String()),
///      speakerVoiceId:    Type.Optional(Type.String()),
///      voice:             Type.Optional(Type.String()),
///      instructions:      Type.Optional(Type.String()),
///      mode:              Type.Optional(TalkModeSchema),
///      transport:         Type.Optional(TalkTransportSchema),
///      vadThreshold:      Type.Optional(Type.Number({ minimum: 0, maximum: 1 })),
///      silenceDurationMs: Type.Optional(Type.Integer({ minimum: 1 })),
///      prefixPaddingMs:   Type.Optional(Type.Integer({ minimum: 0 })),
///      reasoningEffort:   Type.Optional(Type.String({ minLength: 1 })),
///      brain:             Type.Optional(TalkBrainSchema),
///      consultRouting:    Type.Optional(
///        Type.Union([Type.Literal("provider-direct"), Type.Literal("force-agent-consult")]),
///      ),
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TalkRealtimeConfigSchema {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provider: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub providers: Option<HashMap<String, TalkProviderConfigSchema>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub speaker_voice: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub speaker_voice_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub voice: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub instructions: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mode: Option<TalkModeSchema>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub transport: Option<TalkTransportSchema>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub vad_threshold: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub silence_duration_ms: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prefix_padding_ms: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reasoning_effort: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub brain: Option<TalkBrainSchema>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub consult_routing: Option<TalkConsultRoutingSchema>,
}

impl TalkRealtimeConfigSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_optional_number_in_range(
            "vadThreshold",
            self.vad_threshold,
            TALK_VAD_THRESHOLD_MIN,
            TALK_VAD_THRESHOLD_MAX,
        )?;
        validate_optional_positive_integer("silenceDurationMs", self.silence_duration_ms)?;
        validate_optional_non_negative_integer("prefixPaddingMs", self.prefix_padding_ms)?;
        validate_optional_string_min_length("reasoningEffort", self.reasoning_effort.as_deref(), 1)?;
        Ok(())
    }
}

// ---------- ResolvedTalkConfigSchema ----------

/// Resolved active Talk provider plus its normalized provider config.
/// 对齐 TS:
///   `Type.Object({
///      provider: Type.String(),
///      config:   TalkProviderConfigSchema,
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResolvedTalkConfigSchema {
    pub provider: String,
    pub config: TalkProviderConfigSchema,
}

impl ResolvedTalkConfigSchema {
    pub fn validate(&self) -> Result<(), String> {
        self.config.validate().map_err(|e| format!("config: {}", e))?;
        Ok(())
    }
}

// ---------- TalkConfigSchema ----------

/// Talk config subtree returned through gateway config APIs.
/// 对齐 TS:
///   `Type.Object({
///      provider:             Type.Optional(Type.String()),
///      providers:            Type.Optional(Type.Record(Type.String(), TalkProviderConfigSchema)),
///      realtime:             Type.Optional(TalkRealtimeConfigSchema),
///      resolved:             Type.Optional(ResolvedTalkConfigSchema),
///      consultThinkingLevel: Type.Optional(Type.String()),
///      consultFastMode:      Type.Optional(Type.Boolean()),
///      speechLocale:         Type.Optional(Type.String()),
///      interruptOnSpeech:    Type.Optional(Type.Boolean()),
///      silenceTimeoutMs:     Type.Optional(Type.Integer({ minimum: 1 })),
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TalkConfigSchema {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provider: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub providers: Option<HashMap<String, TalkProviderConfigSchema>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub realtime: Option<TalkRealtimeConfigSchema>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resolved: Option<ResolvedTalkConfigSchema>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub consult_thinking_level: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub consult_fast_mode: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub speech_locale: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub interrupt_on_speech: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub silence_timeout_ms: Option<i64>,
}

impl TalkConfigSchema {
    pub fn validate(&self) -> Result<(), String> {
        if let Some(r) = &self.realtime {
            r.validate().map_err(|e| format!("realtime: {}", e))?;
        }
        if let Some(r) = &self.resolved {
            r.validate().map_err(|e| format!("resolved: {}", e))?;
        }
        validate_optional_positive_integer("silenceTimeoutMs", self.silence_timeout_ms)?;
        Ok(())
    }
}

// ---------- TalkConfigResultSchema ----------

/// Full Talk config read result, including related session/UI context.
/// 对齐 TS:
///   `Type.Object({
///      config: Type.Object({
///        talk:    Type.Optional(TalkConfigSchema),
///        session: Type.Optional(Type.Object({ mainKey: Type.Optional(Type.String()) },
///                                          { additionalProperties: false })),
///        ui:      Type.Optional(Type.Object({ seamColor: Type.Optional(Type.String()) },
///                                          { additionalProperties: false })),
///      }, { additionalProperties: false }),
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TalkConfigSessionSubsetSchema {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub main_key: Option<String>,
}

impl TalkConfigSessionSubsetSchema {
    pub fn validate(&self) -> Result<(), String> {
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TalkConfigUiSubsetSchema {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub seam_color: Option<String>,
}

impl TalkConfigUiSubsetSchema {
    pub fn validate(&self) -> Result<(), String> {
        Ok(())
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TalkConfigResultConfigSchema {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub talk: Option<TalkConfigSchema>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub session: Option<TalkConfigSessionSubsetSchema>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ui: Option<TalkConfigUiSubsetSchema>,
}

impl TalkConfigResultConfigSchema {
    pub fn validate(&self) -> Result<(), String> {
        if let Some(t) = &self.talk {
            t.validate().map_err(|e| format!("talk: {}", e))?;
        }
        if let Some(s) = &self.session {
            s.validate().map_err(|e| format!("session: {}", e))?;
        }
        if let Some(u) = &self.ui {
            u.validate().map_err(|e| format!("ui: {}", e))?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TalkConfigResultSchema {
    pub config: TalkConfigResultConfigSchema,
}

impl TalkConfigResultSchema {
    pub fn validate(&self) -> Result<(), String> {
        self.config.validate().map_err(|e| format!("config: {}", e))?;
        Ok(())
    }
}

// ---------- TalkSpeakResultSchema ----------

/// Text-to-speech result with encoded audio and provider output metadata.
/// 对齐 TS:
///   `Type.Object({
///      audioBase64:    NonEmptyString,
///      provider:       NonEmptyString,
///      outputFormat:   Type.Optional(Type.String()),
///      voiceCompatible:Type.Optional(Type.Boolean()),
///      mimeType:       Type.Optional(Type.String()),
///      fileExtension:  Type.Optional(Type.String()),
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TalkSpeakResultSchema {
    pub audio_base64: String,
    pub provider: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output_format: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub voice_compatible: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mime_type: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file_extension: Option<String>,
}

impl TalkSpeakResultSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_non_empty_string("audioBase64", &self.audio_base64)?;
        validate_non_empty_string("provider", &self.provider)?;
        Ok(())
    }
}

// ---------- TtsSpeakResultSchema ----------

/// Text-to-speech result for `tts.speak` with encoded audio and provider metadata.
/// 对齐 TS:
///   `Type.Object({
///      audioBase64:   NonEmptyString,
///      provider:      NonEmptyString,
///      outputFormat:  Type.Optional(Type.String()),
///      mimeType:      Type.Optional(Type.String()),
///      fileExtension: Type.Optional(Type.String()),
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TtsSpeakResultSchema {
    pub audio_base64: String,
    pub provider: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output_format: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mime_type: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file_extension: Option<String>,
}

impl TtsSpeakResultSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_non_empty_string("audioBase64", &self.audio_base64)?;
        validate_non_empty_string("provider", &self.provider)?;
        Ok(())
    }
}

// ============================================================================
// Channel status / lifecycle schemas
// ============================================================================

// ---------- ChannelsStatusParamsSchema ----------

/// Channel status request, optionally probing one channel before returning.
/// 对齐 TS:
///   `Type.Object({
///      probe:     Type.Optional(Type.Boolean()),
///      timeoutMs: Type.Optional(Type.Integer({ minimum: 0 })),
///      channel:   Type.Optional(NonEmptyString),
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChannelsStatusParamsSchema {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub probe: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub timeout_ms: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub channel: Option<String>,
}

impl ChannelsStatusParamsSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_optional_non_negative_integer("timeoutMs", self.timeout_ms)?;
        validate_optional_non_empty_string("channel", self.channel.as_deref())?;
        Ok(())
    }
}

// ---------- ChannelAccountSnapshotSchema (additionalProperties: true) ----------

/// Per-account status snapshot for channel docking.
///
/// This is intentionally schema-light so new channel-specific metadata can ship
/// without a gateway protocol update; known fields stay documented for UI use.
/// 对齐 TS:
///   `Type.Object({
///      accountId:                NonEmptyString,
///      name:                     Type.Optional(Type.String()),
///      enabled:                  Type.Optional(Type.Boolean()),
///      configured:               Type.Optional(Type.Boolean()),
///      linked:                   Type.Optional(Type.Boolean()),
///      running:                  Type.Optional(Type.Boolean()),
///      connected:                Type.Optional(Type.Boolean()),
///      reconnectAttempts:        Type.Optional(Type.Integer({ minimum: 0 })),
///      lastConnectedAt:          Type.Optional(Type.Integer({ minimum: 0 })),
///      lastError:                Type.Optional(Type.String()),
///      healthState:              Type.Optional(Type.String()),
///      lastStartAt:              Type.Optional(Type.Integer({ minimum: 0 })),
///      lastStopAt:               Type.Optional(Type.Integer({ minimum: 0 })),
///      lastInboundAt:            Type.Optional(Type.Integer({ minimum: 0 })),
///      lastOutboundAt:           Type.Optional(Type.Integer({ minimum: 0 })),
///      lastTransportActivityAt:  Type.Optional(Type.Integer({ minimum: 0 })),
///      busy:                     Type.Optional(Type.Boolean()),
///      activeRuns:               Type.Optional(Type.Integer({ minimum: 0 })),
///      lastRunActivityAt:        Type.Optional(Type.Integer({ minimum: 0 })),
///      lastProbeAt:              Type.Optional(Type.Integer({ minimum: 0 })),
///      mode:                     Type.Optional(Type.String()),
///      dmPolicy:                 Type.Optional(Type.String()),
///      allowFrom:                Type.Optional(Type.Array(Type.String())),
///      tokenSource:              Type.Optional(Type.String()),
///      botTokenSource:           Type.Optional(Type.String()),
///      appTokenSource:           Type.Optional(Type.String()),
///      baseUrl:                  Type.Optional(Type.String()),
///      allowUnmentionedGroups:   Type.Optional(Type.Boolean()),
///      cliPath:                  Type.Optional(Type.Union([Type.String(), Type.Null()])),
///      dbPath:                   Type.Optional(Type.Union([Type.String(), Type.Null()])),
///      port:                     Type.Optional(Type.Union([Type.Integer({ minimum: 0 }), Type.Null()])),
///      probe:                    Type.Optional(Type.Unknown()),
///      audit:                    Type.Optional(Type.Unknown()),
///      application:              Type.Optional(Type.Unknown()),
///   }, { additionalProperties: true })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChannelAccountSnapshotSchema {
    pub account_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub configured: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub linked: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub running: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub connected: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reconnect_attempts: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_connected_at: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_error: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub health_state: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_start_at: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_stop_at: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_inbound_at: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_outbound_at: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_transport_activity_at: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub busy: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub active_runs: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_run_activity_at: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_probe_at: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mode: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dm_policy: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub allow_from: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub token_source: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bot_token_source: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub app_token_source: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub base_url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub allow_unmentioned_groups: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cli_path: Option<NullableString>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub db_path: Option<NullableString>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub port: Option<NullableNonNegativeInteger>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub probe: Option<serde_json::Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub audit: Option<serde_json::Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub application: Option<serde_json::Value>,
    #[serde(default, flatten)]
    pub extra: BTreeMap<String, serde_json::Value>,
}

impl ChannelAccountSnapshotSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_non_empty_string("accountId", &self.account_id)?;
        if let Some(n) = self.reconnect_attempts {
            validate_non_negative_integer("reconnectAttempts", n)?;
        }
        if let Some(n) = self.last_connected_at {
            validate_non_negative_integer("lastConnectedAt", n)?;
        }
        if let Some(n) = self.last_start_at {
            validate_non_negative_integer("lastStartAt", n)?;
        }
        if let Some(n) = self.last_stop_at {
            validate_non_negative_integer("lastStopAt", n)?;
        }
        if let Some(n) = self.last_inbound_at {
            validate_non_negative_integer("lastInboundAt", n)?;
        }
        if let Some(n) = self.last_outbound_at {
            validate_non_negative_integer("lastOutboundAt", n)?;
        }
        if let Some(n) = self.last_transport_activity_at {
            validate_non_negative_integer("lastTransportActivityAt", n)?;
        }
        if let Some(n) = self.active_runs {
            validate_non_negative_integer("activeRuns", n)?;
        }
        if let Some(n) = self.last_run_activity_at {
            validate_non_negative_integer("lastRunActivityAt", n)?;
        }
        if let Some(n) = self.last_probe_at {
            validate_non_negative_integer("lastProbeAt", n)?;
        }
        Ok(())
    }
}

/// 对齐 TS: `Type.Union([Type.String(), Type.Null()])` for nullable string fields.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum NullableString {
    Null,
    Value(String),
}

/// 对齐 TS: `Type.Union([Type.Integer({ minimum: 0 }), Type.Null()])` for
/// nullable port values.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum NullableNonNegativeInteger {
    Null,
    Value(i64),
}

impl NullableNonNegativeInteger {
    pub fn validate(&self) -> Result<(), String> {
        if let Self::Value(n) = self {
            validate_non_negative_integer("port", *n)?;
        }
        Ok(())
    }
}

// ---------- ChannelUiMetaSchema ----------

/// UI label and icon metadata for one channel.
/// 对齐 TS:
///   `Type.Object({
///      id:           NonEmptyString,
///      label:        NonEmptyString,
///      detailLabel:  NonEmptyString,
///      systemImage:  Type.Optional(Type.String()),
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChannelUiMetaSchema {
    pub id: String,
    pub label: String,
    pub detail_label: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub system_image: Option<String>,
}

impl ChannelUiMetaSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_non_empty_string("id", &self.id)?;
        validate_non_empty_string("label", &self.label)?;
        validate_non_empty_string("detailLabel", &self.detail_label)?;
        Ok(())
    }
}

// ---------- ChannelEventLoopHealthSchema ----------

/// Event-loop health snapshot included with channel status responses.
/// 对齐 TS:
///   `Type.Object({
///      degraded:      Type.Boolean(),
///      reasons:       Type.Array(
///        Type.Union([
///          Type.Literal("event_loop_delay"),
///          Type.Literal("event_loop_utilization"),
///          Type.Literal("cpu"),
///        ]),
///      ),
///      intervalMs:    Type.Integer({ minimum: 0 }),
///      delayP99Ms:    Type.Number({ minimum: 0 }),
///      delayMaxMs:    Type.Number({ minimum: 0 }),
///      utilization:   Type.Number({ minimum: 0 }),
///      cpuCoreRatio:  Type.Number({ minimum: 0 }),
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChannelEventLoopHealthSchema {
    pub degraded: bool,
    pub reasons: Vec<ChannelEventLoopReasonSchema>,
    pub interval_ms: i64,
    pub delay_p99_ms: f64,
    pub delay_max_ms: f64,
    pub utilization: f64,
    pub cpu_core_ratio: f64,
}

impl ChannelEventLoopHealthSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_non_negative_integer("intervalMs", self.interval_ms)?;
        if self.delay_p99_ms < 0.0 {
            return Err(format!(
                "delayP99Ms: expected >= 0, got {}",
                self.delay_p99_ms
            ));
        }
        if self.delay_max_ms < 0.0 {
            return Err(format!(
                "delayMaxMs: expected >= 0, got {}",
                self.delay_max_ms
            ));
        }
        if self.utilization < 0.0 {
            return Err(format!(
                "utilization: expected >= 0, got {}",
                self.utilization
            ));
        }
        if self.cpu_core_ratio < 0.0 {
            return Err(format!(
                "cpuCoreRatio: expected >= 0, got {}",
                self.cpu_core_ratio
            ));
        }
        Ok(())
    }
}

// ---------- ChannelsStatusResultSchema ----------

/// Full channel status result for dashboard and operator diagnostics.
/// 对齐 TS:
///   `Type.Object({
///      ts:                    Type.Integer({ minimum: 0 }),
///      channelOrder:          Type.Array(NonEmptyString),
///      channelLabels:         Type.Record(NonEmptyString, NonEmptyString),
///      channelDetailLabels:   Type.Optional(Type.Record(NonEmptyString, NonEmptyString)),
///      channelSystemImages:   Type.Optional(Type.Record(NonEmptyString, NonEmptyString)),
///      channelMeta:           Type.Optional(Type.Array(ChannelUiMetaSchema)),
///      channels:              Type.Record(NonEmptyString, Type.Unknown()),
///      channelAccounts:       Type.Record(NonEmptyString, Type.Array(ChannelAccountSnapshotSchema)),
///      channelDefaultAccountId: Type.Record(NonEmptyString, NonEmptyString),
///      eventLoop:             Type.Optional(ChannelEventLoopHealthSchema),
///      partial:               Type.Optional(Type.Boolean()),
///      warnings:              Type.Optional(Type.Array(Type.String())),
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChannelsStatusResultSchema {
    pub ts: i64,
    pub channel_order: Vec<String>,
    pub channel_labels: HashMap<String, String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub channel_detail_labels: Option<HashMap<String, String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub channel_system_images: Option<HashMap<String, String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub channel_meta: Option<Vec<ChannelUiMetaSchema>>,
    pub channels: HashMap<String, serde_json::Value>,
    pub channel_accounts: HashMap<String, Vec<ChannelAccountSnapshotSchema>>,
    pub channel_default_account_id: HashMap<String, String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub event_loop: Option<ChannelEventLoopHealthSchema>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub partial: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub warnings: Option<Vec<String>>,
}

impl ChannelsStatusResultSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_non_negative_integer("ts", self.ts)?;
        validate_non_empty_string_list("channelOrder", &self.channel_order)?;
        if let Some(meta) = &self.channel_meta {
            for (i, m) in meta.iter().enumerate() {
                m.validate()
                    .map_err(|e| format!("channelMeta[{}]: {}", i, e))?;
            }
        }
        for (channel, accounts) in self.channel_accounts.iter() {
            for (i, acc) in accounts.iter().enumerate() {
                acc.validate().map_err(|e| {
                    format!("channelAccounts[{}][{}]: {}", channel, i, e)
                })?;
            }
        }
        if let Some(event_loop) = &self.event_loop {
            event_loop
                .validate()
                .map_err(|e| format!("eventLoop: {}", e))?;
        }
        Ok(())
    }
}

// ---------- Channel lifecycle (start/stop/logout) param schemas ----------

/// Logs out one channel account.
/// 对齐 TS: `Type.Object({
///   channel:   NonEmptyString,
///   accountId: Type.Optional(Type.String()),
/// }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChannelsLogoutParamsSchema {
    pub channel: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub account_id: Option<String>,
}

impl ChannelsLogoutParamsSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_non_empty_string("channel", &self.channel)?;
        Ok(())
    }
}

/// Stops one channel account runtime.
/// 对齐 TS: `Type.Object({
///   channel:   NonEmptyString,
///   accountId: Type.Optional(Type.String()),
/// }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChannelsStopParamsSchema {
    pub channel: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub account_id: Option<String>,
}

impl ChannelsStopParamsSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_non_empty_string("channel", &self.channel)?;
        Ok(())
    }
}

/// Starts one channel account runtime.
/// 对齐 TS: `Type.Object({
///   channel:   NonEmptyString,
///   accountId: Type.Optional(Type.String()),
/// }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChannelsStartParamsSchema {
    pub channel: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub account_id: Option<String>,
}

impl ChannelsStartParamsSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_non_empty_string("channel", &self.channel)?;
        Ok(())
    }
}

// ============================================================================
// Web login / QR schemas
// ============================================================================

// ---------- WebLoginStartParamsSchema ----------

/// Starts browser/web login for a channel account.
/// 对齐 TS:
///   `Type.Object({
///      force:     Type.Optional(Type.Boolean()),
///      timeoutMs: Type.Optional(Type.Integer({ minimum: 0 })),
///      verbose:   Type.Optional(Type.Boolean()),
///      accountId: Type.Optional(Type.String()),
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WebLoginStartParamsSchema {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub force: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub timeout_ms: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub verbose: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub account_id: Option<String>,
}

impl WebLoginStartParamsSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_optional_non_negative_integer("timeoutMs", self.timeout_ms)?;
        Ok(())
    }
}

// ---------- QrDataUrlSchema ----------

/// Inline data URL holding a base64 PNG QR code.
/// 对齐 TS:
///   `Type.String({ maxLength: 16_384, pattern: "^data:image/png;base64," })`.
pub type QrDataUrlSchema = String;

/// Returns true when the value matches the QrDataUrl grammar.
pub fn is_valid_qr_data_url(s: &str) -> bool {
    s.len() <= QR_DATA_URL_MAX_LENGTH && regex(QR_DATA_URL_PATTERN).is_match(s)
}

fn validate_qr_data_url(field: &str, s: &str) -> Result<(), String> {
    if !is_valid_qr_data_url(s) {
        return Err(format!(
            "{}: expected data URL matching {} (length <= {}), got length {}",
            field, QR_DATA_URL_PATTERN, QR_DATA_URL_MAX_LENGTH, s.len()
        ));
    }
    Ok(())
}

// ---------- WebLoginWaitParamsSchema ----------

/// Waits for web login completion or the next QR code.
/// 对齐 TS:
///   `Type.Object({
///      timeoutMs:        Type.Optional(Type.Integer({ minimum: 0 })),
///      accountId:        Type.Optional(Type.String()),
///      currentQrDataUrl: Type.Optional(QrDataUrlSchema),
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WebLoginWaitParamsSchema {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub timeout_ms: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub account_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub current_qr_data_url: Option<QrDataUrlSchema>,
}

impl WebLoginWaitParamsSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_optional_non_negative_integer("timeoutMs", self.timeout_ms)?;
        if let Some(qr) = &self.current_qr_data_url {
            validate_qr_data_url("currentQrDataUrl", qr)?;
        }
        Ok(())
    }
}

// ============================================================================
// Wire type aliases (对应 TS `export type X = Static<typeof XSchema>`)
// ============================================================================

// 对应 TS:
//   export type TalkEvent                        = Static<typeof TalkEventSchema>;
//   export type TalkModeParams                   = Static<typeof TalkModeParamsSchema>;
//   export type TalkCatalogParams                = Static<typeof TalkCatalogParamsSchema>;
//   export type TalkCatalogResult                = Static<typeof TalkCatalogResultSchema>;
//   export type TalkConfigParams                 = Static<typeof TalkConfigParamsSchema>;
//   export type TalkConfigResult                 = Static<typeof TalkConfigResultSchema>;
//   export type TalkClientCreateParams           = Static<typeof TalkClientCreateParamsSchema>;
//   export type TalkClientCreateResult           = Static<typeof TalkClientCreateResultSchema>;
//   export type TalkClientSteerParams            = Static<typeof TalkClientSteerParamsSchema>;
//   export type TalkAgentControlResult           = Static<typeof TalkAgentControlResultSchema>;
//   export type TalkClientToolCallParams         = Static<typeof TalkClientToolCallParamsSchema>;
//   export type TalkClientToolCallResult         = Static<typeof TalkClientToolCallResultSchema>;
//   export type TalkSessionCreateParams          = Static<typeof TalkSessionCreateParamsSchema>;
//   export type TalkSessionCreateResult          = Static<typeof TalkSessionCreateResultSchema>;
//   export type TalkSessionJoinParams            = Static<typeof TalkSessionJoinParamsSchema>;
//   export type TalkSessionJoinResult            = Static<typeof TalkSessionJoinResultSchema>;
//   export type TalkSessionAppendAudioParams     = Static<typeof TalkSessionAppendAudioParamsSchema>;
//   export type TalkSessionTurnParams            = Static<typeof TalkSessionTurnParamsSchema>;
//   export type TalkSessionCancelTurnParams      = Static<typeof TalkSessionCancelTurnParamsSchema>;
//   export type TalkSessionCancelOutputParams    = Static<typeof TalkSessionCancelOutputParamsSchema>;
//   export type TalkSessionTurnResult            = Static<typeof TalkSessionTurnResultSchema>;
//   export type TalkSessionSteerParams           = Static<typeof TalkSessionSteerParamsSchema>;
//   export type TalkSessionSubmitToolResultParams = Static<typeof TalkSessionSubmitToolResultParamsSchema>;
//   export type TalkSessionCloseParams           = Static<typeof TalkSessionCloseParamsSchema>;
//   export type TalkSessionOkResult              = Static<typeof TalkSessionOkResultSchema>;
//   export type TalkSpeakParams                  = Static<typeof TalkSpeakParamsSchema>;
//   export type TalkSpeakResult                  = Static<typeof TalkSpeakResultSchema>;
//   export type TtsSpeakParams                   = Static<typeof TtsSpeakParamsSchema>;
//   export type TtsSpeakResult                   = Static<typeof TtsSpeakResultSchema>;
//   export type ChannelsStatusParams             = Static<typeof ChannelsStatusParamsSchema>;
//   export type ChannelsStatusResult             = Static<typeof ChannelsStatusResultSchema>;
//   export type ChannelsStartParams              = Static<typeof ChannelsStartParamsSchema>;
//   export type ChannelsStopParams               = Static<typeof ChannelsStopParamsSchema>;
//   export type ChannelsLogoutParams             = Static<typeof ChannelsLogoutParamsSchema>;
//   export type WebLoginStartParams              = Static<typeof WebLoginStartParamsSchema>;
//   export type WebLoginWaitParams               = Static<typeof WebLoginWaitParamsSchema>;
pub type TalkEvent = TalkEventSchema;
pub type TalkModeParams = TalkModeParamsSchema;
pub type TalkCatalogParams = TalkCatalogParamsSchema;
pub type TalkCatalogResult = TalkCatalogResultSchema;
pub type TalkConfigParams = TalkConfigParamsSchema;
pub type TalkConfigResult = TalkConfigResultSchema;
pub type TalkClientCreateParams = TalkClientCreateParamsSchema;
pub type TalkClientCreateResult = TalkClientCreateResultSchema;
pub type TalkClientSteerParams = TalkClientSteerParamsSchema;
pub type TalkAgentControlResult = TalkAgentControlResultSchema;
pub type TalkClientToolCallParams = TalkClientToolCallParamsSchema;
pub type TalkClientToolCallResult = TalkClientToolCallResultSchema;
pub type TalkSessionCreateParams = TalkSessionCreateParamsSchema;
pub type TalkSessionCreateResult = TalkSessionCreateResultSchema;
pub type TalkSessionJoinParams = TalkSessionJoinParamsSchema;
pub type TalkSessionJoinResult = TalkSessionJoinResultSchema;
pub type TalkSessionAppendAudioParams = TalkSessionAppendAudioParamsSchema;
pub type TalkSessionTurnParams = TalkSessionTurnParamsSchema;
pub type TalkSessionCancelTurnParams = TalkSessionCancelTurnParamsSchema;
pub type TalkSessionCancelOutputParams = TalkSessionCancelOutputParamsSchema;
pub type TalkSessionTurnResult = TalkSessionTurnResultSchema;
pub type TalkSessionSteerParams = TalkSessionSteerParamsSchema;
pub type TalkSessionSubmitToolResultParams = TalkSessionSubmitToolResultParamsSchema;
pub type TalkSessionCloseParams = TalkSessionCloseParamsSchema;
pub type TalkSessionOkResult = TalkSessionOkResultSchema;
pub type TalkSpeakParams = TalkSpeakParamsSchema;
pub type TalkSpeakResult = TalkSpeakResultSchema;
pub type TtsSpeakParams = TtsSpeakParamsSchema;
pub type TtsSpeakResult = TtsSpeakResultSchema;
pub type ChannelsStatusParams = ChannelsStatusParamsSchema;
pub type ChannelsStatusResult = ChannelsStatusResultSchema;
pub type ChannelsStartParams = ChannelsStartParamsSchema;
pub type ChannelsStopParams = ChannelsStopParamsSchema;
pub type ChannelsLogoutParams = ChannelsLogoutParamsSchema;
pub type WebLoginStartParams = WebLoginStartParamsSchema;
pub type WebLoginWaitParams = WebLoginWaitParamsSchema;
