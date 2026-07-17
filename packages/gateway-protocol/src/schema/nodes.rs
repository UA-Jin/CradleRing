// Gateway Protocol schema: nodes.
// 翻译自 packages/gateway-protocol/src/schema/nodes.ts
//
// Node presence, pairing, plugin/tool/skill catalog, command invocation,
// event envelopes, and pending work drain/enqueue schemas.
//
// TS 用 TypeBox 定义 schema（运行时验证 + 类型）。
// Rust 用 serde struct + 验证函数实现等价的序列化/反序列化语义。

use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::primitives::NonEmptyString;

// ---------- 模块私有常量 ----------

/// 对齐 TS: `Type.String({ minLength: 1, maxLength: 64, pattern: "^[A-Za-z][A-Za-z0-9_-]{0,63}$" })`.
pub const NODE_PLUGIN_TOOL_NAME_MAX_LENGTH: usize = 64;
pub const NODE_PLUGIN_TOOL_NAME_MIN_LENGTH: usize = 1;
const NODE_PLUGIN_TOOL_NAME_PATTERN: &str = r"^[A-Za-z][A-Za-z0-9_-]{0,63}$";

/// 对齐 TS: `Type.String({ minLength: 1, maxLength: 64, pattern: "^(?!.*--)[a-z0-9](?:[a-z0-9-]{0,62}[a-z0-9])?$" })`.
pub const NODE_SKILL_NAME_MAX_LENGTH: usize = 64;
pub const NODE_SKILL_NAME_MIN_LENGTH: usize = 1;
const NODE_SKILL_NAME_PATTERN: &str = r"^(?!.*--)[a-z0-9](?:[a-z0-9-]{0,62}[a-z0-9])?$";

/// 对齐 TS: `Type.String({ minLength: 1, maxLength: 1024 })`.
pub const NODE_SKILL_DESCRIPTION_MAX_LENGTH: usize = 1024;
pub const NODE_SKILL_DESCRIPTION_MIN_LENGTH: usize = 1;

/// 对齐 TS: `Type.String({ minLength: 1, maxLength: 64 * 1024 })`.
pub const NODE_SKILL_CONTENT_MAX_LENGTH: usize = 64 * 1024;
pub const NODE_SKILL_CONTENT_MIN_LENGTH: usize = 1;

/// 对齐 TS: `Type.Integer({ minimum: 0, maximum: 2_592_000 })`.
pub const NODE_PRESENCE_IDLE_SECONDS_MAX: i64 = 2_592_000;

/// 对齐 TS: `Type.Integer({ minimum: 1, maximum: 10 })`.
pub const NODE_PENDING_DRAIN_MAX_ITEMS_MAX: i64 = 10;
pub const NODE_PENDING_DRAIN_MAX_ITEMS_MIN: i64 = 1;

/// 对齐 TS: `Type.Array(NodeSkillDescriptorSchema, { maxItems: 64 })`.
pub const NODE_SKILL_LIST_MAX_ITEMS: usize = 64;

// ---------- 基础验证原语 ----------

fn regex(pattern: &str) -> regex::Regex {
    regex::Regex::new(pattern).expect("invalid regex pattern compiled into nodes")
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

fn validate_non_empty_string_list(field: &str, values: &[String]) -> Result<(), String> {
    for (i, v) in values.iter().enumerate() {
        validate_non_empty_string(&format!("{}[{}]", field, i), v)?;
    }
    Ok(())
}

fn validate_non_negative_integer(field: &str, value: i64) -> Result<(), String> {
    if value < 0 {
        return Err(format!("{}: expected >= 0, got {}", field, value));
    }
    Ok(())
}

/// Returns true when `name` matches the node plugin tool name grammar.
pub fn is_valid_node_plugin_tool_name(name: &str) -> bool {
    let len = name.chars().count();
    if len < NODE_PLUGIN_TOOL_NAME_MIN_LENGTH || len > NODE_PLUGIN_TOOL_NAME_MAX_LENGTH {
        return false;
    }
    regex(NODE_PLUGIN_TOOL_NAME_PATTERN).is_match(name)
}

/// Returns true when `name` matches the node skill name grammar.
pub fn is_valid_node_skill_name(name: &str) -> bool {
    let len = name.chars().count();
    if len < NODE_SKILL_NAME_MIN_LENGTH || len > NODE_SKILL_NAME_MAX_LENGTH {
        return false;
    }
    regex(NODE_SKILL_NAME_PATTERN).is_match(name)
}

// ---------- NodePluginToolNameSchema / NodeSkillNameSchema ----------

/// Bounded plugin tool name grammar.
/// 对齐 TS: `NodePluginToolNameSchema = Type.String({ minLength: 1, maxLength: 64, pattern: ... })`.
pub type NodePluginToolNameSchema = String;

/// Bounded skill name grammar.
/// 对齐 TS: `NodeSkillNameSchema = Type.String({ minLength: 1, maxLength: 64, pattern: ... })`.
pub type NodeSkillNameSchema = String;

// ---------- NodePendingWorkTypeSchema ----------

/// Closed enumeration of pending node work classes.
/// 对齐 TS: `Type.String({ enum: ["status.request", "location.request"] })`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum NodePendingWorkTypeSchema {
    #[serde(rename = "status.request")]
    StatusRequest,
    #[serde(rename = "location.request")]
    LocationRequest,
}

