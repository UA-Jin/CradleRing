// ACP Core module implements session identifiers behavior.
// 翻译自 packages/acp-core/src/runtime/session-identifiers.ts

use std::collections::HashMap;

use normalization_core::string_coerce;
use serde_json::Value;

use crate::normalize_text;
use crate::runtime::session_identity::{
    is_session_identity_pending, resolve_session_identity_from_meta,
};
use crate::types::{SessionAcpIdentity, SessionAcpMeta};

pub const ACP_SESSION_IDENTITY_RENDERER_VERSION: &str = "v1";
pub type AcpSessionIdentifierRenderMode = String;

type SessionResumeHintResolver = Box<dyn Fn(&str) -> String + Send + Sync>;

fn build_hint_map() -> HashMap<String, SessionResumeHintResolver> {
    let mut m: HashMap<String, SessionResumeHintResolver> = HashMap::new();
    let codex_factory: fn(&str) -> String = |agent_session_id: &str| {
        format!(
            "resume in Codex CLI: `codex resume {}` (continues this conversation).",
            agent_session_id
        )
    };
    m.insert("codex".to_string(), Box::new(codex_factory));
    m.insert("openai".to_string(), Box::new(codex_factory));
    m.insert("codex-cli".to_string(), Box::new(codex_factory));
    let kimi_factory: fn(&str) -> String = |agent_session_id: &str| {
        format!(
            "resume in Kimi CLI: `kimi resume {}` (continues this conversation).",
            agent_session_id
        )
    };
    m.insert("kimi".to_string(), Box::new(kimi_factory));
    m.insert("moonshot-kimi".to_string(), Box::new(kimi_factory));
    m
}

fn acp_agent_resume_hint_by_key() -> &'static HashMap<String, SessionResumeHintResolver> {
    use std::sync::OnceLock;
    static MAP: OnceLock<HashMap<String, SessionResumeHintResolver>> = OnceLock::new();
    MAP.get_or_init(build_hint_map)
}

fn normalize_agent_hint_key(value: Option<&str>) -> Option<String> {
    let normalized = normalize_text::normalize_text_opt(value)?;
    let lowered = string_coerce::normalize_lowercase_string_or_empty(&Value::String(normalized));
    let replaced = lowered.replace([' ', '_'], "-");
    if replaced.is_empty() {
        None
    } else {
        Some(replaced)
    }
}

fn resolve_acp_agent_resume_hint_line(
    agent_id: Option<&str>,
    agent_session_id: Option<&str>,
) -> Option<String> {
    let agent_session_id = normalize_text::normalize_text_opt(agent_session_id)?;
    let agent_key = normalize_agent_hint_key(agent_id)?;
    let resolver = acp_agent_resume_hint_by_key().get(&agent_key)?;
    Some(resolver(&agent_session_id))
}

/// Renders resolved ACP backend/agent ids, hiding pending ids from thread intros.
pub fn resolve_acp_session_identifier_lines_from_identity(params: ResolveAcpSessionIdentifierLinesParams) -> Vec<String> {
    let backend = normalize_text::normalize_text_opt(Some(&params.backend))
        .unwrap_or_else(|| "backend".to_string());
    let mode = params.mode.unwrap_or_else(|| "status".to_string());
    let identity = params.identity;
    let agent_session_id = normalize_text::normalize_text_opt(
        identity.as_ref().and_then(|i| i.agent_session_id.as_deref()),
    );
    let acpx_session_id = normalize_text::normalize_text_opt(
        identity.as_ref().and_then(|i| i.acpx_session_id.as_deref()),
    );
    let acpx_record_id = normalize_text::normalize_text_opt(
        identity.as_ref().and_then(|i| i.acpx_record_id.as_deref()),
    );
    let has_identifier =
        agent_session_id.is_some() || acpx_session_id.is_some() || acpx_record_id.is_some();
    if is_session_identity_pending(identity.as_ref()) && has_identifier {
        // Status views explain that ids are still settling; thread intros stay quiet so
        // users do not copy provisional backend ids before the first reply resolves them.
        if mode == "status" {
            return vec!["session ids: pending (available after the first reply)".to_string()];
        }
        return vec![];
    }
    let mut lines: Vec<String> = vec![];
    if let Some(id) = agent_session_id {
        lines.push(format!("agent session id: {}", id));
    }
    if let Some(id) = &acpx_session_id {
        lines.push(format!("{} session id: {}", backend, id));
    }
    if let Some(id) = &acpx_record_id {
        lines.push(format!("{} record id: {}", backend, id));
    }
    lines
}

pub struct ResolveAcpSessionIdentifierLinesParams {
    pub backend: String,
    pub identity: Option<SessionAcpIdentity>,
    pub mode: Option<AcpSessionIdentifierRenderMode>,
}

/// Resolves the runtime cwd, preferring modern runtimeOptions over legacy metadata.
pub fn resolve_acp_session_cwd(meta: Option<&SessionAcpMeta>) -> Option<String> {
    let meta = meta?;
    if let Some(cwd) = meta
        .runtime_options
        .as_ref()
        .and_then(|o| normalize_text::normalize_text_opt(o.cwd.as_deref()))
    {
        return Some(cwd);
    }
    normalize_text::normalize_text_opt(meta.cwd.as_deref())
}

/// Renders thread-detail identifier lines plus a backend-specific resume hint when stable.
pub fn resolve_acp_thread_session_detail_lines(params: ThreadSessionDetailParams) -> Vec<String> {
    let meta = params.meta.as_ref();
    let identity = resolve_session_identity_from_meta(meta);
    let backend = meta
        .and_then(|m| normalize_text::normalize_text_opt(Some(&m.backend)))
        .unwrap_or_else(|| "backend".to_string());
    let mut lines = resolve_acp_session_identifier_lines_from_identity(
        ResolveAcpSessionIdentifierLinesParams {
            backend: backend.clone(),
            identity: identity.clone(),
            mode: Some("thread".to_string()),
        },
    );
    if lines.is_empty() {
        return lines;
    }
    let hint = resolve_acp_agent_resume_hint_line(
        meta.map(|m| m.agent.as_str()),
        identity.as_ref().and_then(|i| i.agent_session_id.as_deref()),
    );
    if let Some(h) = hint {
        lines.push(h);
    }
    lines
}

pub struct ThreadSessionDetailParams {
    pub session_key: String,
    pub meta: Option<SessionAcpMeta>,
}