// Media Core module implements inbound path policy behavior.
// 翻译自 packages/media-core/src/inbound-path-policy.ts

use regex::Regex;
use std::sync::OnceLock;

const WILDCARD_SEGMENT: &str = "*";

fn windows_drive_abs_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"^[A-Za-z]:/").unwrap())
}

fn windows_drive_root_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"^[A-Za-z]:$").unwrap())
}

fn posix_normalize(value: &str) -> String {
    // Emulate path.posix.normalize: collapse ".", "..", and repeated slashes.
    let mut stack: Vec<&str> = Vec::new();
    let mut absolute = false;
    if value.starts_with('/') {
        absolute = true;
    }
    let drive_prefix: Option<String>;
    let body = if windows_drive_abs_re().is_match(value) {
        drive_prefix = Some(value[..2].to_string());
        Some(&value[2..])
    } else {
        drive_prefix = None;
        Some(value)
    };
    let body = body.unwrap_or("");
    let trimmed_body = body.trim_start_matches('/');
    for segment in trimmed_body.split('/') {
        if segment.is_empty() || segment == "." {
            continue;
        }
        if segment == ".." {
            stack.pop();
            continue;
        }
        stack.push(segment);
    }
    let mut result = String::new();
    if absolute {
        result.push('/');
    }
    if let Some(ref dp) = drive_prefix {
        result.push_str(dp);
    }
    result.push_str(&stack.join("/"));
    result
}

fn normalize_posix_absolute_path(value: &str) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() || trimmed.contains('\0') {
        return None;
    }
    // Compare all roots as POSIX-style absolute paths so channel configs can use
    // stable patterns even when a source reports Windows separators.
    let normalized = posix_normalize(&trimmed.replace('\\', "/"));
    let is_absolute = normalized.starts_with('/') || windows_drive_abs_re().is_match(&normalized);
    if !is_absolute || normalized == "/" {
        return None;
    }
    let without_trailing_slash = if normalized.ends_with('/') {
        normalized[..normalized.len() - 1].to_string()
    } else {
        normalized.clone()
    };
    if windows_drive_root_re().is_match(&without_trailing_slash) {
        return None;
    }
    Some(if windows_drive_abs_re().is_match(&without_trailing_slash) {
        without_trailing_slash.to_lowercase()
    } else {
        without_trailing_slash
    })
}

fn split_path_segments(value: &str) -> Vec<&str> {
    value.split('/').filter(|s| !s.is_empty()).collect()
}

fn matches_root_pattern(candidate_path: &str, root_pattern: &str) -> bool {
    let candidate_segments = split_path_segments(candidate_path);
    let root_segments = split_path_segments(root_pattern);
    if candidate_segments.len() < root_segments.len() {
        return false;
    }
    for idx in 0..root_segments.len() {
        let expected = root_segments[idx];
        let actual = candidate_segments[idx];
        if expected == WILDCARD_SEGMENT {
            continue;
        }
        if expected != actual {
            return false;
        }
    }
    true
}

/** Validates an absolute inbound root pattern with whole-segment wildcards only. */
pub fn is_valid_inbound_path_root_pattern(value: &str) -> bool {
    let normalized = match normalize_posix_absolute_path(value) {
        Some(n) => n,
        None => return false,
    };
    let segments = split_path_segments(&normalized);
    if segments.is_empty() {
        return false;
    }
    segments
        .iter()
        .all(|segment| *segment == WILDCARD_SEGMENT || !segment.contains('*'))
}

/** Normalizes configured inbound attachment roots, dropping invalid or duplicate patterns. */
pub fn normalize_inbound_path_roots(roots: Option<&[String]>) -> Vec<String> {
    let mut normalized: Vec<String> = Vec::new();
    let mut seen: std::collections::HashSet<String> = std::collections::HashSet::new();
    let empty_vec: Vec<String> = Vec::new();
    let roots_slice = roots.unwrap_or(&empty_vec);
    for root in roots_slice {
        if !is_valid_inbound_path_root_pattern(root) {
            continue;
        }
        let candidate = match normalize_posix_absolute_path(root) {
            Some(c) => c,
            None => continue,
        };
        if seen.contains(&candidate) {
            continue;
        }
        seen.insert(candidate.clone());
        normalized.push(candidate);
    }
    normalized
}

/** Merges inbound attachment root lists while preserving first-seen priority. */
pub fn merge_inbound_path_roots(
    roots_lists: &[Option<&Vec<String>>],
) -> Vec<String> {
    let mut merged: Vec<String> = Vec::new();
    let mut seen: std::collections::HashSet<String> = std::collections::HashSet::new();
    for roots_opt in roots_lists {
        let roots_slice: &[String] = match roots_opt {
            Some(r) => r.as_slice(),
            None => &[],
        };
        let normalized = normalize_inbound_path_roots(Some(roots_slice));
        for root in normalized {
            if seen.contains(&root) {
                continue;
            }
            seen.insert(root.clone());
            merged.push(root);
        }
    }
    merged
}

pub struct IsInboundPathAllowedParams<'a> {
    pub file_path: &'a str,
    pub roots: &'a [String],
    pub fallback_roots: Option<&'a [String]>,
}

/** Checks whether a candidate inbound media path is covered by configured or fallback roots. */
pub fn is_inbound_path_allowed(params: IsInboundPathAllowedParams) -> bool {
    let candidate_path = match normalize_posix_absolute_path(params.file_path) {
        Some(c) => c,
        None => return false,
    };
    let roots = normalize_inbound_path_roots(Some(params.roots));
    let effective_roots = if !roots.is_empty() {
        roots
    } else {
        normalize_inbound_path_roots(params.fallback_roots)
    };
    if effective_roots.is_empty() {
        return false;
    }
    effective_roots
        .iter()
        .any(|root_pattern| matches_root_pattern(&candidate_path, root_pattern))
}