impl NodePendingWorkTypeSchema {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::StatusRequest => "status.request",
            Self::LocationRequest => "location.request",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "status.request" => Some(Self::StatusRequest),
            "location.request" => Some(Self::LocationRequest),
            _ => None,
        }
    }

    pub fn all() -> &'static [NodePendingWorkTypeSchema] {
        &[Self::StatusRequest, Self::LocationRequest]
    }
}

pub fn is_valid_node_pending_work_type(s: &str) -> bool {
    NodePendingWorkTypeSchema::from_str(s).is_some()
}

// ---------- NodePendingWorkPrioritySchema ----------

/// Closed enumeration of pending node work priorities.
/// 对齐 TS: `Type.String({ enum: ["normal", "high"] })`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum NodePendingWorkPrioritySchema {
    #[serde(rename = "normal")]
    Normal,
    #[serde(rename = "high")]
    High,
}

impl NodePendingWorkPrioritySchema {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Normal => "normal",
            Self::High => "high",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "normal" => Some(Self::Normal),
            "high" => Some(Self::High),
            _ => None,
        }
    }

    pub fn all() -> &'static [NodePendingWorkPrioritySchema] {
        &[Self::Normal, Self::High]
    }
}

pub fn is_valid_node_pending_work_priority(s: &str) -> bool {
    NodePendingWorkPrioritySchema::from_str(s).is_some()
}

// ---------- NodePendingWorkDrainPrioritySchema ----------

/// Drain responses also report `default` as a third priority value alongside
/// `normal`/`high`. Modeled separately so enqueue stays closed-set.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum NodePendingDrainPrioritySchema {
    #[serde(rename = "default")]
    Default,
    #[serde(rename = "normal")]
    Normal,
    #[serde(rename = "high")]
    High,
}

impl NodePendingDrainPrioritySchema {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Default => "default",
            Self::Normal => "normal",
            Self::High => "high",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "default" => Some(Self::Default),
            "normal" => Some(Self::Normal),
            "high" => Some(Self::High),
            _ => None,
        }
    }
}

pub fn is_valid_node_pending_drain_priority(s: &str) -> bool {
    NodePendingDrainPrioritySchema::from_str(s).is_some()
}

// ---------- NodePresenceAliveReasonSchema ----------

/// Reasons a node can report itself alive without implying an operator action.
/// 对齐 TS: `Type.String({ enum: [...] })`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum NodePresenceAliveReasonSchema {
    #[serde(rename = "background")]
    Background,
    #[serde(rename = "silent_push")]
    SilentPush,
    #[serde(rename = "bg_app_refresh")]
    BgAppRefresh,
    #[serde(rename = "significant_location")]
    SignificantLocation,
    #[serde(rename = "manual")]
    Manual,
    #[serde(rename = "connect")]
    Connect,
}

impl NodePresenceAliveReasonSchema {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Background => "background",
            Self::SilentPush => "silent_push",
            Self::BgAppRefresh => "bg_app_refresh",
            Self::SignificantLocation => "significant_location",
            Self::Manual => "manual",
            Self::Connect => "connect",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "background" => Some(Self::Background),
            "silent_push" => Some(Self::SilentPush),
            "bg_app_refresh" => Some(Self::BgAppRefresh),
            "significant_location" => Some(Self::SignificantLocation),
            "manual" => Some(Self::Manual),
            "connect" => Some(Self::Connect),
            _ => None,
        }
    }

    pub fn all() -> &'static [NodePresenceAliveReasonSchema] {
        &[
            Self::Background,
            Self::SilentPush,
            Self::BgAppRefresh,
            Self::SignificantLocation,
            Self::Manual,
            Self::Connect,
        ]
    }
}

