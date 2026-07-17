// Gateway Protocol schema: protocol-schemas registry.
// 翻译自 packages/gateway-protocol/src/schema/protocol-schemas.ts
//
// Central registry for every gateway protocol schema. The TS source maps each
// stable public schema name to a runtime TypeBox schema. In Rust, schemas are
// types in their owning modules; this module keeps a parallel **name-based
// registry** so consumers can:
//   - look up a schema by its public name (string),
//   - introspect its coarse category (request / result / event / ...),
//   - re-export the canonical type for the owning module.
//
// 新增 schema 时, 同步在本文件添加一条 `ProtocolSchemaEntry`, 并在对应模块暴露
// 公共类型. 公共类型已通过 re-export 暴露在 `pub use ...` 块中.

use serde_json::Value;

// =============================================================================
// Re-exports of every schema type from its owning module.
// 命名约定: 公共类型 (无 `Schema` 后缀) 在这里 re-export, 注册表里使用相同名字.
// 实际类型名 (`*Schema` 后缀的结构体) 在各模块中.
// =============================================================================

// --- frames (transport envelopes) ---
pub use crate::frames::{ConnectParams, ErrorShape, EventFrame, GatewayFrame, HelloOk, RequestFrame, ResponseFrame, ShutdownEvent, TickEvent};

// --- snapshot ---
pub use crate::schema::snapshot::{PresenceEntry, Snapshot, StateVersion};

// --- gateway-suspend ---
pub use crate::schema::gateway_suspend::{
    GatewaySuspendBlocker, GatewaySuspendPrepareParams, GatewaySuspendPrepareResult,
    GatewaySuspendResumeParams, GatewaySuspendResumeResult, GatewaySuspendStatusParams,
    GatewaySuspendStatusResult, GatewaySuspendTaskBlocker,
};

// --- environments ---
pub use crate::schema::environments::{
    EnvironmentStatus, EnvironmentSummary, EnvironmentsCreateParams, EnvironmentsCreateResult,
    EnvironmentsDestroyParams, EnvironmentsDestroyResult, EnvironmentsListParams,
    EnvironmentsListResult, EnvironmentsStatusParams, EnvironmentsStatusResult,
    WorkerEnvironmentMetadata, WorkerEnvironmentState, WorkerTunnelStatus,
};

// --- system_info ---
pub use crate::schema::system_info::{SystemInfoParams, SystemInfoResult};

// --- agent ---
pub use crate::schema::agent::{AgentEvent, AgentIdentityParams, AgentIdentityResult, AgentWaitParams, MessageActionParams, PollParams, WakeParams};
// AgentParams, SendParams currently exposed as `AgentParamsSchema`, `SendParamsSchema`.

// --- worktrees ---
pub use crate::schema::worktrees::{
    WorktreeBranch, WorktreeRecord, WorktreesBranchesParams, WorktreesBranchesResult,
    WorktreesCreateParams, WorktreesGcParams, WorktreesGcResult, WorktreesListParams,
    WorktreesListResult, WorktreesRemoveParams, WorktreesRemoveResult, WorktreesRestoreParams,
};

// --- fs ---
pub use crate::schema::fs::{FsDirEntry, FsListDirParams, FsListDirResult};

// --- nodes ---
pub use crate::schema::nodes::{
    NodeDescribeParams, NodeEventParams, NodeEventResult, NodeInvokeParams,
    NodeInvokeResultParams, NodeListParams, NodePairApproveParams, NodePairListParams,
    NodePairRemoveParams, NodePairRejectParams, NodePendingAckParams, NodePendingDrainParams,
    NodePendingDrainResult, NodePendingEnqueueParams, NodePendingEnqueueResult,
    NodePluginToolDescriptor, NodePluginToolsUpdateParams, NodePresenceActivityPayload,
    NodePresenceAlivePayload, NodePresenceAliveReason, NodeRenameParams, NodeSkillDescriptor,
    NodeSkillsUpdateParams,
};
// NodeInvokeRequestEvent currently lives in nodes module as a struct only (no alias yet).

// --- push ---
pub use crate::schema::push::{PushTestParams, PushTestResult};

// --- secrets ---
pub use crate::schema::secrets::{
    SecretsReloadParams, SecretsResolveAssignment, SecretsResolveParams, SecretsResolveResult,
};

// --- sessions / sessions-catalog (sessions module owns all of them) ---
pub use crate::schema::sessions::{
    SessionCompactionCheckpoint, SessionDiffFile, SessionDiffFileStatus, SessionFileBrowserEntry,
    SessionFileBrowserResult, SessionFileEntry, SessionFileKind, SessionFileRelevance, SessionGroup,
    SessionOperationEvent, SessionWorktreeInfo, SessionsAbortParams, SessionsCleanupParams,
    SessionsCompactParams, SessionsCompactionBranchParams, SessionsCompactionBranchResult,
    SessionsCompactionGetParams, SessionsCompactionGetResult, SessionsCompactionListParams,
    SessionsCompactionListResult, SessionsCompactionRestoreParams,
    SessionsCompactionRestoreResult, SessionsCreateParams, SessionsCreateResult,
    SessionsDeleteParams, SessionsDescribeParams, SessionsDiffParams, SessionsDiffResult,
    SessionsFilesGetParams, SessionsFilesGetResult, SessionsFilesListParams,
    SessionsFilesListResult, SessionsFilesSetParams, SessionsFilesSetResult,
    SessionsGroupsDeleteParams, SessionsGroupsListParams, SessionsGroupsListResult,
    SessionsGroupsMutationResult, SessionsGroupsPutParams, SessionsGroupsRenameParams,
    SessionsListParams, SessionsMessagesSubscribeParams, SessionsMessagesUnsubscribeParams,
    SessionsPluginPatchParams, SessionsPluginPatchResult,
    SessionsPreviewParams, SessionsResetParams, SessionsResolveParams, SessionsSearchHit,
    SessionsSearchParams, SessionsSearchResult, SessionsSendParams, SessionsUsageParams,
};
// SessionCatalog* / SessionsCatalog* currently have no type aliases in sessions.rs.
// SessionsPatchParams has no type alias; the struct lives in sessions.rs.

// --- audit ---
pub use crate::schema::audit::{AuditEvent, AuditListParams, AuditListResult};

// --- audit-activity ---
pub use crate::schema::audit_activity::{
    AuditActivityAgentRunV1, AuditActivityEventV1, AuditActivityInboundMessageV1,
    AuditActivityListParams, AuditActivityListResult, AuditActivityOutboundMessageV1,
    AuditActivityToolActionV1,
};

// --- tasks ---
pub use crate::schema::tasks::{
    TaskSummary, TasksCancelParams, TasksCancelResult, TasksGetParams, TasksGetResult,
    TasksListParams, TasksListResult,
};

// --- task-suggestions ---
pub use crate::schema::task_suggestions::{
    TaskSuggestion, TaskSuggestionEvent, TaskSuggestionResolution, TaskSuggestionsAcceptParams,
    TaskSuggestionsAcceptResult, TaskSuggestionsCreateParams, TaskSuggestionsCreateResult,
    TaskSuggestionsDismissParams, TaskSuggestionsDismissResult, TaskSuggestionsListParams,
    TaskSuggestionsListResult,
};

// --- config ---
pub use crate::schema::config::{
    ConfigApplyParams, ConfigGetParams, ConfigPatchParams, ConfigSchemaLookupParams,
    ConfigSchemaLookupResult, ConfigSchemaParams, ConfigSchemaResponse, ConfigSetParams,
    UpdateRunParams, UpdateStatusParams,
};

// --- crestodian ---
pub use crate::schema::crestodian::{
    CrestodianChatParams, CrestodianChatResult, CrestodianSetupActivateParams,
    CrestodianSetupActivateResult, CrestodianSetupAuthStartParams,
    CrestodianSetupAuthStartResult, CrestodianSetupDetectParams, CrestodianSetupDetectResult,
    CrestodianSetupVerifyParams, CrestodianSetupVerifyResult,
};

// --- wizard ---
pub use crate::schema::wizard::{
    WizardCancelParams, WizardNextParams, WizardNextResult, WizardStartParams, WizardStartResult,
    WizardStatusParams, WizardStatusResult, WizardStep,
};

// --- channels (Talk / Channels) ---
pub use crate::schema::channels::{
    ChannelsLogoutParams, ChannelsStartParams, ChannelsStatusParams, ChannelsStatusResult,
    ChannelsStopParams, TalkAgentControlResult, TalkCatalogParams, TalkCatalogResult,
    TalkClientCreateParams, TalkClientCreateResult, TalkClientSteerParams,
    TalkClientToolCallParams, TalkClientToolCallResult, TalkConfigParams, TalkConfigResult,
    TalkEvent, TalkModeParams, TalkSessionAppendAudioParams, TalkSessionCancelOutputParams,
    TalkSessionCancelTurnParams, TalkSessionCloseParams, TalkSessionCreateParams,
    TalkSessionCreateResult, TalkSessionJoinParams, TalkSessionJoinResult, TalkSessionOkResult,
    TalkSessionSteerParams, TalkSessionSubmitToolResultParams, TalkSessionTurnParams,
    TalkSessionTurnResult, TalkSpeakParams, TalkSpeakResult, TtsSpeakParams, TtsSpeakResult,
    WebLoginStartParams, WebLoginWaitParams,
};

// --- logs_chat ---
pub use crate::schema::logs_chat::{
    ChatAbortParams, ChatInjectParams, ChatEvent, ChatMetadataParams, ChatToolTitlesParams,
    LogsTailParams, LogsTailResult,
};
// ChatHistoryParams, ChatMessageGetParams, ChatMessageGetResult, ChatSendParams,
// ChatToolTitlesResult, ChatAbortedEvent, ChatDeltaEvent, ChatErrorEvent, ChatFinalEvent
// currently have no type aliases in logs_chat.rs.

// --- approvals ---
pub use crate::schema::approvals::{
    ApprovalAllowDecision, ApprovalAllowedReason, ApprovalCancelledReason, ApprovalDecision,
    ApprovalDeniedReason, ApprovalExpiredReason, ApprovalKind, ApprovalTerminalReason,
};
pub use crate::schema::plugin_approvals::PluginApprovalSeverity;
// AllowedApprovalSnapshot, ApprovalGetParams, ApprovalGetResult, ApprovalPresentation,
// ApprovalResolveParams, ApprovalResolveResult, ApprovalSnapshot, CancelledApprovalSnapshot,
// DeniedApprovalSnapshot, ExecApprovalPresentation, ExpiredApprovalSnapshot,
// PendingApprovalSnapshot, PendingSessionApprovalEvent, PluginApprovalPresentation,
// SessionApprovalEvent, SessionApprovalReplay, TerminalApprovalSnapshot,
// TerminalSessionApprovalEvent currently have no type aliases in approvals.rs.

// --- exec-approvals ---
pub use crate::schema::exec_approvals::{
    ExecApprovalGetParams, ExecApprovalRequestParams, ExecApprovalResolveParams,
    ExecApprovalsGetParams, ExecApprovalsNodeGetParams, ExecApprovalsNodeSetParams,
    ExecApprovalsNodeSnapshot, ExecApprovalsSetParams, ExecApprovalsSnapshot,
};

// --- plugin-approvals ---
pub use crate::schema::plugin_approvals::{PluginApprovalRequestParams, PluginApprovalResolveParams};

// --- plugins ---
// (no `*Type` aliases in plugins.rs; types only as `*Schema` structs.)

// --- devices ---
// (no type aliases in devices.rs; types only as `*Schema` structs.)

// --- agents_workspace ---
pub use crate::schema::agents_workspace::{
    AgentsWorkspaceEntry, AgentsWorkspaceFile, AgentsWorkspaceGetParams,
    AgentsWorkspaceGetResult, AgentsWorkspaceListParams, AgentsWorkspaceListResult,
};

// --- artifacts ---
pub use crate::schema::artifacts::{
    ArtifactSummary, ArtifactsDownloadParams, ArtifactsDownloadResult, ArtifactsGetParams,
    ArtifactsGetResult, ArtifactsListParams, ArtifactsListResult,
};

