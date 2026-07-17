//! OpenAI reasoning-effort normalization.
//! 翻译自 packages/ai/src/providers/openai-reasoning-effort.ts
//!
//! Different GPT families expose different accepted effort enums, so
//! callers map requested values here before constructing provider payloads.

use std::collections::BTreeSet;

use once_cell::sync::Lazy;
use regex::Regex;
use serde_json::Value;

pub type OpenAIReasoningEffort = String;
pub type OpenAIApiReasoningEffort = String;

pub const GPT_5_REASONING_EFFORTS: &[&str] = &["minimal", "low", "medium", "high"];
pub const GPT_51_REASONING_EFFORTS: &[&str] = &["none", "low", "medium", "high"];
pub const GPT_52_REASONING_EFFORTS: &[&str] = &["none", "low", "medium", "high", "xhigh"];
pub const GPT_56_REASONING_EFFORTS: &[&str] = &["none", "low", "medium", "high", "xhigh", "max"];
pub const GPT_CODEX_REASONING_EFFORTS: &[&str] = &["low", "medium", "high", "xhigh"];
pub const GPT_PRO_REASONING_EFFORTS: &[&str] = &["medium", "high", "xhigh"];
pub const GPT_5_PRO_REASONING_EFFORTS: &[&str] = &["high"];
pub const GPT_51_CODEX_MAX_REASONING_EFFORTS: &[&str] = &["none", "medium", "high", "xhigh"];
pub const GPT_51_CODEX_MINI_REASONING_EFFORTS: &[&str] = &["medium"];
pub const GENERIC_REASONING_EFFORTS: &[&str] = &["low", "medium", "high"];

pub static CANONICAL_REASONING_EFFORTS: Lazy<BTreeSet<&'static str>> = Lazy::new(|| {
    BTreeSet::from([
        "none", "minimal", "low", "medium", "high", "xhigh", "max", "off",
    ])
});

fn normalize_model_id(id: Option<&str>) -> String {
    let raw = id.unwrap_or("").trim().to_lowercase();
    static DATE_SUFFIX: Lazy<Regex> =
        Lazy::new(|| Regex::new(r"-\d{4}-\d{2}-\d{2}$").unwrap());
    DATE_SUFFIX.replace(&raw, "").into_owned()
}

/// Return whether a model is the GPT-5.4 mini family.
pub fn is_openai_gpt_5_4_mini_model(model_id: Option<&str>) -> bool {
    let id = normalize_model_id(model_id);
    static RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"^gpt-5\.4-mini(?:-|$)").unwrap());
    RE.is_match(&id)
}

/// Return whether a model is the GPT-5.5 family.
pub fn is_openai_gpt_5_5_model(model_id: Option<&str>, model_name: Option<&str>) -> bool {
    let id = normalize_model_id(model_id);
    let name = normalize_model_id(model_name);
    static RE_ID: Lazy<Regex> = Lazy::new(|| Regex::new(r"^gpt-5\.5(?:-|$)").unwrap());
    static RE_NAME: Lazy<Regex> = Lazy::new(|| Regex::new(r"^gpt-5\.5(?:\s|\(|-|$)").unwrap());
    RE_ID.is_match(&id) || RE_NAME.is_match(&name)
}

/// Return whether a model is the GPT-5.6 family.
pub fn is_openai_gpt_5_6_model(model_id: Option<&str>, model_name: Option<&str>) -> bool {
    let id = normalize_model_id(model_id);
    let name = normalize_model_id(model_name);
    static RE_ID: Lazy<Regex> = Lazy::new(|| Regex::new(r"^gpt-5\.6(?:-|$)").unwrap());
    static RE_NAME: Lazy<Regex> = Lazy::new(|| Regex::new(r"^gpt-5\.6(?:\s|\(|-|$)").unwrap());
    RE_ID.is_match(&id) || RE_NAME.is_match(&name)
}

/// Normalize user-facing reasoning effort names to API effort names.
pub fn normalize_openai_reasoning_effort(effort: &str) -> String {
    let trimmed = effort.trim();
    let folded = trimmed.to_lowercase();
    match folded.as_str() {
        "auto" | "default" => "medium".to_string(),
        "off" | "none" | "disable" | "disabled" => "none".to_string(),
        "minimal" => "minimal".to_string(),
        "low" => "low".to_string(),
        "medium" => "medium".to_string(),
        "high" => "high".to_string(),
        "xhigh" | "extra-high" | "extra_high" => "xhigh".to_string(),
        "max" | "maximum" => "max".to_string(),
        _ => folded,
    }
}

/// Resolve the API effort string for a given model id and requested effort.
pub fn resolve_openai_api_reasoning_effort(
    model_id: Option<&str>,
    requested: Option<&str>,
) -> Option<String> {
    let req = requested?;
    let normalized = normalize_openai_reasoning_effort(req);
    let id = normalize_model_id(model_id);
    if id.starts_with("gpt-5.6") || id.starts_with("gpt-5_6") {
        if GPT_56_REASONING_EFFORTS.contains(&normalized.as_str()) {
            return Some(normalized);
        }
    }
    if id.starts_with("gpt-5.2") {
        if GPT_52_REASONING_EFFORTS.contains(&normalized.as_str()) {
            return Some(normalized);
        }
    }
    if id.starts_with("gpt-5.1-codex-max") {
        if GPT_51_CODEX_MAX_REASONING_EFFORTS.contains(&normalized.as_str()) {
            return Some(normalized);
        }
    }
    if id.starts_with("gpt-5.1-codex-mini") {
        if GPT_51_CODEX_MINI_REASONING_EFFORTS.contains(&normalized.as_str()) {
            return Some(normalized);
        }
    }
    if id.starts_with("gpt-5.1") || id.starts_with("gpt-5_1") {
        if GPT_51_REASONING_EFFORTS.contains(&normalized.as_str()) {
            return Some(normalized);
        }
    }
    if id.starts_with("gpt-5-codex") {
        if GPT_CODEX_REASONING_EFFORTS.contains(&normalized.as_str()) {
            return Some(normalized);
        }
    }
    if id.starts_with("gpt-5-pro") || id == "gpt-5-pro" {
        if GPT_5_PRO_REASONING_EFFORTS.contains(&normalized.as_str()) {
            return Some(normalized);
        }
    }
    if id.starts_with("gpt-5") || id.starts_with("gpt-5.") {
        if GPT_5_REASONING_EFFORTS.contains(&normalized.as_str()) {
            return Some(normalized);
        }
    }
    if GPT_PRO_REASONING_EFFORTS.contains(&normalized.as_str()) {
        return Some(normalized);
    }
    if GENERIC_REASONING_EFFORTS.contains(&normalized.as_str()) {
        return Some(normalized);
    }
    None
}

/// Convenience: produce a JSON payload fragment that callers can drop into
/// the provider request body.
pub fn reasoning_effort_payload(model_id: Option<&str>, requested: Option<&str>) -> Option<Value> {
    resolve_openai_api_reasoning_effort(model_id, requested).map(|v| serde_json::json!({ "effort": v }))
}