pub fn is_valid_node_presence_alive_reason(s: &str) -> bool {
    NodePresenceAliveReasonSchema::from_str(s).is_some()
}

// ---------- NodePresenceAlivePayloadSchema ----------

/// Presence heartbeat payload sent by remote nodes to refresh gateway state.
/// 对齐 TS:
///   `Type.Object({
///       trigger:        NodePresenceAliveReasonSchema,
///       sentAtMs:       Type.Optional(Type.Integer({ minimum: 0 })),
///       displayName:    Type.Optional(NonEmptyString),
///       version:        Type.Optional(NonEmptyString),
///       platform:       Type.Optional(NonEmptyString),
///       deviceFamily:   Type.Optional(NonEmptyString),
///       modelIdentifier:Type.Optional(NonEmptyString),
///       pushTransport:  Type.Optional(NonEmptyString),
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NodePresenceAlivePayloadSchema {
    pub trigger: NodePresenceAliveReasonSchema,
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "sentAtMs")]
    pub sent_at_ms: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "displayName")]
    pub display_name: Option<NonEmptyString>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: Option<NonEmptyString>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub platform: Option<NonEmptyString>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "deviceFamily"
    )]
    pub device_family: Option<NonEmptyString>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "modelIdentifier"
    )]
    pub model_identifier: Option<NonEmptyString>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "pushTransport"
    )]
    pub push_transport: Option<NonEmptyString>,
}

impl NodePresenceAlivePayloadSchema {
    pub fn validate(&self) -> Result<(), String> {
        if let Some(ts) = self.sent_at_ms {
            validate_non_negative_integer("sentAtMs", ts)?;
        }
        if let Some(s) = &self.display_name {
            validate_non_empty_string("displayName", s)?;
        }
        if let Some(s) = &self.version {
            validate_non_empty_string("version", s)?;
        }
        if let Some(s) = &self.platform {
            validate_non_empty_string("platform", s)?;
        }
        if let Some(s) = &self.device_family {
            validate_non_empty_string("deviceFamily", s)?;
        }
        if let Some(s) = &self.model_identifier {
            validate_non_empty_string("modelIdentifier", s)?;
        }
        if let Some(s) = &self.push_transport {
            validate_non_empty_string("pushTransport", s)?;
        }
        Ok(())
    }
}

// ---------- NodePresenceActivityPayloadSchema ----------

/// Recent operator input activity reported by an interactive node.
/// 对齐 TS:
///   `Type.Object({
///       idleSeconds: Type.Integer({ minimum: 0, maximum: 2_592_000 }),
///       saturated:   Type.Optional(Type.Boolean()),
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NodePresenceActivityPayloadSchema {
    #[serde(rename = "idleSeconds")]
    pub idle_seconds: i64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub saturated: Option<bool>,
}

impl NodePresenceActivityPayloadSchema {
    pub fn validate(&self) -> Result<(), String> {
        if self.idle_seconds < 0 || self.idle_seconds > NODE_PRESENCE_IDLE_SECONDS_MAX {
            return Err(format!(
                "idleSeconds: expected [0, {}], got {}",
                NODE_PRESENCE_IDLE_SECONDS_MAX, self.idle_seconds
            ));
        }
        Ok(())
    }
}

// ---------- NodeEventResultSchema ----------

/// Normalized result for node-originated events after gateway dispatch.
/// 对齐 TS:
///   `Type.Object({
///       ok:      Type.Boolean(),
///       event:   NonEmptyString,
///       handled: Type.Boolean(),
///       reason:  Type.Optional(NonEmptyString),
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeEventResultSchema {
    pub ok: bool,
    pub event: NonEmptyString,
    pub handled: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<NonEmptyString>,
}

impl NodeEventResultSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_non_empty_string("event", &self.event)?;
        if let Some(r) = &self.reason {
            validate_non_empty_string("reason", r)?;
        }
        Ok(())
    }
}

// ---------- NodePairListParamsSchema ----------

/// Lists pending node-pairing requests.
/// 对齐 TS: `Type.Object({}, { additionalProperties: false })`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct NodePairListParamsSchema {}

