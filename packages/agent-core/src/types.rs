// Agent Core type module defines shared TypeScript contracts.
// 翻译自 packages/agent-core/src/types.ts

use std::future::Future;
use std::pin::Pin;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use llm_core::types::{
    AssistantMessage, AssistantMessageEvent, Context, ImageContent, Model, SimpleStreamOptions,
    TextContent, Tool, ToolResultMessage,
};

/// Stream function used by the agent loop.
pub type StreamFn =
    fn(
        model: Model,
        context: Context,
        options: Option<SimpleStreamOptions>,
    ) -> Pin<Box<dyn Future<Output = AssistantMessage> + Send>>;

/// Configuration for how tool calls from a single assistant message are executed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToolExecutionMode {
    Sequential,
    Parallel,
}

/// Controls how many queued user messages are injected when the agent loop reaches a queue drain point.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QueueMode {
    All,
    OneAtATime,
}

/// A single tool call content block emitted by an assistant message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentToolCall {
    pub id: String,
    pub name: String,
    pub arguments: Value,
}

/// Result returned from `beforeToolCall`.
#[derive(Debug, Clone, Default)]
pub struct BeforeToolCallResult {
    pub block: bool,
    pub reason: Option<String>,
}

#[derive(Debug, Clone)]
pub struct DeferredToolCallContext {
    /// The assistant message that requested the deferred tool call.
    pub assistant_message: AssistantMessage,
    /// The raw tool call block whose authorized tool definition is deferred.
    pub tool_call: AgentToolCall,
    /// Current agent context before the deferred tool is hydrated.
    pub context: AgentContext,
}

/// Partial override returned from `afterToolCall`.
#[derive(Debug, Clone, Default)]
pub struct AfterToolCallResult {
    pub content: Option<Vec<TextOrImageContent>>,
    pub details: Option<Value>,
    pub is_error: Option<bool>,
    /// Hint that the agent should stop after the current tool batch.
    pub terminate: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum TextOrImageContent {
    Text(TextContent),
    Image(ImageContent),
}

impl From<TextContent> for TextOrImageContent {
    fn from(t: TextContent) -> Self {
        TextOrImageContent::Text(t)
    }
}

impl From<ImageContent> for TextOrImageContent {
    fn from(i: ImageContent) -> Self {
        TextOrImageContent::Image(i)
    }
}

/// Context passed to `beforeToolCall`.
#[derive(Debug, Clone)]
pub struct BeforeToolCallContext {
    /// The assistant message that requested the tool call.
    pub assistant_message: AssistantMessage,
    /// The raw tool call block from `assistantMessage.content`.
    pub tool_call: AgentToolCall,
    /// Validated tool arguments for the target tool schema.
    pub args: Value,
    /// Current agent context at the time the tool call is prepared.
    pub context: AgentContext,
}

/// Context passed to `afterToolCall`.
#[derive(Debug, Clone)]
pub struct AfterToolCallContext {
    /// The assistant message that requested the tool call.
    pub assistant_message: AssistantMessage,
    /// The raw tool call block from `assistantMessage.content`.
    pub tool_call: AgentToolCall,
    /// Validated tool arguments for the target tool schema.
    pub args: Value,
    /// The executed tool result before unknown `afterToolCall` overrides are applied.
    pub result: AgentToolResult<Value>,
    /// Whether the executed tool result is currently treated as an error.
    pub is_error: bool,
    /// Current agent context at the time the tool call is finalized.
    pub context: AgentContext,
}

/// Context passed to `shouldStopAfterTurn`.
#[derive(Debug, Clone)]
pub struct ShouldStopAfterTurnContext {
    /// The assistant message that completed the turn.
    pub message: AgentMessage,
    /// Tool result messages passed to the preceding `turn_end` event.
    pub tool_results: Vec<ToolResultMessage>,
    /// Current agent context after the turn's assistant message and tool results have been appended.
    pub context: AgentContext,
    /// Messages that this loop invocation will return if it exits at this point.
    pub new_messages: Vec<AgentMessage>,
}

/// Replacement runtime state used by the agent loop before starting another provider request.
#[derive(Debug, Clone, Default)]
pub struct AgentLoopTurnUpdate {
    /// Context for the next provider request.
    pub context: Option<AgentContext>,
    /// Model for the next provider request.
    pub model: Option<Model>,
    /// Thinking level for the next provider request.
    pub thinking_level: Option<ThinkingLevel>,
}

pub type PrepareNextTurnContext = ShouldStopAfterTurnContext;

pub trait AgentLoopConfig: Send + Sync {
    fn model(&self) -> &Model;
    fn thinking_level(&self) -> Option<&ThinkingLevel>;
    fn reasoning(&self) -> Option<&str>;
    fn session_id(&self) -> Option<&str>;
    fn thinking_budgets(&self) -> Option<&llm_core::types::ThinkingBudgets>;
    fn transport(&self) -> &str;
    fn max_retry_delay_ms(&self) -> Option<i64>;
    fn api_key(&self) -> Option<&str>;
    fn signal(&self) -> Option<&AbortSignalShim>;

