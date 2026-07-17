// CradleRing SDK helper module supports normalize behavior.
// 翻译自 packages/sdk/src/normalize.ts

use std::time::{SystemTime, UNIX_EPOCH};

use serde_json::Value;

use crate::types::{GatewayEvent, JsonObject, OpenClawEvent, OpenClawEventType};

fn as_record(value: Option<Value>) -> JsonObject {
    match value {
        Some(Value::Object(m)) => m,
        _ => JsonObject::new(),
    }
}

fn as_value_record(value: Value) -> JsonObject {
    match value {
        Value::Object(m) => m,
        _ => JsonObject::new(),
    }
}

fn read_string(value: &Value) -> Option<String> {
    if let Value::String(s) = value {
        if !s.is_empty() {
            return Some(s.clone());
        }
    }
    None
}

fn read_number(value: &Value) -> Option<f64> {
    if let Value::Number(n) = value {
        n.as_f64().filter(|v| v.is_finite())
    } else {
        None
    }
}

fn read_lower_string(value: &Value) -> Option<String> {
    read_string(value).map(|s| s.to_lowercase())
}

fn read_bool(value: &Value) -> Option<bool> {
    if let Value::Bool(b) = value {
        Some(*b)
    } else {
        None
    }
}

fn has_hard_timeout_metadata(data: &JsonObject, status_already_timeout_attributed: bool) -> bool {
    let timeout_phase = data
        .get("timeoutPhase")
        .and_then(read_lower_string)
        .unwrap_or_default();
    (status_already_timeout_attributed && data.get("providerStarted").and_then(read_bool) == Some(true))
        || timeout_phase == "preflight"
        || timeout_phase == "provider"
        || timeout_phase == "post_turn"
}

fn is_lifecycle_cancellation(data: &JsonObject) -> bool {
    let status = data.get("status").and_then(read_lower_string).unwrap_or_default();
    let stop_reason = data
        .get("stopReason")
        .and_then(read_lower_string)
        .unwrap_or_default();
    status == "aborted"
        || status == "cancelled"
        || status == "canceled"
        || status == "killed"
        || stop_reason == "aborted"
        || stop_reason == "cancelled"
        || stop_reason == "canceled"
        || stop_reason == "killed"
        || stop_reason == "auth-revoked"
        || stop_reason == "restart"
        || stop_reason == "rpc"
        || stop_reason == "user"
        || (data.get("aborted").and_then(read_bool) == Some(true) && stop_reason == "stop")
}

fn normalize_lifecycle_end_event_type(data: &JsonObject) -> OpenClawEventType {
    let status = data.get("status").and_then(read_lower_string).unwrap_or_default();
    let stop_reason = data
        .get("stopReason")
        .and_then(read_lower_string)
        .unwrap_or_default();
    let status_already_timeout_attributed = stop_reason != "restart"
        && (status == "timeout"
            || status == "timed_out"
            || data.get("aborted").and_then(read_bool) == Some(true));
    if has_hard_timeout_metadata(data, status_already_timeout_attributed) {
        return OpenClawEventType::RunTimedOut;
    }
    if is_lifecycle_cancellation(data) {
        return OpenClawEventType::RunCancelled;
    }
    if status == "timeout"
        || status == "timed_out"
        || stop_reason == "timeout"
        || stop_reason == "timed_out"
    {
        return OpenClawEventType::RunTimedOut;
    }
    if data.get("aborted").and_then(read_bool) == Some(true) {
        return OpenClawEventType::RunTimedOut;
    }
    OpenClawEventType::RunCompleted
}