impl NodePairListParamsSchema {
    pub fn validate(&self) -> Result<(), String> {
        Ok(())
    }
}

// ---------- NodePairApproveParamsSchema ----------

/// Approves a pending node-pairing request by request id.
/// 对齐 TS: `Type.Object({ requestId: NonEmptyString }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NodePairApproveParamsSchema {
    #[serde(rename = "requestId")]
    pub request_id: NonEmptyString,
}

impl NodePairApproveParamsSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_non_empty_string("requestId", &self.request_id)?;
        Ok(())
    }
}

// ---------- NodePairRejectParamsSchema ----------

/// Rejects a pending node-pairing request by request id.
/// 对齐 TS: `Type.Object({ requestId: NonEmptyString }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NodePairRejectParamsSchema {
    #[serde(rename = "requestId")]
    pub request_id: NonEmptyString,
}

impl NodePairRejectParamsSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_non_empty_string("requestId", &self.request_id)?;
        Ok(())
    }
}

// ---------- NodePairRemoveParamsSchema ----------

/// Removes an already paired node from the gateway trust set.
/// 对齐 TS: `Type.Object({ nodeId: NonEmptyString }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NodePairRemoveParamsSchema {
    #[serde(rename = "nodeId")]
    pub node_id: NonEmptyString,
}

impl NodePairRemoveParamsSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_non_empty_string("nodeId", &self.node_id)?;
        Ok(())
    }
}

// ---------- NodeRenameParamsSchema ----------

/// Renames a paired node while preserving its stable node id.
/// 对齐 TS: `Type.Object({ nodeId, displayName }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NodeRenameParamsSchema {
    #[serde(rename = "nodeId")]
    pub node_id: NonEmptyString,
    #[serde(rename = "displayName")]
    pub display_name: NonEmptyString,
}

impl NodeRenameParamsSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_non_empty_string("nodeId", &self.node_id)?;
        validate_non_empty_string("displayName", &self.display_name)?;
        Ok(())
    }
}

// ---------- NodeListParamsSchema ----------

/// Lists paired nodes known to the gateway.
/// 对齐 TS: `Type.Object({}, { additionalProperties: false })`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct NodeListParamsSchema {}

impl NodeListParamsSchema {
    pub fn validate(&self) -> Result<(), String> {
        Ok(())
    }
}

// ---------- NodePluginToolMcpSchema ----------

/// MCP tool routing block attached to a plugin tool descriptor.
/// 对齐 TS:
///   `Type.Object({
///       server: NonEmptyString,
///       tool:   NonEmptyString,
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodePluginToolMcpSchema {
    pub server: NonEmptyString,
    pub tool: NonEmptyString,
}

impl NodePluginToolMcpSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_non_empty_string("server", &self.server)?;
        validate_non_empty_string("tool", &self.tool)?;
        Ok(())
    }
}

// ---------- NodePluginToolDescriptorSchema ----------

/// Agent-visible tool descriptor advertised by a connected node.
/// 对齐 TS:
///   `Type.Object({
///       pluginId:    NonEmptyString,
///       name:        NodePluginToolNameSchema,
///       description: NonEmptyString,
///       parameters:  Type.Optional(Type.Record(Type.String(), Type.Unknown())),
///       command:     Type.Optional(NonEmptyString),
///       mcp:         Type.Optional(Type.Object({ server, tool }, { additionalProperties: false })),
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NodePluginToolDescriptorSchema {
    #[serde(rename = "pluginId")]
    pub plugin_id: NonEmptyString,
    pub name: NodePluginToolNameSchema,
    pub description: NonEmptyString,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parameters: Option<std::collections::BTreeMap<String, Value>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub command: Option<NonEmptyString>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mcp: Option<NodePluginToolMcpSchema>,
}

impl NodePluginToolDescriptorSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_non_empty_string("pluginId", &self.plugin_id)?;
        if !is_valid_node_plugin_tool_name(&self.name) {
            return Err(format!("name: invalid plugin tool name: {:?}", self.name));
        }
        validate_non_empty_string("description", &self.description)?;
        if let Some(c) = &self.command {
            validate_non_empty_string("command", c)?;
        }
        if let Some(m) = &self.mcp {
            m.validate().map_err(|e| format!("mcp: {}", e))?;
        }
        Ok(())
    }
}

// ---------- NodePluginToolsUpdateParamsSchema ----------