    fn convert_to_llm(
        &self,
        messages: Vec<AgentMessage>,
    ) -> Pin<Box<dyn Future<Output = Vec<llm_core::types::Message>> + Send>>;

    fn transform_context(
        &self,
        messages: Vec<AgentMessage>,
        signal: Option<AbortSignalShim>,
    ) -> Pin<Box<dyn Future<Output = Vec<AgentMessage>> + Send>>;

    fn get_api_key(
        &self,
        provider: String,
    ) -> Pin<Box<dyn Future<Output = Option<String>> + Send>>;

    fn should_stop_after_turn(
        &self,
        context: ShouldStopAfterTurnContext,
    ) -> Pin<Box<dyn Future<Output = bool> + Send>>;

    fn prepare_next_turn(
        &self,
        context: PrepareNextTurnContext,
    ) -> Pin<Box<dyn Future<Output = Option<AgentLoopTurnUpdate>> + Send>>;

    fn get_steering_messages(
        &self,
    ) -> Pin<Box<dyn Future<Output = Vec<AgentMessage>> + Send>>;

    fn get_follow_up_messages(
        &self,
    ) -> Pin<Box<dyn Future<Output = Vec<AgentMessage>> + Send>>;

    fn tool_execution(&self) -> ToolExecutionMode;

    fn before_tool_call(
        &self,
        context: BeforeToolCallContext,
        signal: Option<AbortSignalShim>,
    ) -> Pin<Box<dyn Future<Output = Option<BeforeToolCallResult>> + Send>>;

    fn resolve_deferred_tool(
        &self,
        context: DeferredToolCallContext,
        signal: Option<AbortSignalShim>,
    ) -> Pin<Box<dyn Future<Output = Option<AgentTool>> + Send>>;

