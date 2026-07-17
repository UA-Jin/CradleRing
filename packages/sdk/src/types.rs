// Public SDK data contracts for Gateway transport, runs, sessions, tools,
// artifacts, tasks, environments, and normalized event streams.
// 翻译自 packages/sdk/src/types.ts

use serde::{Deserialize, Serialize};
use serde_json::Value;

pub type JsonObject = serde_json::Map<String, Value>;

/// Per-request options accepted by SDK transports.
#[derive(Default, Clone, Debug)]
pub struct GatewayRequestOptions {
    pub expect_final: Option<bool>,
    pub timeout_ms: Option<Option<u64>>,
}

impl GatewayRequestOptions {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn expect_final(mut self, v: bool) -> Self {
        self.expect_final = Some(v);
        self
    }
    pub fn timeout_ms(mut self, v: Option<u64>) -> Self {
        self.timeout_ms = Some(v);
        self
    }
}

/// Raw event payload emitted by the Gateway transport.
#[derive(Default, Clone, Debug, Deserialize, Serialize)]
pub struct GatewayEvent {
    pub event: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub payload: Option<Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub seq: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub state_version: Option<Value>,
}

/// Minimal transport interface consumed by the CradleRing SDK client.
pub trait OpenClawTransport: Send + Sync {
    fn request(
        &self,
        method: &str,
        params: Option<Value>,
        options: Option<GatewayRequestOptions>,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Value, SDKError>> + Send>>;
    fn events(
        &self,
        filter: Option<Box<dyn Fn(&GatewayEvent) -> bool + Send + Sync>>,
    ) -> std::pin::Pin<Box<dyn futures_util::Stream<Item = GatewayEvent> + Send>>;
    fn close(&self) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), SDKError>> + Send>>;
}

/// Transport variant that requires an explicit connection step.
pub trait ConnectableOpenClawTransport: OpenClawTransport {
    fn connect(&self) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), SDKError>> + Send>>;
}

/// Desired runtime/harness selection for future per-run execution routing.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum RuntimeSelection {
    Auto,
    Embedded { id: String },
    Cli { id: String },
    Acp { harness: String },
    Managed { provider: String },
}

