// LLM core type definitions.
// 翻译自 packages/llm-core/src/types.ts

use std::collections::BTreeMap;
use std::future::Future;
use std::pin::Pin;

use serde::{Deserialize, Serialize};

use crate::utils::diagnostics::AssistantMessageDiagnostic;

pub use crate::utils::diagnostics::{DiagnosticErrorCode, DiagnosticErrorInfo};

/// Provider API families with first-class request/stream adapters in cradle-ring.
pub const KNOWN_APIS: &[&str] = &[
    "openai-completions",
    "mistral-conversations",
    "openai-responses",
    "azure-openai-responses",
    "openai-chatgpt-responses",
    "anthropic-messages",
    "bedrock-converse-stream",
    "google-generative-ai",
    "google-vertex",
];

/// Provider API id; custom providers can use ids outside the built-in set.
pub type Api = String;

/// Image-generation API families with first-class adapters in cradle-ring.
pub const KNOWN_IMAGES_APIS: &[&str] = &["openrouter-images"];

/// Image API id; custom image providers can use ids outside the built-in set.
pub type ImagesApi = String;

/// Provider id used for routing, diagnostics, and config lookups.
pub type Provider = String;

/// Image provider ids with first-class adapters in cradle-ring.
pub const KNOWN_IMAGES_PROVIDERS: &[&str] = &["openrouter"];

/// Image provider id used for routing, diagnostics, and config lookups.
pub type ImagesProvider = String;

/// Normalized reasoning-effort levels shared across provider-specific knobs.
pub const THINKING_LEVELS: &[&str] = &["minimal", "low", "medium", "high", "xhigh", "max"];
pub type ThinkingLevel = String;

/// Model thinking setting including explicit disabled state.
pub type ModelThinkingLevel = String;

/// Provider-specific values for normalized thinking levels.
pub type ThinkingLevelMap = BTreeMap<String, Option<String>>;

/// Token budgets for each thinking level (token-based providers only).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ThinkingBudgets {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub minimal: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub low: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub medium: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub high: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max: Option<f64>,
}

/// Prompt-cache retention preference shared by providers that expose cache controls.
pub type CacheRetention = String;

/// Streaming transport preference for providers that support multiple transports.
pub type Transport = String;

/// Minimal HTTP response metadata surfaced through provider hooks.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderResponse {
    pub status: i64,
    pub headers: BTreeMap<String, String>,
}

/// Payload callback used to inspect or replace provider payloads before sending.
pub type PayloadHook =
    fn(payload: serde_json::Value, model: Model) -> Pin<Box<dyn Future<Output = Option<serde_json::Value>> + Send>>;

/// Request options shared by text streaming providers.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StreamOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transport: Option<Transport>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_retention: Option<CacheRetention>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt_cache_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub headers: Option<BTreeMap<String, String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout_ms: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_retries: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_retry_delay_ms: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<BTreeMap<String, serde_json::Value>>,
}

/// Provider stream options = StreamOptions + extra provider-specific keys.
pub type ProviderStreamOptions = BTreeMap<String, serde_json::Value>;

/// Request options shared by image-generation providers.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImagesOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub headers: Option<BTreeMap<String, String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout_ms: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_retries: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_retry_delay_ms: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<BTreeMap<String, serde_json::Value>>,
}

/// Provider images options = ImagesOptions + extra provider-specific keys.
pub type ProviderImagesOptions = BTreeMap<String, serde_json::Value>;

/// Unified text options used by simple completion helpers.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SimpleStreamOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transport: Option<Transport>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_retention: Option<CacheRetention>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt_cache_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub headers: Option<BTreeMap<String, String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout_ms: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_retries: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_retry_delay_ms: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<BTreeMap<String, serde_json::Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning: Option<ModelThinkingLevel>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thinking_budgets: Option<ThinkingBudgets>,
}

/// Text signature v1 (used in assistant text content blocks).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TextSignatureV1 {
    pub v: i64,
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub phase: Option<String>,
}

