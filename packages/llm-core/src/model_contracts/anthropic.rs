// Claude model identity resolvers.
// 翻译自 packages/llm-core/src/model-contracts/anthropic.ts

use std::collections::BTreeMap;

use regex::Regex;

use crate::utils::diagnostics::{append_assistant_message_diagnostic, AssistantMessageDiagnostic};

/// Claude Fable 5 thinking profile levels.
pub const CLAUDE_FABLE_5_LEVELS: &[&str] = &[
    "off", "minimal", "low", "medium", "high", "xhigh", "adaptive", "max",
];
pub const CLAUDE_FABLE_5_DEFAULT_LEVEL: &str = "high";
pub const CLAUDE_FABLE_5_PRESERVE_WHEN_CATALOG_REASONING_FALSE: bool = true;

/// Claude Sonnet 5 thinking profile levels.
pub const CLAUDE_SONNET_5_LEVELS: &[&str] = &[
    "off", "minimal", "low", "medium", "high", "xhigh", "adaptive", "max",
];
pub const CLAUDE_SONNET_5_DEFAULT_LEVEL: &str = "high";
pub const CLAUDE_SONNET_5_PRESERVE_WHEN_CATALOG_REASONING_FALSE: bool = false;

/// Reference shape required by the resolvers.
#[derive(Debug, Clone, Default)]
pub struct ClaudeModelRef {
    pub id: Option<String>,
    pub params: Option<BTreeMap<String, serde_json::Value>>,
    pub thinking_level_map: Option<BTreeMap<String, Option<String>>>,
}

fn normalize_claude_model_id(model_id: Option<&str>) -> String {
    let normalized = model_id.map(|s| s.trim().to_lowercase()).unwrap_or_default();
    let unprefixed = normalized
        .strip_prefix("anthropic/")
        .unwrap_or(&normalized);
    let re = Regex::new(r"[._\s]+").unwrap();
    re.replace_all(unprefixed, "-").into_owned()
}

/// Resolve the canonical normalized Claude model id for one runtime model ref.
pub fn resolve_claude_model_identity(ref_: &ClaudeModelRef) -> String {
    let configured_canonical = ref_
        .params
        .as_ref()
        .and_then(|p| p.get("canonicalModelId"))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    let normalized = normalize_claude_model_id(
        configured_canonical
            .as_deref()
            .or(ref_.id.as_deref()),
    );
    let re = Regex::new(r"(?:^|[-/])claude-").unwrap();
    if let Some(mat) = re.find(&normalized) {
        let strip = if mat.as_str().starts_with("claude-") {
            0
        } else {
            1
        };
        normalized[mat.start() + strip..].to_string()
    } else {
        normalized
    }
}

/// Test whether the character at `idx` (if any) is a non-alphanumeric boundary.
fn is_boundary(s: &str, idx: usize) -> bool {
    match s[idx..].chars().next() {
        Some(c) => !c.is_ascii_alphanumeric(),
        None => true,
    }
}

/// Find a match for the variant pattern. The pattern uses a placeholder
/// `(?:^|-)` for the variant leading separator; this fn performs the
/// equivalent of `(?=$|[^a-z0-9])` by trimming trailing alphanumeric chars.
fn match_claude_variant_inner(ref_: &ClaudeModelRef, pattern: &str, variant: &str) -> Option<String> {
    let normalized = resolve_claude_model_identity(ref_);
    let re = Regex::new(pattern).ok()?;
    let mat = re.find(&normalized)?;
    let end = mat.end();
    // Boundary: end of string OR next char not in [a-z0-9]
    if end < normalized.len() && !is_boundary(&normalized, end) {
        return None;
    }
    let strip = if mat.as_str().starts_with('-') { 1 } else { 0 };
    // Strip the variant literal from the slice.
    let slice = &normalized[mat.start() + strip..];
    if let Some(idx) = slice.find(variant) {
        let abs = mat.start() + strip + idx + variant.len();
        if abs < normalized.len() && !is_boundary(&normalized, abs) {
            return None;
        }
    }
    Some(normalized[mat.start() + strip..].to_string())
}

/// Resolve Claude Fable 5 through direct ids, cloud ids, or deployment metadata.
pub fn resolve_claude_fable_5_model_identity(ref_: &ClaudeModelRef) -> Option<String> {
    match_claude_variant_inner(ref_, r"(?:^|-)claude-fable-5", "claude-fable-5")
}

/// Resolve Claude Mythos 5 through direct ids, cloud ids, or deployment metadata.
pub fn resolve_claude_mythos_5_model_identity(ref_: &ClaudeModelRef) -> Option<String> {
    match_claude_variant_inner(ref_, r"(?:^|-)claude-mythos-5", "claude-mythos-5")
}

