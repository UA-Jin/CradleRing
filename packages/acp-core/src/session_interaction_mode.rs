// ACP Core module implements session interaction mode behavior.
// 翻译自 packages/acp-core/src/session-interaction-mode.ts

use normalization_core::string_coerce;
use serde_json::Value;

#[derive(Debug, Clone)]
pub enum AcpSessionInteractionMode {
    Interactive,
    ParentOwnedBackground,
}

#[derive(Debug, Clone, Default)]
pub struct SessionInteractionEntry {
    pub spawned_by: Option<String>,
    pub parent_session_key: Option<String>,
    pub acp: Option<Value>,
}

fn resolve_acp_session_interaction_mode(
    entry: Option<&SessionInteractionEntry>,
) -> AcpSessionInteractionMode {
    // Parent-owned ACP sessions are background work delegated from another session.
    // They should report back through the parent task notifier instead of speaking directly
    // on the user-facing channel themselves.
    let entry = match entry {
        Some(e) => e,
        None => return AcpSessionInteractionMode::Interactive,
    };
    if entry.acp.is_none() {
        return AcpSessionInteractionMode::Interactive;
    }
    let spawned_value = match &entry.spawned_by {
        Some(s) => Value::String(s.clone()),
        None => Value::Null,
    };
    let parent_value = match &entry.parent_session_key {
        Some(s) => Value::String(s.clone()),
        None => Value::Null,
    };
    if string_coerce::normalize_optional_string(&spawned_value).is_some()
        || string_coerce::normalize_optional_string(&parent_value).is_some()
    {
        AcpSessionInteractionMode::ParentOwnedBackground
    } else {
        AcpSessionInteractionMode::Interactive
    }
}

/// Returns true for ACP sessions delegated from a parent session instead of user-facing chat.
pub fn is_parent_owned_background_acp_session(
    entry: Option<&SessionInteractionEntry>,
) -> bool {
    matches!(
        resolve_acp_session_interaction_mode(entry),
        AcpSessionInteractionMode::ParentOwnedBackground
    )
}

/// Returns true when `entry` is a parent-owned background ACP session AND the
/// given `requester_session_key` is the session that spawned/owns it.
pub fn is_requester_parent_of_background_acp_session(
    entry: Option<&SessionInteractionEntry>,
    requester_session_key: Option<&str>,
) -> bool {
    if !is_parent_owned_background_acp_session(entry) {
        return false;
    }
    let entry = match entry {
        Some(e) => e,
        None => return false,
    };
    let requester_value = match requester_session_key {
        Some(s) => Value::String(s.to_string()),
        None => Value::Null,
    };
    let requester = match string_coerce::normalize_optional_string(&requester_value) {
        Some(s) => s,
        None => return false,
    };
    let spawned_value = match &entry.spawned_by {
        Some(s) => Value::String(s.clone()),
        None => Value::Null,
    };
    let parent_value = match &entry.parent_session_key {
        Some(s) => Value::String(s.clone()),
        None => Value::Null,
    };
    let spawned_by = string_coerce::normalize_optional_string(&spawned_value);
    let parent_session_key = string_coerce::normalize_optional_string(&parent_value);
    requester == spawned_by.unwrap_or_default() || requester == parent_session_key.unwrap_or_default()
}