// --- commands ---
pub use crate::schema::commands::{CommandEntry, CommandsListParams, CommandsListResult};

// --- cron ---
pub use crate::schema::cron::{
    CronAddParams, CronAddResult, CronDeclarativeAddResult, CronGetParams, CronJob,
    CronListParams, CronRemoveParams, CronRunLogEntry, CronRunParams, CronRunsParams,
    CronStatusParams, CronUpdateParams,
};

// --- terminal ---
pub use crate::schema::terminal::{
    TerminalAckResult, TerminalAttachParams, TerminalAttachResult, TerminalCloseParams,
    TerminalDataEvent, TerminalEvent, TerminalExitEvent, TerminalInputParams,
    TerminalListResult, TerminalOpenParams, TerminalOpenResult, TerminalResizeParams,
    TerminalSessionInfo, TerminalTextParams, TerminalTextResult,
};

// =============================================================================
// Protocol schema registry
// =============================================================================

/// Coarse category for a registered schema.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ProtocolSchemaKind {
    /// Handshake / transport envelope.
    Transport,
    /// Snapshot / state payload.
    Snapshot,
    /// Request parameters (request `params` field).
    Request,
    /// Result / response payload.
    Result,
    /// Server-pushed event payload.
    Event,
    /// Shared error envelope.
    Error,
    /// Misc supporting type.
    Other,
}

/// Single entry in the central protocol schema registry.
///
/// `name` is the stable public schema name (matches the TypeBox key in the TS
/// source). `module` is the Rust module path that owns the canonical type.
/// `kind` is the coarse category for filtering. `sample` produces a
/// representative `serde_json::Value` (defaults to `Value::Null`).
#[derive(Debug, Clone, Copy)]
pub struct ProtocolSchemaEntry {
    pub name: &'static str,
    pub kind: ProtocolSchemaKind,
    pub module: &'static str,
    pub sample: fn() -> Value,
}

impl ProtocolSchemaEntry {
    pub const fn new(
        name: &'static str,
        kind: ProtocolSchemaKind,
        module: &'static str,
        sample: fn() -> Value,
    ) -> Self {
        Self { name, kind, module, sample }
    }

    pub fn sample_value(&self) -> Value {
        (self.sample)()
    }
}

/// Null sample used for every entry. Each schema's canonical type is in its
/// owning module; the registry only stores the *name* of the schema.
fn null_sample() -> Value {
    Value::Null
}