    fn after_tool_call(
        &self,
        context: AfterToolCallContext,
        signal: Option<AbortSignalShim>,
    ) -> Pin<Box<dyn Future<Output = Option<AfterToolCallResult>> + Send>>;
}

/// Thinking/reasoning level for models that support it.
pub type ThinkingLevel = String;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BashExecutionMessage {
    pub role: String, // "bashExecution"
    pub command: String,
    pub output: String,
    pub exit_code: Option<i64>,
    pub cancelled: bool,
    pub truncated: bool,
    pub full_output_path: Option<String>,
    pub timestamp: i64,
    pub exclude_from_context: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomMessage {
    pub role: String, // "custom"
    pub custom_type: String,
    pub content: CustomMessageContent,
    pub display: bool,
    pub details: Option<Value>,
    pub timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum CustomMessageContent {
    Text(String),
    Parts(Vec<TextOrImageContent>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BranchSummaryMessage {
    pub role: String, // "branchSummary"
    pub summary: String,
    pub from_id: String,
    pub timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompactionSummaryMessage {
    pub role: String, // "compactionSummary"
    pub summary: String,
    pub tokens_before: i64,
    pub timestamp: CompactionTimestamp,
    pub tokens_after: Option<i64>,
    pub first_kept_entry_id: Option<String>,
    pub details: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum CompactionTimestamp {
    Number(i64),
    String(String),
}

/// AgentMessage: Union of LLM messages + custom messages.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum AgentMessage {
    Llm(llm_core::types::Message),
    BashExecution(BashExecutionMessage),
    Custom(CustomMessage),
    BranchSummary(BranchSummaryMessage),
    CompactionSummary(CompactionSummaryMessage),
}

/// Public agent state.
#[derive(Debug, Clone)]
pub struct AgentState {
    pub system_prompt: String,
    pub model: Model,
    pub thinking_level: ThinkingLevel,
    pub tools: Vec<AgentTool>,
    pub messages: Vec<AgentMessage>,
    pub is_streaming: bool,
    pub streaming_message: Option<AgentMessage>,
    pub pending_tool_calls: std::collections::HashSet<String>,
    pub error_message: Option<String>,
}

/// Channel-safe progress text emitted by a running tool.
#[derive(Debug, Clone)]
pub struct AgentToolProgress {
    pub text: String,
    pub visibility: String, // "channel"
    pub privacy: String,    // "public"
    pub id: Option<String>,
}

/// Final or partial result produced by a tool.
#[derive(Debug, Clone)]
pub struct AgentToolResult<T> {
    pub content: Vec<TextOrImageContent>,
    pub details: T,
    pub progress: Option<AgentToolProgress>,
    /// Hint that the agent should stop after the current tool batch.
    pub terminate: Option<bool>,
}

/// Callback used by tools to stream partial execution updates.
pub type AgentToolUpdateCallback<T = Value> = fn(partial_result: AgentToolResult<T>);

/// Tool definition used by the agent runtime.
pub struct AgentTool {
    /// Inherited from Tool: name, description, parameters.
    pub base: Tool,
    /// Human-readable label for UI display.
    pub label: String,
    /// Preserve lifecycle telemetry without rendering transient channel progress.
    pub hide_from_channel_progress: Option<bool>,
    /// Optional compatibility shim for raw tool-call arguments before schema validation.
    pub prepare_arguments: Option<fn(Value) -> Value>,
    /// Execute the tool call. Throw on failure instead of encoding errors in `content`.
    pub execute: fn(
        tool_call_id: String,
        params: Value,
        signal: Option<AbortSignalShim>,
        on_update: Option<AgentToolUpdateCallback>,
    ) -> Pin<Box<dyn Future<Output = AgentToolResult<Value>> + Send>>,
    /// Per-tool execution mode override.
    pub execution_mode: Option<ToolExecutionMode>,
}

impl std::fmt::Debug for AgentTool {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AgentTool")
            .field("base", &self.base)
            .field("label", &self.label)
            .field("hide_from_channel_progress", &self.hide_from_channel_progress)
            .field("execution_mode", &self.execution_mode)
            .finish()
    }
}

impl Clone for AgentTool {
    fn clone(&self) -> Self {
        Self {
            base: self.base.clone(),
            label: self.label.clone(),
            hide_from_channel_progress: self.hide_from_channel_progress,
            prepare_arguments: self.prepare_arguments,
            execute: self.execute,
            execution_mode: self.execution_mode,
        }
    }
}

/// AbortSignal shim mirroring the Node.js Web AbortSignal API.
#[derive(Debug, Clone, Default)]
pub struct AbortSignalShim {
    pub aborted: bool,
    pub reason: Option<Value>,
}

/// Context snapshot passed into the low-level agent loop.
#[derive(Debug, Clone)]
pub struct AgentContext {
    /// System prompt included with the request.
    pub system_prompt: String,
    /// Transcript visible to the model.
    pub messages: Vec<AgentMessage>,
    /// Tools available for this run.
    pub tools: Option<Vec<AgentTool>>,
}

/// Events emitted by the Agent for UI updates.
#[derive(Debug, Clone)]
pub enum AgentEvent {
    AgentStart,
    AgentEnd { messages: Vec<AgentMessage> },
    TurnStart,
    TurnEnd {
        message: AgentMessage,
        tool_results: Vec<ToolResultMessage>,
    },
    MessageStart { message: AgentMessage },
    MessageUpdate {
        message: AgentMessage,
        assistant_message_event: AssistantMessageEvent,
    },
    MessageEnd { message: AgentMessage },
    ToolExecutionStart {
        tool_call_id: String,
        tool_name: String,
        args: Value,
        hide_from_channel_progress: Option<bool>,
    },
    ToolExecutionUpdate {
        tool_call_id: String,
        tool_name: String,
        args: Value,
        partial_result: Value,
        hide_from_channel_progress: Option<bool>,
    },
    ToolExecutionEnd {
        tool_call_id: String,
        tool_name: String,
        result: Value,
        is_error: bool,
        execution_started: Option<bool>,
        error_kind: Option<String>,
        hide_from_channel_progress: Option<bool>,
    },
}

#[allow(dead_code)]
fn _force_use() {
    let _: Option<Tool> = None;
    let _: Option<AssistantMessage> = None;
}