/// Desired execution environment selection for future per-run routing.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum EnvironmentSelection {
    Local { cwd: Option<String> },
    Gateway { url: Option<String>, cwd: Option<String> },
    Node { node_id: String, cwd: Option<String> },
    Managed { provider: String, repo: Option<String>, ref_: Option<String> },
    Ephemeral { provider: String, repo: Option<String>, ref_: Option<String> },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
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

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum WorkerTunnelStatus {
    Stopped,
    Connecting,
    Connected,
    Reconnecting,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WorkerEnvironmentMetadata {
    pub provider_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lease_id: Option<String>,
    pub state: WorkerEnvironmentState,
    pub age_ms: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub idle_ms: Option<u64>,
    pub attached_session_ids: Vec<String>,
    pub tunnel_status: WorkerTunnelStatus,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EnvironmentSummary {
    pub id: String,
    #[serde(rename = "type")]
    pub r#type: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    pub status: EnvironmentStatus,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub capabilities: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub worker: Option<WorkerEnvironmentMetadata>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum EnvironmentStatus {
    #[default]
    Available,
    Unavailable,
    Starting,
    Stopping,
    Error,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EnvironmentCreateParams {
    pub profile_id: String,
    pub idempotency_key: String,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct EnvironmentsListResult {
    #[serde(default)]
    pub environments: Vec<EnvironmentSummary>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct WorkspaceSelection {
    pub cwd: Option<String>,
    pub repo: Option<String>,
    pub ref_: Option<String>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum ApprovalMode {
    #[default]
    Ask,
    Never,
    Auto,
    Trusted,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum ApprovalDecision {
    #[default]
    AllowOnce,
    AllowAlways,
    Deny,
}

#[derive(Clone, Debug)]
pub struct ApprovalDecisionParams {
    pub decision: ApprovalDecision,
}

/// Terminal and non-terminal status values returned by Run.wait.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RunStatus {
    Accepted,
    Completed,
    Failed,
    Cancelled,
    TimedOut,
}

pub type RunTimestamp = String;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SDKMessage {
    pub role: SdkMessageRole,
    pub content: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum SdkMessageRole {
    #[default]
    System,
    User,
    Assistant,
    Tool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ArtifactSummary {
    pub id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub run_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub task_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub session_key: Option<String>,
    #[serde(rename = "type")]
    pub r#type: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mime_type: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub size_bytes: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message_seq: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub download: Option<ArtifactDownload>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ArtifactDownload {
    pub mode: String,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ArtifactQuery {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub session_key: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub run_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub task_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub agent_id: Option<String>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ArtifactsListResult {
    #[serde(default)]
    pub artifacts: Vec<ArtifactSummary>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ArtifactsGetResult {
    pub artifact: ArtifactSummary,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ArtifactsDownloadResult {
    pub artifact: ArtifactSummary,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub encoding: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub data: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskStatus {
    #[default]
    Queued,
    Running,
    Completed,
    Failed,
    Cancelled,
    TimedOut,
}

/// Gateway task summary returned by task list/get calls.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct TaskSummary {
    pub id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub task_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub kind: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub runtime: Option<String>,
    pub status: TaskStatus,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub agent_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub session_key: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub child_session_key: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub owner_key: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub run_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub flow_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent_task_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created_at: Option<RunTimestamp>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<RunTimestamp>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub started_at: Option<RunTimestamp>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ended_at: Option<RunTimestamp>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub progress_summary: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub terminal_summary: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct TasksListParams {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status: Option<Vec<TaskStatus>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub agent_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub session_key: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub limit: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct TasksListResult {
    #[serde(default)]
    pub tasks: Vec<TaskSummary>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub next_cursor: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TasksGetResult {
    pub task: TaskSummary,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TasksCancelResult {
    pub found: bool,
    pub cancelled: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub task: Option<TaskSummary>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SDKError {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
    pub message: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub details: Option<Value>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ToolsEffectiveParams {
    pub session_key: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub agent_id: Option<String>,
}

#[derive(Clone, Debug, Default)]
pub struct ToolInvokeParams {
    pub args: Option<JsonObject>,
    pub session_key: Option<String>,
    pub agent_id: Option<String>,
    pub confirm: Option<bool>,
    pub idempotency_key: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ToolInvokeResult {
    pub ok: bool,
    pub tool_name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output: Option<Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub requires_approval: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub approval_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<SDKError>,
}

/// Normalized result returned by Run.wait.
#[derive(Clone, Debug)]
pub struct RunResult {
    pub run_id: String,
    pub status: RunStatus,
    pub session_id: Option<String>,
    pub session_key: Option<String>,
    pub task_id: Option<String>,
    pub started_at: Option<RunTimestamp>,
    pub ended_at: Option<RunTimestamp>,
    pub output: Option<RunResultOutput>,
    pub usage: Option<RunResultUsage>,
    pub artifacts: Option<Vec<ArtifactSummary>>,
    pub error: Option<SDKError>,
    pub raw: Option<Value>,
}

#[derive(Clone, Debug, Default)]
pub struct RunResultOutput {
    pub text: Option<String>,
    pub messages: Option<Vec<SDKMessage>>,
}

#[derive(Clone, Debug, Default)]
pub struct RunResultUsage {
    pub input_tokens: Option<u64>,
    pub output_tokens: Option<u64>,
    pub total_tokens: Option<u64>,
    pub cost_usd: Option<f64>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum OpenClawEventType {
    #[default]
    RunCreated,
    RunQueued,
    RunStarted,
    RunCompleted,
    RunFailed,
    RunCancelled,
    RunTimedOut,
    AssistantDelta,
    AssistantMessage,
    ThinkingDelta,
    ToolCallStarted,
    ToolCallDelta,
    ToolCallCompleted,
    ToolCallFailed,
    ApprovalRequested,
    ApprovalResolved,
    QuestionRequested,
    QuestionAnswered,
    ArtifactCreated,
    ArtifactUpdated,
    SessionCreated,
    SessionUpdated,
    SessionCompacted,
    TaskUpdated,
    GitBranch,
    GitDiff,
    GitPr,
    Raw,
}

impl OpenClawEventType {
    pub fn as_str(&self) -> &'static str {
        match self {
            OpenClawEventType::RunCreated => "run.created",
            OpenClawEventType::RunQueued => "run.queued",
            OpenClawEventType::RunStarted => "run.started",
            OpenClawEventType::RunCompleted => "run.completed",
            OpenClawEventType::RunFailed => "run.failed",
            OpenClawEventType::RunCancelled => "run.cancelled",
            OpenClawEventType::RunTimedOut => "run.timed_out",
            OpenClawEventType::AssistantDelta => "assistant.delta",
            OpenClawEventType::AssistantMessage => "assistant.message",
            OpenClawEventType::ThinkingDelta => "thinking.delta",
            OpenClawEventType::ToolCallStarted => "tool.call.started",
            OpenClawEventType::ToolCallDelta => "tool.call.delta",
            OpenClawEventType::ToolCallCompleted => "tool.call.completed",
            OpenClawEventType::ToolCallFailed => "tool.call.failed",
            OpenClawEventType::ApprovalRequested => "approval.requested",
            OpenClawEventType::ApprovalResolved => "approval.resolved",
            OpenClawEventType::QuestionRequested => "question.requested",
            OpenClawEventType::QuestionAnswered => "question.answered",
            OpenClawEventType::ArtifactCreated => "artifact.created",
            OpenClawEventType::ArtifactUpdated => "artifact.updated",
            OpenClawEventType::SessionCreated => "session.created",
            OpenClawEventType::SessionUpdated => "session.updated",
            OpenClawEventType::SessionCompacted => "session.compacted",
            OpenClawEventType::TaskUpdated => "task.updated",
            OpenClawEventType::GitBranch => "git.branch",
            OpenClawEventType::GitDiff => "git.diff",
            OpenClawEventType::GitPr => "git.pr",
            OpenClawEventType::Raw => "raw",
        }
    }
}

/// Normalized SDK event with common run/session/task metadata.
#[derive(Clone, Debug, Default)]
pub struct OpenClawEvent {
    pub version: u32,
    pub id: String,
    pub ts: u64,
    pub r#type: OpenClawEventType,
    pub run_id: Option<String>,
    pub session_id: Option<String>,
    pub session_key: Option<String>,
    pub task_id: Option<String>,
    pub agent_id: Option<String>,
    pub data: Option<Value>,
    pub raw: Option<GatewayEvent>,
}

/// Parameters for creating an agent run.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct AgentRunParams {
    pub input: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub agent_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub thinking: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub session_key: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub deliver: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub attachments: Option<Vec<Value>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub timeout_ms: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub runtime: Option<RuntimeSelection>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub environment: Option<EnvironmentSelection>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub workspace: Option<WorkspaceSelection>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub approvals: Option<ApprovalMode>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub idempotency_key: Option<String>,
}

/// Parameters for creating a session.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct SessionCreateParams {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub key: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub agent_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent_session_key: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub task: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

/// Parameters for sending a message to an existing session.
#[derive(Clone, Debug)]
pub struct SessionSendParams {
    pub key: String,
    pub message: String,
    pub thinking: Option<String>,
    pub attachments: Option<Vec<Value>>,
    pub timeout_ms: Option<u64>,
    pub idempotency_key: Option<String>,
}

#[derive(Clone, Debug, Default)]
pub struct SessionTarget {
    pub key: String,
    pub session_id: Option<String>,
    pub agent_id: Option<String>,
    pub label: Option<String>,
}

pub type RunCreateParams = AgentRunParams;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AgentsCreateParams {
    pub name: String,
    pub workspace: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub emoji: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub avatar: Option<String>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct AgentsUpdateParams {
    pub agent_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub workspace: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub emoji: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub avatar: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AgentsDeleteParams {
    pub agent_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub delete_files: Option<bool>,
}

pub type SdkError = SDKError;