/// Replaces the connected node's dynamic agent-visible plugin/MCP tool catalog.
/// 对齐 TS:
///   `Type.Object({
///       tools: Type.Array(NodePluginToolDescriptorSchema),
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodePluginToolsUpdateParamsSchema {
    pub tools: Vec<NodePluginToolDescriptorSchema>,
}

impl NodePluginToolsUpdateParamsSchema {
    pub fn validate(&self) -> Result<(), String> {
        for (i, t) in self.tools.iter().enumerate() {
            t.validate().map_err(|e| format!("tools[{}]: {}", i, e))?;
        }
        Ok(())
    }
}

// ---------- NodeSkillDescriptorSchema ----------

/// Agent-visible skill descriptor advertised by a connected node.
/// 对齐 TS:
///   `Type.Object({
///       name:        NodeSkillNameSchema,
///       description: Type.String({ minLength: 1, maxLength: 1024 }),
///       content:     Type.String({ minLength: 1, maxLength: 64 * 1024 }),
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeSkillDescriptorSchema {
    pub name: NodeSkillNameSchema,
    pub description: String,
    pub content: String,
}

impl NodeSkillDescriptorSchema {
    pub fn validate(&self) -> Result<(), String> {
        if !is_valid_node_skill_name(&self.name) {
            return Err(format!("name: invalid skill name: {:?}", self.name));
        }
        let desc_len = self.description.chars().count();
        if desc_len < NODE_SKILL_DESCRIPTION_MIN_LENGTH
            || desc_len > NODE_SKILL_DESCRIPTION_MAX_LENGTH
        {
            return Err(format!(
                "description: expected length [{}, {}], got {}",
                NODE_SKILL_DESCRIPTION_MIN_LENGTH, NODE_SKILL_DESCRIPTION_MAX_LENGTH, desc_len
            ));
        }
        let content_len = self.content.chars().count();
        if content_len < NODE_SKILL_CONTENT_MIN_LENGTH
            || content_len > NODE_SKILL_CONTENT_MAX_LENGTH
        {
            return Err(format!(
                "content: expected length [{}, {}], got {}",
                NODE_SKILL_CONTENT_MIN_LENGTH, NODE_SKILL_CONTENT_MAX_LENGTH, content_len
            ));
        }
        Ok(())
    }
}

// ---------- NodeSkillsUpdateParamsSchema ----------

/// Replaces the connected node's agent-visible skill catalog.
/// 对齐 TS:
///   `Type.Object({
///       skills: Type.Array(NodeSkillDescriptorSchema, { maxItems: 64 }),
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeSkillsUpdateParamsSchema {
    pub skills: Vec<NodeSkillDescriptorSchema>,
}

impl NodeSkillsUpdateParamsSchema {
    pub fn validate(&self) -> Result<(), String> {
        if self.skills.len() > NODE_SKILL_LIST_MAX_ITEMS {
            return Err(format!(
                "skills: maxItems {}, got {}",
                NODE_SKILL_LIST_MAX_ITEMS,
                self.skills.len()
            ));
        }
        for (i, s) in self.skills.iter().enumerate() {
            s.validate().map_err(|e| format!("skills[{}]: {}", i, e))?;
        }
        Ok(())
    }
}

// ---------- NodePendingAckParamsSchema ----------

/// Acknowledges queued node work that the node has consumed.
/// 对齐 TS:
///   `Type.Object({
///       ids: Type.Array(NonEmptyString, { minItems: 1 }),
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodePendingAckParamsSchema {
    pub ids: Vec<NonEmptyString>,
}

impl NodePendingAckParamsSchema {
    pub fn validate(&self) -> Result<(), String> {
        if self.ids.is_empty() {
            return Err("ids: minItems 1, got 0".to_string());
        }
        validate_non_empty_string_list("ids", &self.ids)?;
        Ok(())
    }
}

// ---------- NodeDescribeParamsSchema ----------

/// Requests detailed metadata for one paired node.
/// 对齐 TS: `Type.Object({ nodeId: NonEmptyString }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NodeDescribeParamsSchema {
    #[serde(rename = "nodeId")]
    pub node_id: NonEmptyString,
}

impl NodeDescribeParamsSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_non_empty_string("nodeId", &self.node_id)?;
        Ok(())
    }
}

// ---------- NodeInvokeParamsSchema ----------

/// `turnSourceThreadId` may be a string or number.
/// 对齐 TS: `Type.Optional(Type.Union([Type.String(), Type.Number()]))`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum NodeInvokeThreadId {
    Text(String),
    Number(f64),
}