/// Public schema registry keyed by stable protocol schema name.
///
/// This slice is the Rust analog of the TypeScript `ProtocolSchemas` object.
/// New schemas must be appended here in the same order as the TS source so
/// downstream tooling can rely on stable indices.
pub const PROTOCOL_SCHEMAS: &[ProtocolSchemaEntry] = &[
    // --- Handshake, transport frames, state snapshots, and shared error envelopes. ---
    ProtocolSchemaEntry::new("ConnectParams", ProtocolSchemaKind::Transport, "crate::frames::ConnectParams", null_sample),
    ProtocolSchemaEntry::new("WorkerAdmissionHandshake", ProtocolSchemaKind::Transport, "crate::schema::worker_admission::WorkerAdmissionHandshakeSchema", null_sample),
    ProtocolSchemaEntry::new("HelloOk", ProtocolSchemaKind::Transport, "crate::frames::HelloOk", null_sample),
    ProtocolSchemaEntry::new("RequestFrame", ProtocolSchemaKind::Transport, "crate::frames::RequestFrame", null_sample),
    ProtocolSchemaEntry::new("ResponseFrame", ProtocolSchemaKind::Transport, "crate::frames::ResponseFrame", null_sample),
    ProtocolSchemaEntry::new("EventFrame", ProtocolSchemaKind::Transport, "crate::frames::EventFrame", null_sample),
    ProtocolSchemaEntry::new("GatewayFrame", ProtocolSchemaKind::Transport, "crate::frames::GatewayFrame", null_sample),
    ProtocolSchemaEntry::new("PresenceEntry", ProtocolSchemaKind::Snapshot, "crate::schema::snapshot::PresenceEntry", null_sample),
    ProtocolSchemaEntry::new("StateVersion", ProtocolSchemaKind::Snapshot, "crate::schema::snapshot::StateVersion", null_sample),
    ProtocolSchemaEntry::new("Snapshot", ProtocolSchemaKind::Snapshot, "crate::schema::snapshot::Snapshot", null_sample),
    ProtocolSchemaEntry::new("ErrorShape", ProtocolSchemaKind::Error, "crate::frames::ErrorShape", null_sample),
    ProtocolSchemaEntry::new("GatewaySuspendTaskBlocker", ProtocolSchemaKind::Other, "crate::schema::gateway_suspend::GatewaySuspendTaskBlocker", null_sample),
    ProtocolSchemaEntry::new("GatewaySuspendBlocker", ProtocolSchemaKind::Other, "crate::schema::gateway_suspend::GatewaySuspendBlocker", null_sample),
    ProtocolSchemaEntry::new("GatewaySuspendPrepareParams", ProtocolSchemaKind::Request, "crate::schema::gateway_suspend::GatewaySuspendPrepareParams", null_sample),
    ProtocolSchemaEntry::new("GatewaySuspendPrepareBusyResult", ProtocolSchemaKind::Result, "crate::schema::gateway_suspend::GatewaySuspendPrepareResult", null_sample),
    ProtocolSchemaEntry::new("GatewaySuspendPrepareReadyResult", ProtocolSchemaKind::Result, "crate::schema::gateway_suspend::GatewaySuspendPrepareResult", null_sample),
    ProtocolSchemaEntry::new("GatewaySuspendPrepareResult", ProtocolSchemaKind::Result, "crate::schema::gateway_suspend::GatewaySuspendPrepareResult", null_sample),
    ProtocolSchemaEntry::new("GatewaySuspendStatusParams", ProtocolSchemaKind::Request, "crate::schema::gateway_suspend::GatewaySuspendStatusParams", null_sample),
    ProtocolSchemaEntry::new("GatewaySuspendStatusRunningResult", ProtocolSchemaKind::Result, "crate::schema::gateway_suspend::GatewaySuspendStatusResult", null_sample),
    ProtocolSchemaEntry::new("GatewaySuspendStatusReadyResult", ProtocolSchemaKind::Result, "crate::schema::gateway_suspend::GatewaySuspendStatusResult", null_sample),
    ProtocolSchemaEntry::new("GatewaySuspendStatusResult", ProtocolSchemaKind::Result, "crate::schema::gateway_suspend::GatewaySuspendStatusResult", null_sample),
    ProtocolSchemaEntry::new("GatewaySuspendResumeParams", ProtocolSchemaKind::Request, "crate::schema::gateway_suspend::GatewaySuspendResumeParams", null_sample),
    ProtocolSchemaEntry::new("GatewaySuspendResumeResult", ProtocolSchemaKind::Result, "crate::schema::gateway_suspend::GatewaySuspendResumeResult", null_sample),

    // --- Environment and agent-facing control RPC payloads. ---
    ProtocolSchemaEntry::new("EnvironmentStatus", ProtocolSchemaKind::Result, "crate::schema::environments::EnvironmentStatus", null_sample),
    ProtocolSchemaEntry::new("WorkerEnvironmentState", ProtocolSchemaKind::Other, "crate::schema::environments::WorkerEnvironmentState", null_sample),
    ProtocolSchemaEntry::new("WorkerTunnelStatus", ProtocolSchemaKind::Other, "crate::schema::environments::WorkerTunnelStatus", null_sample),
    ProtocolSchemaEntry::new("WorkerEnvironmentMetadata", ProtocolSchemaKind::Other, "crate::schema::environments::WorkerEnvironmentMetadata", null_sample),
    ProtocolSchemaEntry::new("EnvironmentSummary", ProtocolSchemaKind::Result, "crate::schema::environments::EnvironmentSummary", null_sample),
    ProtocolSchemaEntry::new("EnvironmentsCreateParams", ProtocolSchemaKind::Request, "crate::schema::environments::EnvironmentsCreateParams", null_sample),
    ProtocolSchemaEntry::new("EnvironmentsCreateResult", ProtocolSchemaKind::Result, "crate::schema::environments::EnvironmentsCreateResult", null_sample),
    ProtocolSchemaEntry::new("EnvironmentsDestroyParams", ProtocolSchemaKind::Request, "crate::schema::environments::EnvironmentsDestroyParams", null_sample),
    ProtocolSchemaEntry::new("EnvironmentsDestroyResult", ProtocolSchemaKind::Result, "crate::schema::environments::EnvironmentsDestroyResult", null_sample),
    ProtocolSchemaEntry::new("EnvironmentsListParams", ProtocolSchemaKind::Request, "crate::schema::environments::EnvironmentsListParams", null_sample),
    ProtocolSchemaEntry::new("EnvironmentsListResult", ProtocolSchemaKind::Result, "crate::schema::environments::EnvironmentsListResult", null_sample),
    ProtocolSchemaEntry::new("EnvironmentsStatusParams", ProtocolSchemaKind::Request, "crate::schema::environments::EnvironmentsStatusParams", null_sample),
    ProtocolSchemaEntry::new("EnvironmentsStatusResult", ProtocolSchemaKind::Result, "crate::schema::environments::EnvironmentsStatusResult", null_sample),
    ProtocolSchemaEntry::new("SystemInfoParams", ProtocolSchemaKind::Request, "crate::schema::system_info::SystemInfoParams", null_sample),
    ProtocolSchemaEntry::new("SystemInfoResult", ProtocolSchemaKind::Result, "crate::schema::system_info::SystemInfoResult", null_sample),
    ProtocolSchemaEntry::new("AgentEvent", ProtocolSchemaKind::Event, "crate::schema::agent::AgentEvent", null_sample),
    ProtocolSchemaEntry::new("MessageActionParams", ProtocolSchemaKind::Request, "crate::schema::agent::MessageActionParams", null_sample),
    ProtocolSchemaEntry::new("SendParams", ProtocolSchemaKind::Request, "crate::schema::agent::SendParamsSchema", null_sample),
    ProtocolSchemaEntry::new("PollParams", ProtocolSchemaKind::Request, "crate::schema::agent::PollParams", null_sample),
    ProtocolSchemaEntry::new("AgentParams", ProtocolSchemaKind::Request, "crate::schema::agent::AgentParamsSchema", null_sample),
    ProtocolSchemaEntry::new("AgentIdentityParams", ProtocolSchemaKind::Request, "crate::schema::agent::AgentIdentityParams", null_sample),
    ProtocolSchemaEntry::new("AgentIdentityResult", ProtocolSchemaKind::Result, "crate::schema::agent::AgentIdentityResult", null_sample),
    ProtocolSchemaEntry::new("AgentWaitParams", ProtocolSchemaKind::Request, "crate::schema::agent::AgentWaitParams", null_sample),
    ProtocolSchemaEntry::new("WakeParams", ProtocolSchemaKind::Request, "crate::schema::agent::WakeParams", null_sample),
    ProtocolSchemaEntry::new("WorktreeRecord", ProtocolSchemaKind::Other, "crate::schema::worktrees::WorktreeRecordSchema", null_sample),
    ProtocolSchemaEntry::new("WorktreesListParams", ProtocolSchemaKind::Request, "crate::schema::worktrees::WorktreesListParams", null_sample),
    ProtocolSchemaEntry::new("WorktreesListResult", ProtocolSchemaKind::Result, "crate::schema::worktrees::WorktreesListResult", null_sample),
    ProtocolSchemaEntry::new("WorktreesCreateParams", ProtocolSchemaKind::Request, "crate::schema::worktrees::WorktreesCreateParams", null_sample),
    ProtocolSchemaEntry::new("WorktreesRemoveParams", ProtocolSchemaKind::Request, "crate::schema::worktrees::WorktreesRemoveParams", null_sample),
    ProtocolSchemaEntry::new("WorktreesRemoveResult", ProtocolSchemaKind::Result, "crate::schema::worktrees::WorktreesRemoveResult", null_sample),
    ProtocolSchemaEntry::new("WorktreesRestoreParams", ProtocolSchemaKind::Request, "crate::schema::worktrees::WorktreesRestoreParams", null_sample),
    ProtocolSchemaEntry::new("WorktreesGcParams", ProtocolSchemaKind::Request, "crate::schema::worktrees::WorktreesGcParams", null_sample),
    ProtocolSchemaEntry::new("WorktreesGcResult", ProtocolSchemaKind::Result, "crate::schema::worktrees::WorktreesGcResult", null_sample),
    ProtocolSchemaEntry::new("WorktreeBranch", ProtocolSchemaKind::Other, "crate::schema::worktrees::WorktreeBranchSchema", null_sample),
    ProtocolSchemaEntry::new("WorktreesBranchesParams", ProtocolSchemaKind::Request, "crate::schema::worktrees::WorktreesBranchesParams", null_sample),
    ProtocolSchemaEntry::new("WorktreesBranchesResult", ProtocolSchemaKind::Result, "crate::schema::worktrees::WorktreesBranchesResult", null_sample),
    ProtocolSchemaEntry::new("FsDirEntry", ProtocolSchemaKind::Other, "crate::schema::fs::FsDirEntrySchema", null_sample),
    ProtocolSchemaEntry::new("FsListDirParams", ProtocolSchemaKind::Request, "crate::schema::fs::FsListDirParamsSchema", null_sample),
    ProtocolSchemaEntry::new("FsListDirResult", ProtocolSchemaKind::Result, "crate::schema::fs::FsListDirResultSchema", null_sample),

    // --- Node pairing, invocation, presence, and pending-queue payloads. ---
    ProtocolSchemaEntry::new("NodePairListParams", ProtocolSchemaKind::Request, "crate::schema::nodes::NodePairListParams", null_sample),
    ProtocolSchemaEntry::new("NodePairApproveParams", ProtocolSchemaKind::Request, "crate::schema::nodes::NodePairApproveParams", null_sample),
    ProtocolSchemaEntry::new("NodePairRejectParams", ProtocolSchemaKind::Request, "crate::schema::nodes::NodePairRejectParams", null_sample),
    ProtocolSchemaEntry::new("NodePairRemoveParams", ProtocolSchemaKind::Request, "crate::schema::nodes::NodePairRemoveParams", null_sample),
    ProtocolSchemaEntry::new("NodeRenameParams", ProtocolSchemaKind::Request, "crate::schema::nodes::NodeRenameParams", null_sample),
    ProtocolSchemaEntry::new("NodeListParams", ProtocolSchemaKind::Request, "crate::schema::nodes::NodeListParams", null_sample),
    ProtocolSchemaEntry::new("NodePluginToolDescriptor", ProtocolSchemaKind::Other, "crate::schema::nodes::NodePluginToolDescriptor", null_sample),
    ProtocolSchemaEntry::new("NodePluginToolsUpdateParams", ProtocolSchemaKind::Request, "crate::schema::nodes::NodePluginToolsUpdateParams", null_sample),
    ProtocolSchemaEntry::new("NodeSkillDescriptor", ProtocolSchemaKind::Other, "crate::schema::nodes::NodeSkillDescriptor", null_sample),
    ProtocolSchemaEntry::new("NodeSkillsUpdateParams", ProtocolSchemaKind::Request, "crate::schema::nodes::NodeSkillsUpdateParams", null_sample),
    ProtocolSchemaEntry::new("NodePendingAckParams", ProtocolSchemaKind::Request, "crate::schema::nodes::NodePendingAckParams", null_sample),
    ProtocolSchemaEntry::new("NodeDescribeParams", ProtocolSchemaKind::Request, "crate::schema::nodes::NodeDescribeParams", null_sample),
    ProtocolSchemaEntry::new("NodeInvokeParams", ProtocolSchemaKind::Request, "crate::schema::nodes::NodeInvokeParams", null_sample),
    ProtocolSchemaEntry::new("NodeInvokeResultParams", ProtocolSchemaKind::Request, "crate::schema::nodes::NodeInvokeResultParams", null_sample),
    ProtocolSchemaEntry::new("NodeEventParams", ProtocolSchemaKind::Request, "crate::schema::nodes::NodeEventParams", null_sample),
    ProtocolSchemaEntry::new("NodeEventResult", ProtocolSchemaKind::Result, "crate::schema::nodes::NodeEventResult", null_sample),
    ProtocolSchemaEntry::new("NodePresenceAlivePayload", ProtocolSchemaKind::Event, "crate::schema::nodes::NodePresenceAlivePayload", null_sample),
    ProtocolSchemaEntry::new("NodePresenceAliveReason", ProtocolSchemaKind::Other, "crate::schema::nodes::NodePresenceAliveReason", null_sample),
    ProtocolSchemaEntry::new("NodePresenceActivityPayload", ProtocolSchemaKind::Event, "crate::schema::nodes::NodePresenceActivityPayload", null_sample),
    ProtocolSchemaEntry::new("NodePendingDrainParams", ProtocolSchemaKind::Request, "crate::schema::nodes::NodePendingDrainParams", null_sample),
    ProtocolSchemaEntry::new("NodePendingDrainResult", ProtocolSchemaKind::Result, "crate::schema::nodes::NodePendingDrainResult", null_sample),
    ProtocolSchemaEntry::new("NodePendingEnqueueParams", ProtocolSchemaKind::Request, "crate::schema::nodes::NodePendingEnqueueParams", null_sample),
    ProtocolSchemaEntry::new("NodePendingEnqueueResult", ProtocolSchemaKind::Result, "crate::schema::nodes::NodePendingEnqueueResult", null_sample),
    ProtocolSchemaEntry::new("NodeInvokeRequestEvent", ProtocolSchemaKind::Event, "crate::schema::nodes::NodeInvokeRequestEventSchema", null_sample),

    // --- Push and secret-resolution payloads used by mobile/control integrations. ---
    ProtocolSchemaEntry::new("PushTestParams", ProtocolSchemaKind::Request, "crate::schema::push::PushTestParams", null_sample),
    ProtocolSchemaEntry::new("PushTestResult", ProtocolSchemaKind::Result, "crate::schema::push::PushTestResult", null_sample),
    ProtocolSchemaEntry::new("SecretsReloadParams", ProtocolSchemaKind::Request, "crate::schema::secrets::SecretsReloadParams", null_sample),
    ProtocolSchemaEntry::new("SecretsResolveParams", ProtocolSchemaKind::Request, "crate::schema::secrets::SecretsResolveParams", null_sample),
    ProtocolSchemaEntry::new("SecretsResolveAssignment", ProtocolSchemaKind::Other, "crate::schema::secrets::SecretsResolveAssignment", null_sample),
    ProtocolSchemaEntry::new("SecretsResolveResult", ProtocolSchemaKind::Result, "crate::schema::secrets::SecretsResolveResult", null_sample),

    // --- Session lifecycle, message routing, compaction, and usage accounting. ---
    ProtocolSchemaEntry::new("SessionsListParams", ProtocolSchemaKind::Request, "crate::schema::sessions::SessionsListParams", null_sample),
    ProtocolSchemaEntry::new("SessionCatalogCapabilities", ProtocolSchemaKind::Other, "crate::schema::sessions_catalog::SessionCatalogCapabilitiesSchema", null_sample),
    ProtocolSchemaEntry::new("SessionCatalogDescriptor", ProtocolSchemaKind::Other, "crate::schema::sessions_catalog::SessionCatalogDescriptorSchema", null_sample),
    ProtocolSchemaEntry::new("SessionCatalogSession", ProtocolSchemaKind::Other, "crate::schema::sessions_catalog::SessionCatalogSessionSchema", null_sample),
    ProtocolSchemaEntry::new("SessionCatalogHost", ProtocolSchemaKind::Other, "crate::schema::sessions_catalog::SessionCatalogHostSchema", null_sample),
    ProtocolSchemaEntry::new("SessionCatalog", ProtocolSchemaKind::Other, "crate::schema::sessions_catalog::SessionCatalogSchema", null_sample),
    ProtocolSchemaEntry::new("SessionCatalogTranscriptItem", ProtocolSchemaKind::Other, "crate::schema::sessions_catalog::SessionCatalogTranscriptItemSchema", null_sample),
    ProtocolSchemaEntry::new("SessionsCatalogListParams", ProtocolSchemaKind::Request, "crate::schema::sessions_catalog::SessionsCatalogListParamsSchema", null_sample),
    ProtocolSchemaEntry::new("SessionsCatalogListResult", ProtocolSchemaKind::Result, "crate::schema::sessions_catalog::SessionsCatalogListResultSchema", null_sample),
    ProtocolSchemaEntry::new("SessionsCatalogReadParams", ProtocolSchemaKind::Request, "crate::schema::sessions_catalog::SessionsCatalogReadParamsSchema", null_sample),
    ProtocolSchemaEntry::new("SessionsCatalogReadResult", ProtocolSchemaKind::Result, "crate::schema::sessions_catalog::SessionsCatalogReadResultSchema", null_sample),
    ProtocolSchemaEntry::new("SessionsCatalogContinueParams", ProtocolSchemaKind::Request, "crate::schema::sessions_catalog::SessionsCatalogContinueParamsSchema", null_sample),
    ProtocolSchemaEntry::new("SessionsCatalogContinueResult", ProtocolSchemaKind::Result, "crate::schema::sessions_catalog::SessionsCatalogContinueResultSchema", null_sample),
    ProtocolSchemaEntry::new("SessionsCatalogArchiveParams", ProtocolSchemaKind::Request, "crate::schema::sessions_catalog::SessionsCatalogArchiveParamsSchema", null_sample),
    ProtocolSchemaEntry::new("SessionsCatalogArchiveResult", ProtocolSchemaKind::Result, "crate::schema::sessions_catalog::SessionsCatalogArchiveResultSchema", null_sample),
    ProtocolSchemaEntry::new("SessionsCleanupParams", ProtocolSchemaKind::Request, "crate::schema::sessions::SessionsCleanupParams", null_sample),
    ProtocolSchemaEntry::new("SessionsPreviewParams", ProtocolSchemaKind::Request, "crate::schema::sessions::SessionsPreviewParams", null_sample),
    ProtocolSchemaEntry::new("SessionsDescribeParams", ProtocolSchemaKind::Request, "crate::schema::sessions::SessionsDescribeParams", null_sample),
    ProtocolSchemaEntry::new("SessionsResolveParams", ProtocolSchemaKind::Request, "crate::schema::sessions::SessionsResolveParams", null_sample),
    ProtocolSchemaEntry::new("SessionsSearchHit", ProtocolSchemaKind::Result, "crate::schema::sessions::SessionsSearchHit", null_sample),
    ProtocolSchemaEntry::new("SessionsSearchParams", ProtocolSchemaKind::Request, "crate::schema::sessions::SessionsSearchParams", null_sample),
    ProtocolSchemaEntry::new("SessionsSearchResult", ProtocolSchemaKind::Result, "crate::schema::sessions::SessionsSearchResult", null_sample),
    ProtocolSchemaEntry::new("SessionCompactionCheckpoint", ProtocolSchemaKind::Other, "crate::schema::sessions::SessionCompactionCheckpoint", null_sample),
    ProtocolSchemaEntry::new("SessionOperationEvent", ProtocolSchemaKind::Event, "crate::schema::sessions::SessionOperationEvent", null_sample),
    ProtocolSchemaEntry::new("SessionsCompactionListParams", ProtocolSchemaKind::Request, "crate::schema::sessions::SessionsCompactionListParams", null_sample),
    ProtocolSchemaEntry::new("SessionsCompactionGetParams", ProtocolSchemaKind::Request, "crate::schema::sessions::SessionsCompactionGetParams", null_sample),
    ProtocolSchemaEntry::new("SessionsCompactionBranchParams", ProtocolSchemaKind::Request, "crate::schema::sessions::SessionsCompactionBranchParams", null_sample),
    ProtocolSchemaEntry::new("SessionsCompactionRestoreParams", ProtocolSchemaKind::Request, "crate::schema::sessions::SessionsCompactionRestoreParams", null_sample),
    ProtocolSchemaEntry::new("SessionsCompactionListResult", ProtocolSchemaKind::Result, "crate::schema::sessions::SessionsCompactionListResult", null_sample),
    ProtocolSchemaEntry::new("SessionsCompactionGetResult", ProtocolSchemaKind::Result, "crate::schema::sessions::SessionsCompactionGetResult", null_sample),
    ProtocolSchemaEntry::new("SessionsCompactionBranchResult", ProtocolSchemaKind::Result, "crate::schema::sessions::SessionsCompactionBranchResult", null_sample),
    ProtocolSchemaEntry::new("SessionsCompactionRestoreResult", ProtocolSchemaKind::Result, "crate::schema::sessions::SessionsCompactionRestoreResult", null_sample),
    ProtocolSchemaEntry::new("SessionFileBrowserEntry", ProtocolSchemaKind::Other, "crate::schema::sessions::SessionFileBrowserEntry", null_sample),
    ProtocolSchemaEntry::new("SessionFileBrowserResult", ProtocolSchemaKind::Result, "crate::schema::sessions::SessionFileBrowserResult", null_sample),
    ProtocolSchemaEntry::new("SessionFileKind", ProtocolSchemaKind::Other, "crate::schema::sessions::SessionFileKind", null_sample),
    ProtocolSchemaEntry::new("SessionFileEntry", ProtocolSchemaKind::Other, "crate::schema::sessions::SessionFileEntry", null_sample),
    ProtocolSchemaEntry::new("SessionFileRelevance", ProtocolSchemaKind::Other, "crate::schema::sessions::SessionFileRelevance", null_sample),
    ProtocolSchemaEntry::new("SessionsFilesListParams", ProtocolSchemaKind::Request, "crate::schema::sessions::SessionsFilesListParams", null_sample),
    ProtocolSchemaEntry::new("SessionsFilesListResult", ProtocolSchemaKind::Result, "crate::schema::sessions::SessionsFilesListResult", null_sample),
    ProtocolSchemaEntry::new("SessionsFilesGetParams", ProtocolSchemaKind::Request, "crate::schema::sessions::SessionsFilesGetParams", null_sample),
    ProtocolSchemaEntry::new("SessionsFilesGetResult", ProtocolSchemaKind::Result, "crate::schema::sessions::SessionsFilesGetResult", null_sample),
    ProtocolSchemaEntry::new("SessionsFilesSetParams", ProtocolSchemaKind::Request, "crate::schema::sessions::SessionsFilesSetParams", null_sample),
    ProtocolSchemaEntry::new("SessionsFilesSetResult", ProtocolSchemaKind::Result, "crate::schema::sessions::SessionsFilesSetResult", null_sample),
    ProtocolSchemaEntry::new("SessionDiffFileStatus", ProtocolSchemaKind::Other, "crate::schema::sessions::SessionDiffFileStatus", null_sample),
    ProtocolSchemaEntry::new("SessionDiffFile", ProtocolSchemaKind::Other, "crate::schema::sessions::SessionDiffFile", null_sample),
    ProtocolSchemaEntry::new("SessionsDiffParams", ProtocolSchemaKind::Request, "crate::schema::sessions::SessionsDiffParams", null_sample),
    ProtocolSchemaEntry::new("SessionsDiffResult", ProtocolSchemaKind::Result, "crate::schema::sessions::SessionsDiffResult", null_sample),
    ProtocolSchemaEntry::new("SessionWorktreeInfo", ProtocolSchemaKind::Other, "crate::schema::sessions::SessionWorktreeInfo", null_sample),
    ProtocolSchemaEntry::new("SessionsCreateParams", ProtocolSchemaKind::Request, "crate::schema::sessions::SessionsCreateParams", null_sample),
    ProtocolSchemaEntry::new("SessionsCreateResult", ProtocolSchemaKind::Result, "crate::schema::sessions::SessionsCreateResult", null_sample),
    ProtocolSchemaEntry::new("SessionsSendParams", ProtocolSchemaKind::Request, "crate::schema::sessions::SessionsSendParams", null_sample),
    ProtocolSchemaEntry::new("SessionsMessagesSubscribeParams", ProtocolSchemaKind::Request, "crate::schema::sessions::SessionsMessagesSubscribeParams", null_sample),
    ProtocolSchemaEntry::new("SessionsMessagesUnsubscribeParams", ProtocolSchemaKind::Request, "crate::schema::sessions::SessionsMessagesUnsubscribeParams", null_sample),
    ProtocolSchemaEntry::new("SessionsAbortParams", ProtocolSchemaKind::Request, "crate::schema::sessions::SessionsAbortParams", null_sample),
    ProtocolSchemaEntry::new("SessionsPatchParams", ProtocolSchemaKind::Request, "crate::schema::sessions::SessionsPatchParamsSchema", null_sample),
    ProtocolSchemaEntry::new("SessionsPluginPatchParams", ProtocolSchemaKind::Request, "crate::schema::sessions::SessionsPluginPatchParams", null_sample),
    ProtocolSchemaEntry::new("SessionsPluginPatchResult", ProtocolSchemaKind::Result, "crate::schema::sessions::SessionsPluginPatchResult", null_sample),
    ProtocolSchemaEntry::new("SessionsResetParams", ProtocolSchemaKind::Request, "crate::schema::sessions::SessionsResetParams", null_sample),
    ProtocolSchemaEntry::new("SessionsDeleteParams", ProtocolSchemaKind::Request, "crate::schema::sessions::SessionsDeleteParams", null_sample),
    ProtocolSchemaEntry::new("SessionGroup", ProtocolSchemaKind::Other, "crate::schema::sessions::SessionGroup", null_sample),
    ProtocolSchemaEntry::new("SessionsGroupsListParams", ProtocolSchemaKind::Request, "crate::schema::sessions::SessionsGroupsListParams", null_sample),
    ProtocolSchemaEntry::new("SessionsGroupsListResult", ProtocolSchemaKind::Result, "crate::schema::sessions::SessionsGroupsListResult", null_sample),
    ProtocolSchemaEntry::new("SessionsGroupsPutParams", ProtocolSchemaKind::Request, "crate::schema::sessions::SessionsGroupsPutParams", null_sample),
    ProtocolSchemaEntry::new("SessionsGroupsRenameParams", ProtocolSchemaKind::Request, "crate::schema::sessions::SessionsGroupsRenameParams", null_sample),
    ProtocolSchemaEntry::new("SessionsGroupsDeleteParams", ProtocolSchemaKind::Request, "crate::schema::sessions::SessionsGroupsDeleteParams", null_sample),
    ProtocolSchemaEntry::new("SessionsGroupsMutationResult", ProtocolSchemaKind::Result, "crate::schema::sessions::SessionsGroupsMutationResult", null_sample),
    ProtocolSchemaEntry::new("SessionsCompactParams", ProtocolSchemaKind::Request, "crate::schema::sessions::SessionsCompactParams", null_sample),
    ProtocolSchemaEntry::new("SessionsUsageParams", ProtocolSchemaKind::Request, "crate::schema::sessions::SessionsUsageParams", null_sample),

    // --- Audit/task ledgers and config/wizard setup payloads. ---
    ProtocolSchemaEntry::new("AuditActivityAgentRunV1", ProtocolSchemaKind::Event, "crate::schema::audit_activity::AuditActivityAgentRunV1", null_sample),
    ProtocolSchemaEntry::new("AuditActivityToolActionV1", ProtocolSchemaKind::Event, "crate::schema::audit_activity::AuditActivityToolActionV1", null_sample),
    ProtocolSchemaEntry::new("AuditActivityInboundMessageV1", ProtocolSchemaKind::Event, "crate::schema::audit_activity::AuditActivityInboundMessageV1", null_sample),
    ProtocolSchemaEntry::new("AuditActivityOutboundMessageV1", ProtocolSchemaKind::Event, "crate::schema::audit_activity::AuditActivityOutboundMessageV1", null_sample),
    ProtocolSchemaEntry::new("AuditActivityEventV1", ProtocolSchemaKind::Event, "crate::schema::audit_activity::AuditActivityEventV1", null_sample),
    ProtocolSchemaEntry::new("AuditActivityListParams", ProtocolSchemaKind::Request, "crate::schema::audit_activity::AuditActivityListParams", null_sample),
    ProtocolSchemaEntry::new("AuditActivityListResult", ProtocolSchemaKind::Result, "crate::schema::audit_activity::AuditActivityListResult", null_sample),
    ProtocolSchemaEntry::new("AuditEvent", ProtocolSchemaKind::Event, "crate::schema::audit::AuditEvent", null_sample),
    ProtocolSchemaEntry::new("AuditListParams", ProtocolSchemaKind::Request, "crate::schema::audit::AuditListParams", null_sample),
    ProtocolSchemaEntry::new("AuditListResult", ProtocolSchemaKind::Result, "crate::schema::audit::AuditListResult", null_sample),
    ProtocolSchemaEntry::new("TaskSuggestion", ProtocolSchemaKind::Other, "crate::schema::task_suggestions::TaskSuggestionSchema", null_sample),
    ProtocolSchemaEntry::new("TaskSuggestionEvent", ProtocolSchemaKind::Event, "crate::schema::task_suggestions::TaskSuggestionEventSchema", null_sample),
    ProtocolSchemaEntry::new("TaskSuggestionResolution", ProtocolSchemaKind::Other, "crate::schema::task_suggestions::TaskSuggestionResolutionSchema", null_sample),
    ProtocolSchemaEntry::new("TaskSuggestionsAcceptParams", ProtocolSchemaKind::Request, "crate::schema::task_suggestions::TaskSuggestionsAcceptParamsSchema", null_sample),
    ProtocolSchemaEntry::new("TaskSuggestionsAcceptResult", ProtocolSchemaKind::Result, "crate::schema::task_suggestions::TaskSuggestionsAcceptResultSchema", null_sample),
    ProtocolSchemaEntry::new("TaskSuggestionsCreateParams", ProtocolSchemaKind::Request, "crate::schema::task_suggestions::TaskSuggestionsCreateParamsSchema", null_sample),
    ProtocolSchemaEntry::new("TaskSuggestionsCreateResult", ProtocolSchemaKind::Result, "crate::schema::task_suggestions::TaskSuggestionsCreateResultSchema", null_sample),
    ProtocolSchemaEntry::new("TaskSuggestionsDismissParams", ProtocolSchemaKind::Request, "crate::schema::task_suggestions::TaskSuggestionsDismissParamsSchema", null_sample),
    ProtocolSchemaEntry::new("TaskSuggestionsDismissResult", ProtocolSchemaKind::Result, "crate::schema::task_suggestions::TaskSuggestionsDismissResultSchema", null_sample),
    ProtocolSchemaEntry::new("TaskSuggestionsListParams", ProtocolSchemaKind::Request, "crate::schema::task_suggestions::TaskSuggestionsListParamsSchema", null_sample),
    ProtocolSchemaEntry::new("TaskSuggestionsListResult", ProtocolSchemaKind::Result, "crate::schema::task_suggestions::TaskSuggestionsListResultSchema", null_sample),
    ProtocolSchemaEntry::new("TaskSummary", ProtocolSchemaKind::Other, "crate::schema::tasks::TaskSummarySchema", null_sample),
    ProtocolSchemaEntry::new("TasksListParams", ProtocolSchemaKind::Request, "crate::schema::tasks::TasksListParamsSchema", null_sample),
    ProtocolSchemaEntry::new("TasksListResult", ProtocolSchemaKind::Result, "crate::schema::tasks::TasksListResultSchema", null_sample),
    ProtocolSchemaEntry::new("TasksGetParams", ProtocolSchemaKind::Request, "crate::schema::tasks::TasksGetParamsSchema", null_sample),
    ProtocolSchemaEntry::new("TasksGetResult", ProtocolSchemaKind::Result, "crate::schema::tasks::TasksGetResultSchema", null_sample),
    ProtocolSchemaEntry::new("TasksCancelParams", ProtocolSchemaKind::Request, "crate::schema::tasks::TasksCancelParamsSchema", null_sample),
    ProtocolSchemaEntry::new("TasksCancelResult", ProtocolSchemaKind::Result, "crate::schema::tasks::TasksCancelResultSchema", null_sample),
    ProtocolSchemaEntry::new("ConfigGetParams", ProtocolSchemaKind::Request, "crate::schema::config::ConfigGetParams", null_sample),
    ProtocolSchemaEntry::new("ConfigSetParams", ProtocolSchemaKind::Request, "crate::schema::config::ConfigSetParams", null_sample),
    ProtocolSchemaEntry::new("ConfigApplyParams", ProtocolSchemaKind::Request, "crate::schema::config::ConfigApplyParams", null_sample),
    ProtocolSchemaEntry::new("ConfigPatchParams", ProtocolSchemaKind::Request, "crate::schema::config::ConfigPatchParams", null_sample),
    ProtocolSchemaEntry::new("ConfigSchemaParams", ProtocolSchemaKind::Request, "crate::schema::config::ConfigSchemaParams", null_sample),
    ProtocolSchemaEntry::new("ConfigSchemaLookupParams", ProtocolSchemaKind::Request, "crate::schema::config::ConfigSchemaLookupParams", null_sample),
    ProtocolSchemaEntry::new("ConfigSchemaResponse", ProtocolSchemaKind::Result, "crate::schema::config::ConfigSchemaResponse", null_sample),
    ProtocolSchemaEntry::new("ConfigSchemaLookupResult", ProtocolSchemaKind::Result, "crate::schema::config::ConfigSchemaLookupResult", null_sample),
    ProtocolSchemaEntry::new("CrestodianChatParams", ProtocolSchemaKind::Request, "crate::schema::crestodian::CrestodianChatParams", null_sample),
    ProtocolSchemaEntry::new("CrestodianChatResult", ProtocolSchemaKind::Result, "crate::schema::crestodian::CrestodianChatResult", null_sample),
    ProtocolSchemaEntry::new("CrestodianSetupDetectParams", ProtocolSchemaKind::Request, "crate::schema::crestodian::CrestodianSetupDetectParams", null_sample),
    ProtocolSchemaEntry::new("CrestodianSetupDetectResult", ProtocolSchemaKind::Result, "crate::schema::crestodian::CrestodianSetupDetectResult", null_sample),
    ProtocolSchemaEntry::new("CrestodianSetupVerifyParams", ProtocolSchemaKind::Request, "crate::schema::crestodian::CrestodianSetupVerifyParams", null_sample),
    ProtocolSchemaEntry::new("CrestodianSetupVerifyResult", ProtocolSchemaKind::Result, "crate::schema::crestodian::CrestodianSetupVerifyResult", null_sample),
    ProtocolSchemaEntry::new("CrestodianSetupActivateParams", ProtocolSchemaKind::Request, "crate::schema::crestodian::CrestodianSetupActivateParams", null_sample),
    ProtocolSchemaEntry::new("CrestodianSetupActivateResult", ProtocolSchemaKind::Result, "crate::schema::crestodian::CrestodianSetupActivateResult", null_sample),
    ProtocolSchemaEntry::new("CrestodianSetupAuthStartParams", ProtocolSchemaKind::Request, "crate::schema::crestodian::CrestodianSetupAuthStartParams", null_sample),
    ProtocolSchemaEntry::new("CrestodianSetupAuthStartResult", ProtocolSchemaKind::Result, "crate::schema::crestodian::CrestodianSetupAuthStartResult", null_sample),
    ProtocolSchemaEntry::new("WizardStartParams", ProtocolSchemaKind::Request, "crate::schema::wizard::WizardStartParams", null_sample),
    ProtocolSchemaEntry::new("WizardNextParams", ProtocolSchemaKind::Request, "crate::schema::wizard::WizardNextParams", null_sample),
    ProtocolSchemaEntry::new("WizardCancelParams", ProtocolSchemaKind::Request, "crate::schema::wizard::WizardCancelParams", null_sample),
    ProtocolSchemaEntry::new("WizardStatusParams", ProtocolSchemaKind::Request, "crate::schema::wizard::WizardStatusParams", null_sample),
    ProtocolSchemaEntry::new("WizardStep", ProtocolSchemaKind::Other, "crate::schema::wizard::WizardStep", null_sample),
    ProtocolSchemaEntry::new("WizardNextResult", ProtocolSchemaKind::Result, "crate::schema::wizard::WizardNextResult", null_sample),
    ProtocolSchemaEntry::new("WizardStartResult", ProtocolSchemaKind::Result, "crate::schema::wizard::WizardStartResult", null_sample),
    ProtocolSchemaEntry::new("WizardStatusResult", ProtocolSchemaKind::Result, "crate::schema::wizard::WizardStatusResult", null_sample),

    // --- Realtime Talk client/session events and channel control payloads. ---
    ProtocolSchemaEntry::new("TalkModeParams", ProtocolSchemaKind::Request, "crate::schema::channels::TalkModeParams", null_sample),
    ProtocolSchemaEntry::new("TalkEvent", ProtocolSchemaKind::Event, "crate::schema::channels::TalkEvent", null_sample),
    ProtocolSchemaEntry::new("TalkCatalogParams", ProtocolSchemaKind::Request, "crate::schema::channels::TalkCatalogParams", null_sample),
    ProtocolSchemaEntry::new("TalkCatalogResult", ProtocolSchemaKind::Result, "crate::schema::channels::TalkCatalogResult", null_sample),
    ProtocolSchemaEntry::new("TalkClientCreateParams", ProtocolSchemaKind::Request, "crate::schema::channels::TalkClientCreateParams", null_sample),
    ProtocolSchemaEntry::new("TalkClientCreateResult", ProtocolSchemaKind::Result, "crate::schema::channels::TalkClientCreateResult", null_sample),
    ProtocolSchemaEntry::new("TalkClientSteerParams", ProtocolSchemaKind::Request, "crate::schema::channels::TalkClientSteerParams", null_sample),
    ProtocolSchemaEntry::new("TalkAgentControlResult", ProtocolSchemaKind::Result, "crate::schema::channels::TalkAgentControlResult", null_sample),
    ProtocolSchemaEntry::new("TalkClientToolCallParams", ProtocolSchemaKind::Request, "crate::schema::channels::TalkClientToolCallParams", null_sample),
    ProtocolSchemaEntry::new("TalkClientToolCallResult", ProtocolSchemaKind::Result, "crate::schema::channels::TalkClientToolCallResult", null_sample),
    ProtocolSchemaEntry::new("TalkConfigParams", ProtocolSchemaKind::Request, "crate::schema::channels::TalkConfigParams", null_sample),
    ProtocolSchemaEntry::new("TalkConfigResult", ProtocolSchemaKind::Result, "crate::schema::channels::TalkConfigResult", null_sample),
    ProtocolSchemaEntry::new("TalkSessionAppendAudioParams", ProtocolSchemaKind::Request, "crate::schema::channels::TalkSessionAppendAudioParams", null_sample),
    ProtocolSchemaEntry::new("TalkSessionCancelOutputParams", ProtocolSchemaKind::Request, "crate::schema::channels::TalkSessionCancelOutputParams", null_sample),
    ProtocolSchemaEntry::new("TalkSessionCancelTurnParams", ProtocolSchemaKind::Request, "crate::schema::channels::TalkSessionCancelTurnParams", null_sample),
    ProtocolSchemaEntry::new("TalkSessionCreateParams", ProtocolSchemaKind::Request, "crate::schema::channels::TalkSessionCreateParams", null_sample),
    ProtocolSchemaEntry::new("TalkSessionCreateResult", ProtocolSchemaKind::Result, "crate::schema::channels::TalkSessionCreateResult", null_sample),
    ProtocolSchemaEntry::new("TalkSessionJoinParams", ProtocolSchemaKind::Request, "crate::schema::channels::TalkSessionJoinParams", null_sample),
    ProtocolSchemaEntry::new("TalkSessionJoinResult", ProtocolSchemaKind::Result, "crate::schema::channels::TalkSessionJoinResult", null_sample),
    ProtocolSchemaEntry::new("TalkSessionTurnParams", ProtocolSchemaKind::Request, "crate::schema::channels::TalkSessionTurnParams", null_sample),
    ProtocolSchemaEntry::new("TalkSessionTurnResult", ProtocolSchemaKind::Result, "crate::schema::channels::TalkSessionTurnResult", null_sample),
    ProtocolSchemaEntry::new("TalkSessionSteerParams", ProtocolSchemaKind::Request, "crate::schema::channels::TalkSessionSteerParams", null_sample),
    ProtocolSchemaEntry::new("TalkSessionSubmitToolResultParams", ProtocolSchemaKind::Request, "crate::schema::channels::TalkSessionSubmitToolResultParams", null_sample),
    ProtocolSchemaEntry::new("TalkSessionCloseParams", ProtocolSchemaKind::Request, "crate::schema::channels::TalkSessionCloseParams", null_sample),
    ProtocolSchemaEntry::new("TalkSessionOkResult", ProtocolSchemaKind::Result, "crate::schema::channels::TalkSessionOkResult", null_sample),
    ProtocolSchemaEntry::new("TalkSpeakParams", ProtocolSchemaKind::Request, "crate::schema::channels::TalkSpeakParams", null_sample),
    ProtocolSchemaEntry::new("TalkSpeakResult", ProtocolSchemaKind::Result, "crate::schema::channels::TalkSpeakResult", null_sample),
    ProtocolSchemaEntry::new("TtsSpeakParams", ProtocolSchemaKind::Request, "crate::schema::channels::TtsSpeakParams", null_sample),
    ProtocolSchemaEntry::new("TtsSpeakResult", ProtocolSchemaKind::Result, "crate::schema::channels::TtsSpeakResult", null_sample),
    ProtocolSchemaEntry::new("ChannelsStatusParams", ProtocolSchemaKind::Request, "crate::schema::channels::ChannelsStatusParams", null_sample),
    ProtocolSchemaEntry::new("ChannelsStatusResult", ProtocolSchemaKind::Result, "crate::schema::channels::ChannelsStatusResult", null_sample),
    ProtocolSchemaEntry::new("ChannelsStartParams", ProtocolSchemaKind::Request, "crate::schema::channels::ChannelsStartParams", null_sample),
    ProtocolSchemaEntry::new("ChannelsStopParams", ProtocolSchemaKind::Request, "crate::schema::channels::ChannelsStopParams", null_sample),
    ProtocolSchemaEntry::new("ChannelsLogoutParams", ProtocolSchemaKind::Request, "crate::schema::channels::ChannelsLogoutParams", null_sample),
    ProtocolSchemaEntry::new("WebLoginStartParams", ProtocolSchemaKind::Request, "crate::schema::channels::WebLoginStartParams", null_sample),
    ProtocolSchemaEntry::new("WebLoginWaitParams", ProtocolSchemaKind::Request, "crate::schema::channels::WebLoginWaitParams", null_sample),

    // --- Agent files, artifacts, model catalogs, commands, tools, and skill workshop. ---
    // (These types live in agents-models-skills.rs; not yet exposed via type aliases.)
    ProtocolSchemaEntry::new("AgentSummary", ProtocolSchemaKind::Result, "crate::schema::agents_models_skills::AgentSummarySchema", null_sample),
    ProtocolSchemaEntry::new("AgentsCreateParams", ProtocolSchemaKind::Request, "crate::schema::agents_models_skills::AgentsCreateParamsSchema", null_sample),
    ProtocolSchemaEntry::new("AgentsCreateResult", ProtocolSchemaKind::Result, "crate::schema::agents_models_skills::AgentsCreateResultSchema", null_sample),
    ProtocolSchemaEntry::new("AgentsUpdateParams", ProtocolSchemaKind::Request, "crate::schema::agents_models_skills::AgentsUpdateParamsSchema", null_sample),
    ProtocolSchemaEntry::new("AgentsUpdateResult", ProtocolSchemaKind::Result, "crate::schema::agents_models_skills::AgentsUpdateResultSchema", null_sample),
    ProtocolSchemaEntry::new("AgentsDeleteParams", ProtocolSchemaKind::Request, "crate::schema::agents_models_skills::AgentsDeleteParamsSchema", null_sample),
    ProtocolSchemaEntry::new("AgentsDeleteResult", ProtocolSchemaKind::Result, "crate::schema::agents_models_skills::AgentsDeleteResultSchema", null_sample),
    ProtocolSchemaEntry::new("AgentsFileEntry", ProtocolSchemaKind::Other, "crate::schema::agents_models_skills::AgentsFileEntrySchema", null_sample),
    ProtocolSchemaEntry::new("AgentsFilesListParams", ProtocolSchemaKind::Request, "crate::schema::agents_models_skills::AgentsFilesListParamsSchema", null_sample),
    ProtocolSchemaEntry::new("AgentsFilesListResult", ProtocolSchemaKind::Result, "crate::schema::agents_models_skills::AgentsFilesListResultSchema", null_sample),
    ProtocolSchemaEntry::new("AgentsFilesGetParams", ProtocolSchemaKind::Request, "crate::schema::agents_models_skills::AgentsFilesGetParamsSchema", null_sample),
    ProtocolSchemaEntry::new("AgentsFilesGetResult", ProtocolSchemaKind::Result, "crate::schema::agents_models_skills::AgentsFilesGetResultSchema", null_sample),
    ProtocolSchemaEntry::new("AgentsFilesSetParams", ProtocolSchemaKind::Request, "crate::schema::agents_models_skills::AgentsFilesSetParamsSchema", null_sample),
    ProtocolSchemaEntry::new("AgentsFilesSetResult", ProtocolSchemaKind::Result, "crate::schema::agents_models_skills::AgentsFilesSetResultSchema", null_sample),
    ProtocolSchemaEntry::new("AgentsWorkspaceEntry", ProtocolSchemaKind::Other, "crate::schema::agents_workspace::AgentsWorkspaceEntry", null_sample),
    ProtocolSchemaEntry::new("AgentsWorkspaceFile", ProtocolSchemaKind::Other, "crate::schema::agents_workspace::AgentsWorkspaceFile", null_sample),
    ProtocolSchemaEntry::new("AgentsWorkspaceListParams", ProtocolSchemaKind::Request, "crate::schema::agents_workspace::AgentsWorkspaceListParams", null_sample),
    ProtocolSchemaEntry::new("AgentsWorkspaceListResult", ProtocolSchemaKind::Result, "crate::schema::agents_workspace::AgentsWorkspaceListResult", null_sample),
    ProtocolSchemaEntry::new("AgentsWorkspaceGetParams", ProtocolSchemaKind::Request, "crate::schema::agents_workspace::AgentsWorkspaceGetParams", null_sample),
    ProtocolSchemaEntry::new("AgentsWorkspaceGetResult", ProtocolSchemaKind::Result, "crate::schema::agents_workspace::AgentsWorkspaceGetResult", null_sample),
    ProtocolSchemaEntry::new("ArtifactSummary", ProtocolSchemaKind::Result, "crate::schema::artifacts::ArtifactSummary", null_sample),
    ProtocolSchemaEntry::new("ArtifactsListParams", ProtocolSchemaKind::Request, "crate::schema::artifacts::ArtifactsListParams", null_sample),
    ProtocolSchemaEntry::new("ArtifactsListResult", ProtocolSchemaKind::Result, "crate::schema::artifacts::ArtifactsListResult", null_sample),
    ProtocolSchemaEntry::new("ArtifactsGetParams", ProtocolSchemaKind::Request, "crate::schema::artifacts::ArtifactsGetParams", null_sample),
    ProtocolSchemaEntry::new("ArtifactsGetResult", ProtocolSchemaKind::Result, "crate::schema::artifacts::ArtifactsGetResult", null_sample),
    ProtocolSchemaEntry::new("ArtifactsDownloadParams", ProtocolSchemaKind::Request, "crate::schema::artifacts::ArtifactsDownloadParams", null_sample),
    ProtocolSchemaEntry::new("ArtifactsDownloadResult", ProtocolSchemaKind::Result, "crate::schema::artifacts::ArtifactsDownloadResult", null_sample),
    ProtocolSchemaEntry::new("AgentsListParams", ProtocolSchemaKind::Request, "crate::schema::agents_models_skills::AgentsListParamsSchema", null_sample),
    ProtocolSchemaEntry::new("AgentsListResult", ProtocolSchemaKind::Result, "crate::schema::agents_models_skills::AgentsListResultSchema", null_sample),
    ProtocolSchemaEntry::new("ModelChoice", ProtocolSchemaKind::Other, "crate::schema::agents_models_skills::ModelChoiceSchema", null_sample),
    ProtocolSchemaEntry::new("ModelsListParams", ProtocolSchemaKind::Request, "crate::schema::agents_models_skills::ModelsListParamsSchema", null_sample),
    ProtocolSchemaEntry::new("ModelsListResult", ProtocolSchemaKind::Result, "crate::schema::agents_models_skills::ModelsListResultSchema", null_sample),
    ProtocolSchemaEntry::new("CommandEntry", ProtocolSchemaKind::Other, "crate::schema::commands::CommandEntry", null_sample),
    ProtocolSchemaEntry::new("CommandsListParams", ProtocolSchemaKind::Request, "crate::schema::commands::CommandsListParams", null_sample),
    ProtocolSchemaEntry::new("CommandsListResult", ProtocolSchemaKind::Result, "crate::schema::commands::CommandsListResult", null_sample),
    ProtocolSchemaEntry::new("SkillsStatusParams", ProtocolSchemaKind::Request, "crate::schema::agents_models_skills::SkillsStatusParamsSchema", null_sample),
    ProtocolSchemaEntry::new("ToolsCatalogParams", ProtocolSchemaKind::Request, "crate::schema::agents_models_skills::ToolsCatalogParamsSchema", null_sample),
    ProtocolSchemaEntry::new("ToolCatalogProfile", ProtocolSchemaKind::Other, "crate::schema::agents_models_skills::ToolCatalogProfileSchema", null_sample),
    ProtocolSchemaEntry::new("ToolCatalogEntry", ProtocolSchemaKind::Other, "crate::schema::agents_models_skills::ToolCatalogEntrySchema", null_sample),
    ProtocolSchemaEntry::new("ToolCatalogGroup", ProtocolSchemaKind::Other, "crate::schema::agents_models_skills::ToolCatalogGroupSchema", null_sample),
    ProtocolSchemaEntry::new("ToolsCatalogResult", ProtocolSchemaKind::Result, "crate::schema::agents_models_skills::ToolsCatalogResultSchema", null_sample),
    ProtocolSchemaEntry::new("ToolsEffectiveParams", ProtocolSchemaKind::Request, "crate::schema::agents_models_skills::ToolsEffectiveParamsSchema", null_sample),
    ProtocolSchemaEntry::new("ToolsEffectiveEntry", ProtocolSchemaKind::Other, "crate::schema::agents_models_skills::ToolsEffectiveEntrySchema", null_sample),
    ProtocolSchemaEntry::new("ToolsEffectiveGroup", ProtocolSchemaKind::Other, "crate::schema::agents_models_skills::ToolsEffectiveGroupSchema", null_sample),
    ProtocolSchemaEntry::new("ToolsEffectiveNotice", ProtocolSchemaKind::Other, "crate::schema::agents_models_skills::ToolsEffectiveNoticeSchema", null_sample),
    ProtocolSchemaEntry::new("ToolsEffectiveResult", ProtocolSchemaKind::Result, "crate::schema::agents_models_skills::ToolsEffectiveResultSchema", null_sample),
    ProtocolSchemaEntry::new("ToolsInvokeParams", ProtocolSchemaKind::Request, "crate::schema::agents_models_skills::ToolsInvokeParamsSchema", null_sample),
    ProtocolSchemaEntry::new("ToolsInvokeError", ProtocolSchemaKind::Other, "crate::schema::agents_models_skills::ToolsInvokeErrorSchema", null_sample),
    ProtocolSchemaEntry::new("ToolsInvokeResult", ProtocolSchemaKind::Result, "crate::schema::agents_models_skills::ToolsInvokeResultSchema", null_sample),
    ProtocolSchemaEntry::new("SkillsBinsParams", ProtocolSchemaKind::Request, "crate::schema::agents_models_skills::SkillsBinsParamsSchema", null_sample),
    ProtocolSchemaEntry::new("SkillsBinsResult", ProtocolSchemaKind::Result, "crate::schema::agents_models_skills::SkillsBinsResultSchema", null_sample),
    ProtocolSchemaEntry::new("SkillsSearchParams", ProtocolSchemaKind::Request, "crate::schema::agents_models_skills::SkillsSearchParamsSchema", null_sample),
    ProtocolSchemaEntry::new("SkillsSearchResult", ProtocolSchemaKind::Result, "crate::schema::agents_models_skills::SkillsSearchResultSchema", null_sample),
    ProtocolSchemaEntry::new("SkillsDetailParams", ProtocolSchemaKind::Request, "crate::schema::agents_models_skills::SkillsDetailParamsSchema", null_sample),
    ProtocolSchemaEntry::new("SkillsDetailResult", ProtocolSchemaKind::Result, "crate::schema::agents_models_skills::SkillsDetailResultSchema", null_sample),
    ProtocolSchemaEntry::new("SkillsCuratorActionParams", ProtocolSchemaKind::Request, "crate::schema::agents_models_skills::SkillsCuratorActionParamsSchema", null_sample),
    ProtocolSchemaEntry::new("SkillsCuratorActionResult", ProtocolSchemaKind::Result, "crate::schema::agents_models_skills::SkillsCuratorActionResultSchema", null_sample),
    ProtocolSchemaEntry::new("SkillsCuratorStatusParams", ProtocolSchemaKind::Request, "crate::schema::agents_models_skills::SkillsCuratorStatusParamsSchema", null_sample),
    ProtocolSchemaEntry::new("SkillsCuratorStatusResult", ProtocolSchemaKind::Result, "crate::schema::agents_models_skills::SkillsCuratorStatusResultSchema", null_sample),
    ProtocolSchemaEntry::new("SkillsProposalsListParams", ProtocolSchemaKind::Request, "crate::schema::agents_models_skills::SkillsProposalsListParamsSchema", null_sample),
    ProtocolSchemaEntry::new("SkillsProposalsListResult", ProtocolSchemaKind::Result, "crate::schema::agents_models_skills::SkillsProposalsListResultSchema", null_sample),
    ProtocolSchemaEntry::new("SkillsProposalInspectParams", ProtocolSchemaKind::Request, "crate::schema::agents_models_skills::SkillsProposalInspectParamsSchema", null_sample),
    ProtocolSchemaEntry::new("SkillsProposalInspectResult", ProtocolSchemaKind::Result, "crate::schema::agents_models_skills::SkillsProposalInspectResultSchema", null_sample),
    ProtocolSchemaEntry::new("SkillsProposalCreateParams", ProtocolSchemaKind::Request, "crate::schema::agents_models_skills::SkillsProposalCreateParamsSchema", null_sample),
    ProtocolSchemaEntry::new("SkillsProposalUpdateParams", ProtocolSchemaKind::Request, "crate::schema::agents_models_skills::SkillsProposalUpdateParamsSchema", null_sample),
    ProtocolSchemaEntry::new("SkillsProposalReviseParams", ProtocolSchemaKind::Request, "crate::schema::agents_models_skills::SkillsProposalReviseParamsSchema", null_sample),
    ProtocolSchemaEntry::new("SkillsProposalRequestRevisionParams", ProtocolSchemaKind::Request, "crate::schema::agents_models_skills::SkillsProposalRequestRevisionParamsSchema", null_sample),
    ProtocolSchemaEntry::new("SkillsProposalRequestRevisionResult", ProtocolSchemaKind::Result, "crate::schema::agents_models_skills::SkillsProposalRequestRevisionResultSchema", null_sample),
    ProtocolSchemaEntry::new("SkillsProposalActionParams", ProtocolSchemaKind::Request, "crate::schema::agents_models_skills::SkillsProposalActionParamsSchema", null_sample),
    ProtocolSchemaEntry::new("SkillsProposalApplyResult", ProtocolSchemaKind::Result, "crate::schema::agents_models_skills::SkillsProposalApplyResultSchema", null_sample),
    ProtocolSchemaEntry::new("SkillsProposalRecordResult", ProtocolSchemaKind::Result, "crate::schema::agents_models_skills::SkillsProposalRecordResultSchema", null_sample),
    ProtocolSchemaEntry::new("SkillsSecurityVerdictsParams", ProtocolSchemaKind::Request, "crate::schema::agents_models_skills::SkillsSecurityVerdictsParamsSchema", null_sample),
    ProtocolSchemaEntry::new("SkillsSecurityVerdictsResult", ProtocolSchemaKind::Result, "crate::schema::agents_models_skills::SkillsSecurityVerdictsResultSchema", null_sample),
    ProtocolSchemaEntry::new("SkillsSkillCardParams", ProtocolSchemaKind::Request, "crate::schema::agents_models_skills::SkillsSkillCardParamsSchema", null_sample),
    ProtocolSchemaEntry::new("SkillsSkillCardResult", ProtocolSchemaKind::Result, "crate::schema::agents_models_skills::SkillsSkillCardResultSchema", null_sample),
    ProtocolSchemaEntry::new("SkillsUploadBeginParams", ProtocolSchemaKind::Request, "crate::schema::agents_models_skills::SkillsUploadBeginParamsSchema", null_sample),
    ProtocolSchemaEntry::new("SkillsUploadChunkParams", ProtocolSchemaKind::Request, "crate::schema::agents_models_skills::SkillsUploadChunkParamsSchema", null_sample),
    ProtocolSchemaEntry::new("SkillsUploadCommitParams", ProtocolSchemaKind::Request, "crate::schema::agents_models_skills::SkillsUploadCommitParamsSchema", null_sample),
    ProtocolSchemaEntry::new("SkillsInstallParams", ProtocolSchemaKind::Request, "crate::schema::agents_models_skills::SkillsInstallParamsSchema", null_sample),
    ProtocolSchemaEntry::new("SkillsUpdateParams", ProtocolSchemaKind::Request, "crate::schema::agents_models_skills::SkillsUpdateParamsSchema", null_sample),

    // --- Scheduler, logs, approval, plugin control, device, chat, and lifecycle events. ---
    ProtocolSchemaEntry::new("CronJob", ProtocolSchemaKind::Other, "crate::schema::cron::CronJobSchema", null_sample),
    ProtocolSchemaEntry::new("CronListParams", ProtocolSchemaKind::Request, "crate::schema::cron::CronListParamsSchema", null_sample),
    ProtocolSchemaEntry::new("CronStatusParams", ProtocolSchemaKind::Request, "crate::schema::cron::CronStatusParamsSchema", null_sample),
    ProtocolSchemaEntry::new("CronGetParams", ProtocolSchemaKind::Request, "crate::schema::cron::CronGetParamsSchema", null_sample),
    ProtocolSchemaEntry::new("CronAddParams", ProtocolSchemaKind::Request, "crate::schema::cron::CronAddParams", null_sample),
    ProtocolSchemaEntry::new("CronAddResult", ProtocolSchemaKind::Result, "crate::schema::cron::CronAddResult", null_sample),
    ProtocolSchemaEntry::new("CronDeclarativeAddResult", ProtocolSchemaKind::Result, "crate::schema::cron::CronDeclarativeAddResult", null_sample),
    ProtocolSchemaEntry::new("CronUpdateParams", ProtocolSchemaKind::Request, "crate::schema::cron::CronUpdateParamsSchema", null_sample),
    ProtocolSchemaEntry::new("CronRemoveParams", ProtocolSchemaKind::Request, "crate::schema::cron::CronRemoveParams", null_sample),
    ProtocolSchemaEntry::new("CronRunParams", ProtocolSchemaKind::Request, "crate::schema::cron::CronRunParamsSchema", null_sample),
    ProtocolSchemaEntry::new("CronRunsParams", ProtocolSchemaKind::Request, "crate::schema::cron::CronRunsParamsSchema", null_sample),
    ProtocolSchemaEntry::new("CronRunLogEntry", ProtocolSchemaKind::Other, "crate::schema::cron::CronRunLogEntrySchema", null_sample),
    ProtocolSchemaEntry::new("LogsTailParams", ProtocolSchemaKind::Request, "crate::schema::logs_chat::LogsTailParams", null_sample),
    ProtocolSchemaEntry::new("LogsTailResult", ProtocolSchemaKind::Result, "crate::schema::logs_chat::LogsTailResult", null_sample),
    ProtocolSchemaEntry::new("TerminalOpenParams", ProtocolSchemaKind::Request, "crate::schema::terminal::TerminalOpenParams", null_sample),
    ProtocolSchemaEntry::new("TerminalOpenResult", ProtocolSchemaKind::Result, "crate::schema::terminal::TerminalOpenResult", null_sample),
    ProtocolSchemaEntry::new("TerminalInputParams", ProtocolSchemaKind::Request, "crate::schema::terminal::TerminalInputParams", null_sample),
    ProtocolSchemaEntry::new("TerminalResizeParams", ProtocolSchemaKind::Request, "crate::schema::terminal::TerminalResizeParams", null_sample),
    ProtocolSchemaEntry::new("TerminalCloseParams", ProtocolSchemaKind::Request, "crate::schema::terminal::TerminalCloseParams", null_sample),
    ProtocolSchemaEntry::new("TerminalAttachParams", ProtocolSchemaKind::Request, "crate::schema::terminal::TerminalAttachParams", null_sample),
    ProtocolSchemaEntry::new("TerminalAttachResult", ProtocolSchemaKind::Result, "crate::schema::terminal::TerminalAttachResult", null_sample),
    ProtocolSchemaEntry::new("TerminalSessionInfo", ProtocolSchemaKind::Other, "crate::schema::terminal::TerminalSessionInfo", null_sample),
    ProtocolSchemaEntry::new("TerminalListResult", ProtocolSchemaKind::Result, "crate::schema::terminal::TerminalListResult", null_sample),
    ProtocolSchemaEntry::new("TerminalTextParams", ProtocolSchemaKind::Request, "crate::schema::terminal::TerminalTextParams", null_sample),
    ProtocolSchemaEntry::new("TerminalTextResult", ProtocolSchemaKind::Result, "crate::schema::terminal::TerminalTextResult", null_sample),
    ProtocolSchemaEntry::new("TerminalAckResult", ProtocolSchemaKind::Result, "crate::schema::terminal::TerminalAckResult", null_sample),
    ProtocolSchemaEntry::new("TerminalDataEvent", ProtocolSchemaKind::Event, "crate::schema::terminal::TerminalDataEvent", null_sample),
    ProtocolSchemaEntry::new("TerminalExitEvent", ProtocolSchemaKind::Event, "crate::schema::terminal::TerminalExitEvent", null_sample),
    ProtocolSchemaEntry::new("TerminalEvent", ProtocolSchemaKind::Event, "crate::schema::terminal::TerminalEvent", null_sample),
    ProtocolSchemaEntry::new("ApprovalKind", ProtocolSchemaKind::Other, "crate::schema::approvals::ApprovalKind", null_sample),
    ProtocolSchemaEntry::new("ApprovalDecision", ProtocolSchemaKind::Other, "crate::schema::approvals::ApprovalDecision", null_sample),
    ProtocolSchemaEntry::new("ApprovalAllowDecision", ProtocolSchemaKind::Other, "crate::schema::approvals::ApprovalAllowDecision", null_sample),
    ProtocolSchemaEntry::new("ApprovalAllowedReason", ProtocolSchemaKind::Other, "crate::schema::approvals::ApprovalAllowedReason", null_sample),
    ProtocolSchemaEntry::new("ApprovalDeniedReason", ProtocolSchemaKind::Other, "crate::schema::approvals::ApprovalDeniedReason", null_sample),
    ProtocolSchemaEntry::new("ApprovalExpiredReason", ProtocolSchemaKind::Other, "crate::schema::approvals::ApprovalExpiredReason", null_sample),
    ProtocolSchemaEntry::new("ApprovalCancelledReason", ProtocolSchemaKind::Other, "crate::schema::approvals::ApprovalCancelledReason", null_sample),
    ProtocolSchemaEntry::new("PluginApprovalSeverity", ProtocolSchemaKind::Other, "crate::schema::approvals::PluginApprovalSeverity", null_sample),
    ProtocolSchemaEntry::new("ExecApprovalPresentation", ProtocolSchemaKind::Other, "crate::schema::approvals::ExecApprovalPresentationType", null_sample),
    ProtocolSchemaEntry::new("PluginApprovalPresentation", ProtocolSchemaKind::Other, "crate::schema::approvals::PluginApprovalPresentationType", null_sample),
    ProtocolSchemaEntry::new("ApprovalPresentation", ProtocolSchemaKind::Other, "crate::schema::approvals::ApprovalPresentationType", null_sample),
    ProtocolSchemaEntry::new("PendingApprovalSnapshot", ProtocolSchemaKind::Other, "crate::schema::approvals::PendingApprovalSnapshotType", null_sample),
    ProtocolSchemaEntry::new("AllowedApprovalSnapshot", ProtocolSchemaKind::Other, "crate::schema::approvals::AllowedApprovalSnapshotType", null_sample),
    ProtocolSchemaEntry::new("DeniedApprovalSnapshot", ProtocolSchemaKind::Other, "crate::schema::approvals::DeniedApprovalSnapshotType", null_sample),
    ProtocolSchemaEntry::new("ExpiredApprovalSnapshot", ProtocolSchemaKind::Other, "crate::schema::approvals::ExpiredApprovalSnapshotType", null_sample),
    ProtocolSchemaEntry::new("CancelledApprovalSnapshot", ProtocolSchemaKind::Other, "crate::schema::approvals::CancelledApprovalSnapshotType", null_sample),
    ProtocolSchemaEntry::new("ApprovalSnapshot", ProtocolSchemaKind::Other, "crate::schema::approvals::ApprovalSnapshotType", null_sample),
    ProtocolSchemaEntry::new("ApprovalTerminalReason", ProtocolSchemaKind::Other, "crate::schema::approvals::ApprovalTerminalReason", null_sample),
    ProtocolSchemaEntry::new("TerminalApprovalSnapshot", ProtocolSchemaKind::Other, "crate::schema::approvals::TerminalApprovalSnapshotType", null_sample),
    ProtocolSchemaEntry::new("ApprovalGetParams", ProtocolSchemaKind::Request, "crate::schema::approvals::ApprovalGetParamsType", null_sample),
    ProtocolSchemaEntry::new("ApprovalGetResult", ProtocolSchemaKind::Result, "crate::schema::approvals::ApprovalGetResultType", null_sample),
    ProtocolSchemaEntry::new("ApprovalResolveParams", ProtocolSchemaKind::Request, "crate::schema::approvals::ApprovalResolveParamsType", null_sample),
    ProtocolSchemaEntry::new("ApprovalResolveResult", ProtocolSchemaKind::Result, "crate::schema::approvals::ApprovalResolveResultType", null_sample),
    ProtocolSchemaEntry::new("PendingSessionApprovalEvent", ProtocolSchemaKind::Event, "crate::schema::approvals::PendingSessionApprovalEventSchema", null_sample),
    ProtocolSchemaEntry::new("TerminalSessionApprovalEvent", ProtocolSchemaKind::Event, "crate::schema::approvals::TerminalSessionApprovalEventSchema", null_sample),
    ProtocolSchemaEntry::new("SessionApprovalEvent", ProtocolSchemaKind::Event, "crate::schema::approvals::SessionApprovalEventType", null_sample),
    ProtocolSchemaEntry::new("SessionApprovalReplay", ProtocolSchemaKind::Event, "crate::schema::approvals::SessionApprovalReplayType", null_sample),
    ProtocolSchemaEntry::new("ExecApprovalsGetParams", ProtocolSchemaKind::Request, "crate::schema::exec_approvals::ExecApprovalsGetParamsSchema", null_sample),
    ProtocolSchemaEntry::new("ExecApprovalsSetParams", ProtocolSchemaKind::Request, "crate::schema::exec_approvals::ExecApprovalsSetParamsSchema", null_sample),
    ProtocolSchemaEntry::new("ExecApprovalsNodeGetParams", ProtocolSchemaKind::Request, "crate::schema::exec_approvals::ExecApprovalsNodeGetParamsSchema", null_sample),
    ProtocolSchemaEntry::new("ExecApprovalsNodeSnapshot", ProtocolSchemaKind::Other, "crate::schema::exec_approvals::ExecApprovalsNodeSnapshotSchema", null_sample),
    ProtocolSchemaEntry::new("ExecApprovalsNodeSetParams", ProtocolSchemaKind::Request, "crate::schema::exec_approvals::ExecApprovalsNodeSetParamsSchema", null_sample),
    ProtocolSchemaEntry::new("ExecApprovalsSnapshot", ProtocolSchemaKind::Other, "crate::schema::exec_approvals::ExecApprovalsSnapshotSchema", null_sample),
    ProtocolSchemaEntry::new("ExecApprovalGetParams", ProtocolSchemaKind::Request, "crate::schema::exec_approvals::ExecApprovalGetParamsSchema", null_sample),
    ProtocolSchemaEntry::new("ExecApprovalRequestParams", ProtocolSchemaKind::Request, "crate::schema::exec_approvals::ExecApprovalRequestParamsSchema", null_sample),
    ProtocolSchemaEntry::new("ExecApprovalResolveParams", ProtocolSchemaKind::Request, "crate::schema::exec_approvals::ExecApprovalResolveParamsSchema", null_sample),
    ProtocolSchemaEntry::new("PluginApprovalRequestParams", ProtocolSchemaKind::Request, "crate::schema::plugin_approvals::PluginApprovalRequestParams", null_sample),
    ProtocolSchemaEntry::new("PluginApprovalResolveParams", ProtocolSchemaKind::Request, "crate::schema::plugin_approvals::PluginApprovalResolveParams", null_sample),
    // (Plugins and Devices modules do not expose non-`Schema` type aliases yet;
    //  the schema names are still registered so consumers can find them.)
    ProtocolSchemaEntry::new("PluginCatalogClawHubInstall", ProtocolSchemaKind::Other, "crate::schema::plugins::PluginCatalogClawHubInstallSchema", null_sample),
    ProtocolSchemaEntry::new("PluginCatalogEntry", ProtocolSchemaKind::Other, "crate::schema::plugins::PluginCatalogEntrySchema", null_sample),
    ProtocolSchemaEntry::new("PluginCatalogInstallAction", ProtocolSchemaKind::Other, "crate::schema::plugins::PluginCatalogInstallActionSchema", null_sample),
    ProtocolSchemaEntry::new("PluginCatalogOfficialInstall", ProtocolSchemaKind::Other, "crate::schema::plugins::PluginCatalogOfficialInstallSchema", null_sample),
    ProtocolSchemaEntry::new("PluginControlUiDescriptor", ProtocolSchemaKind::Other, "crate::schema::plugins::PluginControlUiDescriptorSchema", null_sample),
    ProtocolSchemaEntry::new("PluginSearchPackage", ProtocolSchemaKind::Other, "crate::schema::plugins::PluginSearchPackageSchema", null_sample),
    ProtocolSchemaEntry::new("PluginSearchResultEntry", ProtocolSchemaKind::Other, "crate::schema::plugins::PluginSearchResultEntrySchema", null_sample),
    ProtocolSchemaEntry::new("PluginsInstallParams", ProtocolSchemaKind::Request, "crate::schema::plugins::PluginsInstallParamsSchema", null_sample),
    ProtocolSchemaEntry::new("PluginsInstallResult", ProtocolSchemaKind::Result, "crate::schema::plugins::PluginsInstallResultSchema", null_sample),
    ProtocolSchemaEntry::new("PluginsListParams", ProtocolSchemaKind::Request, "crate::schema::plugins::PluginsListParamsSchema", null_sample),
    ProtocolSchemaEntry::new("PluginsListResult", ProtocolSchemaKind::Result, "crate::schema::plugins::PluginsListResultSchema", null_sample),
    ProtocolSchemaEntry::new("PluginsSearchParams", ProtocolSchemaKind::Request, "crate::schema::plugins::PluginsSearchParamsSchema", null_sample),
    ProtocolSchemaEntry::new("PluginsSearchResult", ProtocolSchemaKind::Result, "crate::schema::plugins::PluginsSearchResultSchema", null_sample),
    ProtocolSchemaEntry::new("PluginsSessionActionFailureResult", ProtocolSchemaKind::Result, "crate::schema::plugins::PluginsSessionActionFailureResultSchema", null_sample),
    ProtocolSchemaEntry::new("PluginsSessionActionParams", ProtocolSchemaKind::Request, "crate::schema::plugins::PluginsSessionActionParamsSchema", null_sample),
    ProtocolSchemaEntry::new("PluginsSessionActionResult", ProtocolSchemaKind::Result, "crate::schema::plugins::PluginsSessionActionResultSchema", null_sample),
    ProtocolSchemaEntry::new("PluginsSessionActionSuccessResult", ProtocolSchemaKind::Result, "crate::schema::plugins::PluginsSessionActionSuccessResultSchema", null_sample),
    ProtocolSchemaEntry::new("PluginsSetEnabledParams", ProtocolSchemaKind::Request, "crate::schema::plugins::PluginsSetEnabledParamsSchema", null_sample),
    ProtocolSchemaEntry::new("PluginsSetEnabledResult", ProtocolSchemaKind::Result, "crate::schema::plugins::PluginsSetEnabledResultSchema", null_sample),
    ProtocolSchemaEntry::new("PluginsUiDescriptorsParams", ProtocolSchemaKind::Request, "crate::schema::plugins::PluginsUiDescriptorsParamsSchema", null_sample),
    ProtocolSchemaEntry::new("PluginsUiDescriptorsResult", ProtocolSchemaKind::Result, "crate::schema::plugins::PluginsUiDescriptorsResultSchema", null_sample),
    ProtocolSchemaEntry::new("PluginsUninstallParams", ProtocolSchemaKind::Request, "crate::schema::plugins::PluginsUninstallParamsSchema", null_sample),
    ProtocolSchemaEntry::new("PluginsUninstallResult", ProtocolSchemaKind::Result, "crate::schema::plugins::PluginsUninstallResultSchema", null_sample),
    ProtocolSchemaEntry::new("DevicePairListParams", ProtocolSchemaKind::Request, "crate::schema::devices::DevicePairListParamsSchema", null_sample),
    ProtocolSchemaEntry::new("DevicePairApproveParams", ProtocolSchemaKind::Request, "crate::schema::devices::DevicePairApproveParamsSchema", null_sample),
    ProtocolSchemaEntry::new("DevicePairRejectParams", ProtocolSchemaKind::Request, "crate::schema::devices::DevicePairRejectParamsSchema", null_sample),
    ProtocolSchemaEntry::new("DevicePairRemoveParams", ProtocolSchemaKind::Request, "crate::schema::devices::DevicePairRemoveParamsSchema", null_sample),
    ProtocolSchemaEntry::new("DevicePairSetupCodeParams", ProtocolSchemaKind::Request, "crate::schema::devices::DevicePairSetupCodeParamsSchema", null_sample),
    ProtocolSchemaEntry::new("DevicePairSetupCodeResult", ProtocolSchemaKind::Result, "crate::schema::devices::DevicePairSetupCodeResultSchema", null_sample),
    ProtocolSchemaEntry::new("DevicePairRenameParams", ProtocolSchemaKind::Request, "crate::schema::devices::DevicePairRenameParamsSchema", null_sample),
    ProtocolSchemaEntry::new("DeviceTokenRotateParams", ProtocolSchemaKind::Request, "crate::schema::devices::DeviceTokenRotateParamsSchema", null_sample),
    ProtocolSchemaEntry::new("DeviceTokenRevokeParams", ProtocolSchemaKind::Request, "crate::schema::devices::DeviceTokenRevokeParamsSchema", null_sample),
    ProtocolSchemaEntry::new("DevicePairRequestedEvent", ProtocolSchemaKind::Event, "crate::schema::devices::DevicePairRequestedEventSchema", null_sample),
    ProtocolSchemaEntry::new("DevicePairResolvedEvent", ProtocolSchemaKind::Event, "crate::schema::devices::DevicePairResolvedEventSchema", null_sample),
    ProtocolSchemaEntry::new("ChatHistoryParams", ProtocolSchemaKind::Request, "crate::schema::logs_chat::ChatHistoryParamsSchema", null_sample),
    ProtocolSchemaEntry::new("ChatMetadataParams", ProtocolSchemaKind::Request, "crate::schema::logs_chat::ChatMetadataParams", null_sample),
    ProtocolSchemaEntry::new("ChatMessageGetParams", ProtocolSchemaKind::Request, "crate::schema::logs_chat::ChatMessageGetParamsSchema", null_sample),
    ProtocolSchemaEntry::new("ChatMessageGetResult", ProtocolSchemaKind::Result, "crate::schema::logs_chat::ChatMessageGetResultSchema", null_sample),
    ProtocolSchemaEntry::new("ChatToolTitlesParams", ProtocolSchemaKind::Request, "crate::schema::logs_chat::ChatToolTitlesParams", null_sample),
    ProtocolSchemaEntry::new("ChatToolTitlesResult", ProtocolSchemaKind::Result, "crate::schema::logs_chat::ChatToolTitlesResultSchema", null_sample),
    ProtocolSchemaEntry::new("ChatSendParams", ProtocolSchemaKind::Request, "crate::schema::logs_chat::ChatSendParamsSchema", null_sample),
    ProtocolSchemaEntry::new("ChatAbortParams", ProtocolSchemaKind::Request, "crate::schema::logs_chat::ChatAbortParams", null_sample),
    ProtocolSchemaEntry::new("ChatInjectParams", ProtocolSchemaKind::Request, "crate::schema::logs_chat::ChatInjectParams", null_sample),
    ProtocolSchemaEntry::new("ChatDeltaEvent", ProtocolSchemaKind::Event, "crate::schema::logs_chat::ChatDeltaEventSchema", null_sample),
    ProtocolSchemaEntry::new("ChatFinalEvent", ProtocolSchemaKind::Event, "crate::schema::logs_chat::ChatFinalEventSchema", null_sample),
    ProtocolSchemaEntry::new("ChatAbortedEvent", ProtocolSchemaKind::Event, "crate::schema::logs_chat::ChatAbortedEventSchema", null_sample),
    ProtocolSchemaEntry::new("ChatErrorEvent", ProtocolSchemaKind::Event, "crate::schema::logs_chat::ChatErrorEventSchema", null_sample),
    ProtocolSchemaEntry::new("ChatEvent", ProtocolSchemaKind::Event, "crate::schema::logs_chat::ChatEvent", null_sample),
    ProtocolSchemaEntry::new("UpdateStatusParams", ProtocolSchemaKind::Request, "crate::schema::config::UpdateStatusParams", null_sample),
    ProtocolSchemaEntry::new("UpdateRunParams", ProtocolSchemaKind::Request, "crate::schema::config::UpdateRunParams", null_sample),
    ProtocolSchemaEntry::new("TickEvent", ProtocolSchemaKind::Event, "crate::frames::TickEvent", null_sample),
    ProtocolSchemaEntry::new("ShutdownEvent", ProtocolSchemaKind::Event, "crate::frames::ShutdownEvent", null_sample),
];

// =============================================================================
// Version constants (re-exported from `version` module).
// =============================================================================

pub use crate::version::{
    MIN_CLIENT_PROTOCOL_VERSION, MIN_NODE_PROTOCOL_VERSION, MIN_PROBE_PROTOCOL_VERSION, PROTOCOL_VERSION,
};
