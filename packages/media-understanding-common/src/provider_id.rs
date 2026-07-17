// Provider id normalization for media-understanding config and execution.
// 翻译自 packages/media-understanding-common/src/provider-id.ts

/** Normalize a provider id for comparison. */
fn normalize_provider_id(provider: &str) -> String {
    provider.trim().to_lowercase()
}

/** Normalize provider aliases to canonical config provider ids. */
pub fn normalize_media_provider_id(id: &str) -> String {
    let normalized = normalize_provider_id(id);
    if normalized == "gemini" {
        return "google".to_string();
    }
    if normalized == "minimax-cn" {
        return "minimax".to_string();
    }
    if normalized == "minimax-portal-cn" {
        return "minimax-portal".to_string();
    }
    normalized
}

/** Normalize provider ids while preserving execution-specific regional aliases. */
pub fn normalize_media_execution_provider_id(id: &str) -> String {
    let normalized = normalize_provider_id(id);
    if normalized == "minimax-cn" || normalized == "minimax-portal-cn" {
        return normalized;
    }
    normalize_media_provider_id(&normalized)
}