/// Plain assistant/user text content block.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TextContent {
    #[serde(rename = "type")]
    pub type_: String,
    pub text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text_signature: Option<String>,
}

/// Provider reasoning/thinking content block.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ThinkingContent {
    #[serde(rename = "type")]
    pub type_: String,
    pub thinking: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thinking_signature: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub redacted: Option<bool>,
}

/// Base64 image content block with MIME type metadata.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImageContent {
    #[serde(rename = "type")]
    pub type_: String,
    pub data: String,
    pub mime_type: String,
}

/// Normalized token and cost accounting for a provider response.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UsageCost {
    pub input: f64,
    pub output: f64,
    pub cache_read: f64,
    pub cache_write: f64,
    pub total: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_origin: Option<String>,
}

/// Context usage state used by Usage.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "state", rename_all = "camelCase")]
pub enum ContextUsage {
    Available {
        prompt_tokens: i64,
        total_tokens: i64,
    },
    Unavailable,
}

/// Normalized token and cost accounting for a provider response.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Usage {
    pub input: i64,
    pub output: i64,
    pub cache_read: i64,
    pub cache_write: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context_usage: Option<ContextUsage>,
    pub total_tokens: i64,
    pub cost: UsageCost,
}

/// Normalized assistant stop reasons across text providers.
pub type StopReason = String;

/// Normalized tool-call execution mode.
pub type ToolExecutionMode = String;

/// Normalized assistant tool call emitted by providers or repaired from text.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolCall {
    #[serde(rename = "type")]
    pub type_: String,
    pub id: String,
    pub name: String,
    pub arguments: BTreeMap<String, serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thought_signature: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub execution_mode: Option<ToolExecutionMode>,
}

/// User turn in a text-model conversation.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserMessage {
    pub role: String,
    pub content: UserMessageContent,
    pub timestamp: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub runtime_context_carrier: Option<bool>,
}

/// User message content: either a string or a list of TextContent/ImageContent.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum UserMessageContent {
    Text(String),
    Parts(Vec<UserMessagePart>),
}

impl Default for UserMessageContent {
    fn default() -> Self {
        UserMessageContent::Text(String::new())
    }
}

/// Individual content parts for a user message.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum UserMessagePart {
    Text(TextContent),
    Image(ImageContent),
}

/// Assistant turn, including provider identity and final stop state.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AssistantMessage {
    pub role: String,
    pub content: Vec<AssistantContentPart>,
    pub api: Api,
    pub provider: Provider,
    pub model: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub diagnostics: Option<Vec<AssistantMessageDiagnostic>>,
    pub usage: Usage,
    pub stop_reason: StopReason,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_body: Option<String>,
    pub timestamp: i64,
}

/// Assistant content parts (text, thinking, or tool call).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum AssistantContentPart {
    Text(TextContent),
    Thinking(ThinkingContent),
    ToolCall(ToolCall),
}

/// Tool result turn that answers a prior assistant tool call.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolResultMessage {
    pub role: String,
    pub tool_call_id: String,
    pub tool_name: String,
    pub content: Vec<ToolResultPart>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
    pub is_error: bool,
    pub timestamp: i64,
}

/// Tool result content parts.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum ToolResultPart {
    Text(TextContent),
    Image(ImageContent),
}

/// Any text-model conversation message supported by LLM core.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "role", rename_all = "camelCase")]
pub enum Message {
    User(UserMessage),
    Assistant(AssistantMessage),
    ToolResult(ToolResultMessage),
}

/// Image request input content accepted by image providers.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum ImagesInputContent {
    Text(TextContent),
    Image(ImageContent),
}

/// Image response output content returned by image providers.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum ImagesOutputContent {
    Text(TextContent),
    Image(ImageContent),
}

/// Image-generation request context.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ImagesContext {
    pub input: Vec<ImagesInputContent>,
}

/// Normalized image-generation stop reasons.
pub type ImagesStopReason = String;