/// Invokes a command on a paired node; idempotency allows safe retries.
/// 对齐 TS:
///   `Type.Object({
///       nodeId:             NonEmptyString,
///       command:            NonEmptyString,
///       params:             Type.Optional(Type.Unknown()),
///       timeoutMs:          Type.Optional(Type.Integer({ minimum: 0 })),
///       idempotencyKey:     NonEmptyString,
///       turnSourceChannel:  Type.Optional(Type.String()),
///       turnSourceTo:       Type.Optional(Type.String()),
///       turnSourceAccountId:Type.Optional(Type.String()),
///       turnSourceThreadId: Type.Optional(Type.Union([Type.String(), Type.Number()])),
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NodeInvokeParamsSchema {
    #[serde(rename = "nodeId")]
    pub node_id: NonEmptyString,
    pub command: NonEmptyString,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub params: Option<Value>,
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "timeoutMs")]
    pub timeout_ms: Option<i64>,
    #[serde(rename = "idempotencyKey")]
    pub idempotency_key: NonEmptyString,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "turnSourceChannel"
    )]
    pub turn_source_channel: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "turnSourceTo")]
    pub turn_source_to: Option<String>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "turnSourceAccountId"
    )]
    pub turn_source_account_id: Option<String>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "turnSourceThreadId"
    )]
    pub turn_source_thread_id: Option<NodeInvokeThreadId>,
}

impl NodeInvokeParamsSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_non_empty_string("nodeId", &self.node_id)?;
        validate_non_empty_string("command", &self.command)?;
        if let Some(t) = self.timeout_ms {
            validate_non_negative_integer("timeoutMs", t)?;
        }
        validate_non_empty_string("idempotencyKey", &self.idempotency_key)?;
        Ok(())
    }
}

// ---------- NodeInvokeResultErrorSchema ----------

/// Error block attached to invocation result payloads.
/// 对齐 TS:
///   `Type.Object({
///       code:    Type.Optional(NonEmptyString),
///       message: Type.Optional(NonEmptyString),
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct NodeInvokeResultErrorSchema {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub code: Option<NonEmptyString>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message: Option<NonEmptyString>,
}

impl NodeInvokeResultErrorSchema {
    pub fn validate(&self) -> Result<(), String> {
        if let Some(c) = &self.code {
            validate_non_empty_string("code", c)?;
        }
        if let Some(m) = &self.message {
            validate_non_empty_string("message", m)?;
        }
        Ok(())
    }
}

// ---------- NodeInvokeResultParamsSchema ----------

/// Result callback payload for a node command invocation.
/// 对齐 TS:
///   `Type.Object({
///       id:         NonEmptyString,
///       nodeId:     NonEmptyString,
///       ok:         Type.Boolean(),
///       payload:    Type.Optional(Type.Unknown()),
///       payloadJSON:Type.Optional(Type.String()),
///       error:      Type.Optional(Type.Object({ code, message }, { additionalProperties: false })),
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NodeInvokeResultParamsSchema {
    pub id: NonEmptyString,
    #[serde(rename = "nodeId")]
    pub node_id: NonEmptyString,
    pub ok: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub payload: Option<Value>,
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "payloadJSON")]
    pub payload_json: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<NodeInvokeResultErrorSchema>,
}

impl NodeInvokeResultParamsSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_non_empty_string("id", &self.id)?;
        validate_non_empty_string("nodeId", &self.node_id)?;
        if let Some(e) = &self.error {
            e.validate().map_err(|e_| format!("error: {}", e_))?;
        }
        Ok(())
    }
}

// ---------- NodeEventParamsSchema ----------

/// Generic node event envelope accepted by the gateway.
/// 对齐 TS:
///   `Type.Object({
///       event:      NonEmptyString,
///       payload:    Type.Optional(Type.Unknown()),
///       payloadJSON:Type.Optional(Type.String()),
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NodeEventParamsSchema {
    pub event: NonEmptyString,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub payload: Option<Value>,
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "payloadJSON")]
    pub payload_json: Option<String>,
}

impl NodeEventParamsSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_non_empty_string("event", &self.event)?;
        Ok(())
    }
}

// ---------- NodePendingDrainParamsSchema ----------