fn normalize_agent_event_type(payload: &JsonObject) -> OpenClawEventType {
    let stream = payload.get("stream").and_then(read_string).unwrap_or_default();
    let data = payload
        .get("data")
        .and_then(|v| Some(as_value_record(v.clone())))
        .unwrap_or_default();
    let phase = data.get("phase").and_then(read_string).unwrap_or_default();
    let status = data.get("status").and_then(read_string).unwrap_or_default();
    let _ = stream.clone();
    let _ = phase.clone();
    let _ = status.clone();

    if stream == "assistant" {
        let delta = data.get("delta");
        let is_delta = match delta {
            Some(Value::Bool(b)) => *b,
            Some(Value::String(_)) => true,
            _ => false,
        };
        return if is_delta {
            OpenClawEventType::AssistantDelta
        } else {
            OpenClawEventType::AssistantMessage
        };
    }
    if stream == "thinking" || stream == "plan" {
        return OpenClawEventType::ThinkingDelta;
    }
    if stream == "lifecycle" {
        if phase == "start" {
            return OpenClawEventType::RunStarted;
        }
        if phase == "end" {
            return normalize_lifecycle_end_event_type(&data);
        }
        if phase == "error" {
            if has_hard_timeout_metadata(&data, false) {
                return OpenClawEventType::RunTimedOut;
            }
            if is_lifecycle_cancellation(&data) {
                return OpenClawEventType::RunCancelled;
            }
            return OpenClawEventType::RunFailed;
        }
    }
    if stream == "tool" || stream == "item" || stream == "command_output" {
        if phase == "start" || status == "running" {
            return OpenClawEventType::ToolCallStarted;
        }
        if phase == "delta" || phase == "update" {
            return OpenClawEventType::ToolCallDelta;
        }
        if status == "failed" || status == "blocked" {
            return OpenClawEventType::ToolCallFailed;
        }
        if phase == "end" || status == "completed" {
            return OpenClawEventType::ToolCallCompleted;
        }
        return OpenClawEventType::ToolCallDelta;
    }
    if stream == "approval" {
        return if phase == "resolved" {
            OpenClawEventType::ApprovalResolved
        } else {
            OpenClawEventType::ApprovalRequested
        };
    }
    if stream == "patch" {
        return OpenClawEventType::ArtifactUpdated;
    }
    if stream == "error" {
        return OpenClawEventType::RunFailed;
    }
    OpenClawEventType::Raw
}

fn normalize_named_event_type(event: &GatewayEvent) -> OpenClawEventType {
    let payload = event.payload.clone().map(as_value_record).unwrap_or_default();
    match event.event.as_str() {
        "agent" => normalize_agent_event_type(&payload),
        "sessions.changed" => {
            let reason = payload.get("reason").and_then(read_string).unwrap_or_default();
            if reason == "create" {
                OpenClawEventType::SessionCreated
            } else if reason == "compact" {
                OpenClawEventType::SessionCompacted
            } else {
                OpenClawEventType::SessionUpdated
            }
        }
        "session.message" => OpenClawEventType::AssistantMessage,
        "session.tool" => OpenClawEventType::ToolCallDelta,
        "exec.approval.requested" | "plugin.approval.requested" => OpenClawEventType::ApprovalRequested,
        "exec.approval.resolved" | "plugin.approval.resolved" => OpenClawEventType::ApprovalResolved,
        "task.updated" | "tasks.changed" => OpenClawEventType::TaskUpdated,
        _ => OpenClawEventType::Raw,
    }
}

/// Normalize a raw Gateway event into the public SDK event shape.
pub fn normalize_gateway_event(event: &GatewayEvent) -> OpenClawEvent {
    let payload = event.payload.clone().map(as_value_record).unwrap_or_default();
    let run_id = payload.get("runId").and_then(read_string).unwrap_or_default();
    let session_id = payload.get("sessionId").and_then(read_string).unwrap_or_default();
    let session_key = payload.get("sessionKey").and_then(read_string).unwrap_or_default();
    let task_id = payload.get("taskId").and_then(read_string).unwrap_or_default();
    let agent_id = payload.get("agentId").and_then(read_string).unwrap_or_default();
    let ts = payload
        .get("ts")
        .and_then(read_number)
        .map(|v| v as u64)
        .unwrap_or_else(|| {
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_millis() as u64)
                .unwrap_or(0)
        });
    let seq_part = event
        .seq
        .map(|s| s.to_string())
        .unwrap_or_else(|| "local".to_string());
    let id_parts = vec![seq_part, event.event.clone(), run_id.clone(), session_key.clone(), ts.to_string()];
    let id = id_parts.join(":");

    let data = event
        .payload
        .clone()
        .and_then(|v| v.as_object().and_then(|m| m.get("data")).cloned())
        .or_else(|| event.payload.clone());

    OpenClawEvent {
        version: 1,
        id,
        ts,
        r#type: normalize_named_event_type(event),
        run_id: if run_id.is_empty() { None } else { Some(run_id) },
        session_id: if session_id.is_empty() { None } else { Some(session_id) },
        session_key: if session_key.is_empty() { None } else { Some(session_key) },
        task_id: if task_id.is_empty() { None } else { Some(task_id) },
        agent_id: if agent_id.is_empty() { None } else { Some(agent_id) },
        data,
        raw: Some(event.clone()),
    }
}

pub fn as_record_opt(value: Option<Value>) -> JsonObject {
    as_record(value)
}