/// Final image-generation response shape.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AssistantImages {
    pub api: ImagesApi,
    pub provider: ImagesProvider,
    pub model: String,
    pub output: Vec<ImagesOutputContent>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage: Option<Usage>,
    pub stop_reason: ImagesStopReason,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_message: Option<String>,
    pub timestamp: i64,
}

/// Provider tool declaration with a JSON-schema parameter object.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Tool {
    pub name: String,
    pub description: String,
    /// JSON schema describing the tool's parameters.
    pub parameters: serde_json::Value,
}

/// Text-model request context shared by provider adapters.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Context {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system_prompt: Option<String>,
    pub messages: Vec<Message>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<Tool>>,
}

/// Event protocol for AssistantMessageEventStream.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum AssistantMessageEvent {
    Start {
        partial: AssistantMessage,
    },
    TextStart {
        content_index: i64,
        partial: AssistantMessage,
    },
    TextDelta {
        content_index: i64,
        delta: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        partial: Option<AssistantMessage>,
    },
    TextEnd {
        content_index: i64,
        content: String,
        partial: AssistantMessage,
    },
    ThinkingStart {
        content_index: i64,
        partial: AssistantMessage,
    },
    ThinkingDelta {
        content_index: i64,
        delta: String,
        partial: AssistantMessage,
    },
    ThinkingEnd {
        content_index: i64,
        content: String,
        partial: AssistantMessage,
    },
    ToolcallStart {
        content_index: i64,
        partial: AssistantMessage,
    },
    ToolcallDelta {
        content_index: i64,
        delta: String,
        partial: AssistantMessage,
    },
    ToolcallEnd {
        content_index: i64,
        tool_call: ToolCall,
        partial: AssistantMessage,
    },
    Done {
        reason: String,
        message: AssistantMessage,
    },
    Error {
        reason: String,
        error: AssistantMessage,
    },
}

/// Stream contract for AssistantMessageEventStream.
pub trait AssistantMessageEventStreamContract: Send + Sync {
    /// Queue one stream event for consumers.
    fn push(&self, event: AssistantMessageEvent);
    /// Complete the stream and optionally resolve the final message.
    fn end(&self, result: Option<AssistantMessage>);
    /// Final assistant message produced by the stream.
    fn result(&self) -> Pin<Box<dyn Future<Output = AssistantMessage> + Send>>;
}

/// Compatibility settings for OpenAI-compatible completions APIs.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OpenAICompletionsCompat {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub supports_store: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub supports_developer_role: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub supports_reasoning_effort: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub supports_usage_in_streaming: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens_field: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub requires_tool_result_name: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub requires_assistant_after_tool_result: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub requires_thinking_as_text: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub requires_reasoning_content_on_assistant_messages: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thinking_format: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub open_router_routing: Option<OpenRouterRouting>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vercel_gateway_routing: Option<VercelGatewayRouting>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub zai_tool_stream: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub supports_strict_mode: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_control_format: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub send_session_affinity_headers: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub supports_prompt_cache_key: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub supports_long_cache_retention: Option<bool>,
}

/// Compatibility settings for OpenAI Responses APIs.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OpenAIResponsesCompat {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub supports_temperature: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub send_session_id_header: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub supports_long_cache_retention: Option<bool>,
}

/// Compatibility settings for Anthropic Messages-compatible APIs.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AnthropicMessagesCompat {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub supports_eager_tool_input_streaming: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub supports_long_cache_retention: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub send_session_affinity_headers: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub supports_cache_control_on_tools: Option<bool>,
}

/// OpenRouter provider routing preferences.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct OpenRouterRouting {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allow_fallbacks: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub require_parameters: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data_collection: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub zdr: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enforce_distillable_text: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub only: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ignore: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quantizations: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sort: Option<OpenRouterSortChoice>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_price: Option<OpenRouterMaxPrice>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preferred_min_throughput: Option<OpenRouterMetricPreference>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preferred_max_latency: Option<OpenRouterMetricPreference>,
}

/// OpenRouter sort preference (string or structured form).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum OpenRouterSortChoice {
    Simple(String),
    Structured {
        #[serde(skip_serializing_if = "Option::is_none")]
        by: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        partition: Option<serde_json::Value>,
    },
}