/// Request for a bounded batch of queued work assigned to the calling node.
/// 对齐 TS:
///   `Type.Object({
///       maxItems: Type.Optional(Type.Integer({ minimum: 1, maximum: 10 })),
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NodePendingDrainParamsSchema {
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "maxItems")]
    pub max_items: Option<i64>,
}

impl NodePendingDrainParamsSchema {
    pub fn validate(&self) -> Result<(), String> {
        if let Some(m) = self.max_items {
            if m < NODE_PENDING_DRAIN_MAX_ITEMS_MIN || m > NODE_PENDING_DRAIN_MAX_ITEMS_MAX {
                return Err(format!(
                    "maxItems: expected [{}, {}], got {}",
                    NODE_PENDING_DRAIN_MAX_ITEMS_MIN, NODE_PENDING_DRAIN_MAX_ITEMS_MAX, m
                ));
            }
        }
        Ok(())
    }
}

// ---------- NodePendingDrainItemSchema ----------

/// One queued node-work item returned by pending-work drain calls.
/// 对齐 TS:
///   `Type.Object({
///       id:           NonEmptyString,
///       type:         NodePendingWorkTypeSchema,
///       priority:     Type.String({ enum: ["default", "normal", "high"] }),
///       createdAtMs:  Type.Integer({ minimum: 0 }),
///       expiresAtMs:  Type.Optional(Type.Union([Type.Integer({ minimum: 0 }), Type.Null()])),
///       payload:      Type.Optional(Type.Record(Type.String(), Type.Unknown())),
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NodePendingDrainItemSchema {
    pub id: NonEmptyString,
    #[serde(rename = "type")]
    pub work_type: NodePendingWorkTypeSchema,
    pub priority: NodePendingDrainPrioritySchema,
    #[serde(rename = "createdAtMs")]
    pub created_at_ms: i64,
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "expiresAtMs")]
    pub expires_at_ms: Option<NodeExpiresAtMs>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub payload: Option<std::collections::BTreeMap<String, Value>>,
}

/// `expiresAtMs` accepts either a non-negative integer or `null`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum NodeExpiresAtMs {
    Millis(i64),
    Null(Value),
}

impl NodePendingDrainItemSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_non_empty_string("id", &self.id)?;
        validate_non_negative_integer("createdAtMs", self.created_at_ms)?;
        if let Some(NodeExpiresAtMs::Millis(m)) = &self.expires_at_ms {
            validate_non_negative_integer("expiresAtMs", *m)?;
        }
        Ok(())
    }
}

// ---------- NodePendingDrainResultSchema ----------

/// Drain response with a revision marker for node queue state.
/// 对齐 TS:
///   `Type.Object({
///       nodeId:   NonEmptyString,
///       revision: Type.Integer({ minimum: 0 }),
///       items:    Type.Array(NodePendingDrainItemSchema),
///       hasMore:  Type.Boolean(),
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NodePendingDrainResultSchema {
    #[serde(rename = "nodeId")]
    pub node_id: NonEmptyString,
    pub revision: i64,
    pub items: Vec<NodePendingDrainItemSchema>,
    #[serde(rename = "hasMore")]
    pub has_more: bool,
}

impl NodePendingDrainResultSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_non_empty_string("nodeId", &self.node_id)?;
        validate_non_negative_integer("revision", self.revision)?;
        for (i, item) in self.items.iter().enumerate() {
            item.validate().map_err(|e| format!("items[{}]: {}", i, e))?;
        }
        Ok(())
    }
}

// ---------- NodePendingEnqueueParamsSchema ----------

/// Enqueues gateway-initiated work for a paired node.
/// 对齐 TS:
///   `Type.Object({
///       nodeId:      NonEmptyString,
///       type:        NodePendingWorkTypeSchema,
///       priority:    Type.Optional(NodePendingWorkPrioritySchema),
///       expiresInMs: Type.Optional(Type.Integer({ minimum: 1_000, maximum: 86_400_000 })),
///       wake:        Type.Optional(Type.Boolean()),
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NodePendingEnqueueParamsSchema {
    #[serde(rename = "nodeId")]
    pub node_id: NonEmptyString,
    #[serde(rename = "type")]
    pub work_type: NodePendingWorkTypeSchema,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub priority: Option<NodePendingWorkPrioritySchema>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "expiresInMs"
    )]
    pub expires_in_ms: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub wake: Option<bool>,
}

impl NodePendingEnqueueParamsSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_non_empty_string("nodeId", &self.node_id)?;
        if let Some(t) = self.expires_in_ms {
            if !(1_000..=86_400_000).contains(&t) {
                return Err(format!(
                    "expiresInMs: expected [1000, 86400000], got {}",
                    t
                ));
            }
        }
        Ok(())
    }
}

// ---------- NodePendingEnqueueResultSchema ----------

/// Enqueue result echoes queue revision and whether wake delivery was attempted.
/// 对齐 TS:
///   `Type.Object({
///       nodeId:        NonEmptyString,
///       revision:      Type.Integer({ minimum: 0 }),
///       queued:        NodePendingDrainItemSchema,
///       wakeTriggered: Type.Boolean(),
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NodePendingEnqueueResultSchema {
    #[serde(rename = "nodeId")]
    pub node_id: NonEmptyString,
    pub revision: i64,
    pub queued: NodePendingDrainItemSchema,
    #[serde(rename = "wakeTriggered")]
    pub wake_triggered: bool,
}

impl NodePendingEnqueueResultSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_non_empty_string("nodeId", &self.node_id)?;
        validate_non_negative_integer("revision", self.revision)?;
        self.queued.validate().map_err(|e| format!("queued: {}", e))?;
        Ok(())
    }
}

// ---------- NodeInvokeRequestEventSchema ----------

/// Event payload used by the gateway to ask a node to run a command.
/// 对齐 TS:
///   `Type.Object({
///       id:             NonEmptyString,
///       nodeId:         NonEmptyString,
///       command:        NonEmptyString,
///       paramsJSON:     Type.Optional(Type.String()),
///       timeoutMs:      Type.Optional(Type.Integer({ minimum: 0 })),
///       idempotencyKey: Type.Optional(NonEmptyString),
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NodeInvokeRequestEventSchema {
    pub id: NonEmptyString,
    #[serde(rename = "nodeId")]
    pub node_id: NonEmptyString,
    pub command: NonEmptyString,
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "paramsJSON")]
    pub params_json: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "timeoutMs")]
    pub timeout_ms: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "idempotencyKey")]
    pub idempotency_key: Option<NonEmptyString>,
}

impl NodeInvokeRequestEventSchema {
    pub fn validate(&self) -> Result<(), String> {
        validate_non_empty_string("id", &self.id)?;
        validate_non_empty_string("nodeId", &self.node_id)?;
        validate_non_empty_string("command", &self.command)?;
        if let Some(t) = self.timeout_ms {
            validate_non_negative_integer("timeoutMs", t)?;
        }
        if let Some(k) = &self.idempotency_key {
            validate_non_empty_string("idempotencyKey", k)?;
        }
        Ok(())
    }
}

// Wire type aliases (对标 TS `type X = Static<typeof YSchema>`)
pub type NodePairListParams = NodePairListParamsSchema;
pub type NodePairApproveParams = NodePairApproveParamsSchema;
pub type NodePairRejectParams = NodePairRejectParamsSchema;
pub type NodePairRemoveParams = NodePairRemoveParamsSchema;
pub type NodeRenameParams = NodeRenameParamsSchema;
pub type NodeListParams = NodeListParamsSchema;
pub type NodePendingAckParams = NodePendingAckParamsSchema;
pub type NodeDescribeParams = NodeDescribeParamsSchema;
pub type NodeInvokeParams = NodeInvokeParamsSchema;
pub type NodeInvokeResultParams = NodeInvokeResultParamsSchema;
pub type NodeEventParams = NodeEventParamsSchema;
pub type NodeEventResult = NodeEventResultSchema;
pub type NodePresenceAlivePayload = NodePresenceAlivePayloadSchema;
pub type NodePresenceAliveReason = NodePresenceAliveReasonSchema;
pub type NodePresenceActivityPayload = NodePresenceActivityPayloadSchema;
pub type NodePendingDrainParams = NodePendingDrainParamsSchema;
pub type NodePendingDrainResult = NodePendingDrainResultSchema;
pub type NodePendingEnqueueParams = NodePendingEnqueueParamsSchema;
pub type NodePendingEnqueueResult = NodePendingEnqueueResultSchema;
pub type NodePluginToolDescriptor = NodePluginToolDescriptorSchema;
pub type NodePluginToolsUpdateParams = NodePluginToolsUpdateParamsSchema;
pub type NodeSkillDescriptor = NodeSkillDescriptorSchema;
pub type NodeSkillsUpdateParams = NodeSkillsUpdateParamsSchema;