fn matches_variant(ref_: &ClaudeModelRef, variants: &[&str]) -> bool {
    let model_id = resolve_claude_model_identity(ref_);
    for variant in variants {
        if match_claude_variant_inner(ref_, &format!(r"(?:^|-){}", variant), variant).is_some() {
            return true;
        }
    }
    // Suppress unused warnings on model_id (kept for debugging/trace parity).
    let _ = model_id;
    false
}

/// Return whether a Claude model requires adaptive thinking instead of manual budgets.
pub fn requires_claude_mandatory_adaptive_thinking(ref_: &ClaudeModelRef) -> bool {
    resolve_claude_fable_5_model_identity(ref_).is_some()
        || resolve_claude_mythos_5_model_identity(ref_).is_some()
        || matches_variant(ref_, &["claude-mythos-preview"])
}

/// Resolve Claude Sonnet 5 through direct ids, cloud ids, or deployment metadata.
pub fn resolve_claude_sonnet_5_model_identity(ref_: &ClaudeModelRef) -> Option<String> {
    match_claude_variant_inner(ref_, r"(?:^|-)claude-sonnet-5", "claude-sonnet-5")
}

/// Return whether a Claude model supports adaptive thinking.
pub fn supports_claude_adaptive_thinking(ref_: &ClaudeModelRef) -> bool {
    matches_variant(
        ref_,
        &[
            "claude-fable-5",
            "claude-mythos-5",
            "claude-mythos-preview",
            "claude-opus-4-6",
            "claude-opus-4-7",
            "claude-opus-4-8",
            "claude-sonnet-5",
            "claude-sonnet-4-6",
        ],
    )
}

/// Return whether a Claude model supports native max effort.
pub fn supports_claude_native_max_effort(ref_: &ClaudeModelRef) -> bool {
    matches_variant(
        ref_,
        &[
            "claude-fable-5",
            "claude-mythos-5",
            "claude-opus-4-6",
            "claude-opus-4-7",
            "claude-opus-4-8",
            "claude-sonnet-5",
            "claude-sonnet-4-6",
        ],
    )
}

/// Return whether a Claude model supports native xhigh effort.
pub fn supports_claude_native_xhigh_effort(ref_: &ClaudeModelRef) -> bool {
    matches_variant(
        ref_,
        &[
            "claude-fable-5",
            "claude-mythos-5",
            "claude-opus-4-7",
            "claude-opus-4-8",
            "claude-sonnet-5",
        ],
    )
}

/// Return whether a Claude model rejects caller-selected sampling parameters.
pub fn requires_claude_default_sampling(ref_: &ClaudeModelRef) -> bool {
    supports_claude_native_xhigh_effort(ref_)
        || matches_variant(ref_, &["claude-mythos-preview"])
}

/// Fill native Claude effort mappings only when the provider did not publish a
/// narrower route-specific contract.
pub fn resolve_claude_native_thinking_level_map(
    ref_: &ClaudeModelRef,
) -> Option<BTreeMap<String, Option<String>>> {
    if let Some(map) = ref_.thinking_level_map.clone() {
        return Some(map);
    }
    if !supports_claude_native_max_effort(ref_) {
        return None;
    }
    let mut map: BTreeMap<String, Option<String>> = BTreeMap::new();
    map.insert(
        "xhigh".to_string(),
        if supports_claude_native_xhigh_effort(ref_) {
            Some("xhigh".to_string())
        } else {
            None
        },
    );
    map.insert("max".to_string(), Some("max".to_string()));
    Some(map)
}

// Re-export diagnostic helper for downstream modules that want consistent
// `appendAssistantMessageDiagnostic` semantics.
#[allow(dead_code)]
pub(crate) fn append_diagnostic(
    diagnostics: &mut Option<Vec<AssistantMessageDiagnostic>>,
    diagnostic: AssistantMessageDiagnostic,
) {
    append_assistant_message_diagnostic(diagnostics, diagnostic);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ref_with_id(id: &str) -> ClaudeModelRef {
        ClaudeModelRef {
            id: Some(id.to_string()),
            params: None,
            thinking_level_map: None,
        }
    }

    #[test]
    fn strips_anthropic_prefix_and_separators() {
        let id = resolve_claude_model_identity(&ref_with_id("Anthropic.Claude_Sonnet 4.6"));
        assert_eq!(id, "claude-sonnet-4-6");
    }

    #[test]
    fn detects_fable_5() {
        assert!(resolve_claude_fable_5_model_identity(&ref_with_id("claude-fable-5")).is_some());
        assert!(resolve_claude_fable_5_model_identity(&ref_with_id("anthropic/claude-fable-5")).is_some());
    }

    #[test]
    fn xhigh_only_for_fable_5_and_newer() {
        assert!(supports_claude_native_xhigh_effort(&ref_with_id("claude-fable-5")));
        assert!(supports_claude_native_xhigh_effort(&ref_with_id("claude-opus-4-7")));
        assert!(!supports_claude_native_xhigh_effort(&ref_with_id("claude-sonnet-4-6")));
    }
}