/// OpenRouter max_price substructure.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct OpenRouterMaxPrice {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completion: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub audio: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request: Option<serde_json::Value>,
}

/// OpenRouter throughput/latency percentile cutoffs (number or structured).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum OpenRouterMetricPreference {
    Number(f64),
    Cutoffs(OpenRouterPercentileCutoffs),
}

/// OpenRouter percentile cutoffs.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct OpenRouterPercentileCutoffs {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub p50: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub p75: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub p90: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub p99: Option<f64>,
}

/// Vercel AI Gateway routing preferences.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct VercelGatewayRouting {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub only: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order: Option<Vec<String>>,
}

/// Model interface for the unified model system.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Model {
    pub id: String,
    pub name: String,
    pub api: Api,
    pub provider: Provider,
    pub base_url: String,
    pub reasoning: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thinking_level_map: Option<ThinkingLevelMap>,
    pub input: Vec<String>,
    pub cost: ModelCost,
    pub context_window: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context_tokens: Option<f64>,
    pub max_tokens: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<BTreeMap<String, serde_json::Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub headers: Option<BTreeMap<String, String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auth_header: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub compat: Option<ModelCompat>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub media_input: Option<ModelMediaInput>,
}

/// Per-million-token cost on the unified Model record.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelCost {
    pub input: f64,
    pub output: f64,
    pub cache_read: f64,
    pub cache_write: f64,
}

/// Compat field that is variant based on `api`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ModelCompat {
    OpenAICompletions(OpenAICompletionsCompat),
    OpenAIResponses(OpenAIResponsesCompat),
    AnthropicMessages(AnthropicMessagesCompat),
}

/// Media-input limits surfaced through the unified Model record.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ModelMediaInput {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image: Option<ModelImageInput>,
}

/// Per-image input limits surfaced through the unified Model record.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelImageInput {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_bytes: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_pixels: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_side_px: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preferred_side_px: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token_mode: Option<String>,
}

/// Image-generation model extends the text Model with image-output support.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImagesModel {
    pub id: String,
    pub name: String,
    pub api: ImagesApi,
    pub provider: ImagesProvider,
    pub base_url: String,
    pub input: Vec<String>,
    pub output: Vec<String>,
    pub cost: ModelCost,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thinking_level_map: Option<ThinkingLevelMap>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<BTreeMap<String, serde_json::Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub headers: Option<BTreeMap<String, String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auth_header: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub media_input: Option<ModelMediaInput>,
}

/// Type alias for the standard stream function signature.
pub type StreamFn =
    fn(
        model: Model,
        context: Context,
        options: Option<SimpleStreamOptions>,
    ) -> Pin<Box<dyn Future<Output = AssistantMessage> + Send>>;

/// Type alias for the simple-completion signature.
pub type CompleteSimpleFn =
    fn(
        model: Model,
        context: PickContext,
        options: Option<SimpleStreamOptions>,
    ) -> Pin<Box<dyn Future<Output = AssistantMessage> + Send>>;

/// Minimal context used by `CompleteSimpleFn`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PickContext {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system_prompt: Option<String>,
    pub messages: Vec<Message>,
}

/// Type alias for the validation hook signature.
pub type ValidateToolArgumentsFn = fn(tool: Tool, tool_call: ToolCall) -> serde_json::Value;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn role_constants_match_documented_values() {
        assert_eq!(UserMessage::default().role, "");
        let assistant = AssistantMessage {
            role: "assistant".to_string(),
            content: Vec::new(),
            api: "openai-completions".to_string(),
            provider: "openai".to_string(),
            model: "gpt-4".to_string(),
            response_model: None,
            response_id: None,
            diagnostics: None,
            usage: Usage::default(),
            stop_reason: "stop".to_string(),
            error_message: None,
            error_code: None,
            error_type: None,
            error_body: None,
            timestamp: 0,
        };
        assert_eq!(assistant.role, "assistant");
    }
}