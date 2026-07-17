// Gateway Protocol schema module: protocol validation shapes.
// 翻译自 packages/gateway-protocol/src/schema/* (barrel)

pub mod agent;
#[path = "agents-models-skills.rs"]
pub mod agents_models_skills;
pub mod agents_workspace;
pub mod approval_id;
pub mod approvals;
pub mod artifacts;
pub mod audit;
#[path = "audit-activity.rs"]
pub mod audit_activity;
pub mod commands;
pub mod channels;
pub mod config;
pub mod crestodian;
pub mod cron;
pub mod devices;
#[path = "error-codes.rs"]
pub mod error_codes;
pub mod environments;
// #[path = "exec-approvals.rs"] -- broken self-ref
// pub mod exec_approvals; -- broken
#[path = "exec-approvals.rs"]
pub mod exec_approvals;
pub mod fs;
pub mod gateway_suspend;
pub mod logs_chat;
pub mod nodes;
pub mod plugin_approvals;
pub mod plugins;
pub mod primitives;
pub mod protocol_schemas;
pub mod push;
pub mod secrets;
pub mod sessions;
#[path = "sessions-catalog.rs"]
pub mod sessions_catalog;
pub mod snapshot;
pub mod system_info;
pub mod tasks;
#[path = "task-suggestions.rs"]
pub mod task_suggestions;
pub mod terminal;
pub mod wizard;
#[path = "worker-admission.rs"]
pub mod worker_admission;
pub mod worktrees;
