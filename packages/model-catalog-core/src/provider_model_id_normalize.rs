// Provider model id normalization helpers (Google / Together / Antigravity).
// 翻译自 packages/model-catalog-core/src/provider-model-id-normalize.ts

fn antigravity_bare_pro_ids_contains(id: &str) -> bool {
    matches!(id, "gemini-3-pro" | "gemini-3.1-pro" | "gemini-3-1-pro")
}

const GOOGLE_PROVIDER_PREFIX: &str = "google/";

/// Normalize a Google preview model id to the current canonical id.
pub fn normalize_google_preview_model_id(id: &str) -> String {
    if let Some(stripped) = id.strip_prefix(GOOGLE_PROVIDER_PREFIX) {
        let normalized = normalize_google_preview_model_id(stripped);
        return if normalized == stripped {
            id.to_string()
        } else {
            format!("{}{}", GOOGLE_PROVIDER_PREFIX, normalized)
        };
    }
    match id {
        "gemini-3-pro" | "gemini-3-pro-preview" => "gemini-3.1-pro-preview".to_string(),
        "gemini-3-flash" => "gemini-3-flash-preview".to_string(),
        "gemini-3.1-pro" => "gemini-3.1-pro-preview".to_string(),
        // Gemini 3.1 Flash Lite graduated to GA on 2026-05-07; the -preview
        // endpoint is deprecated (shutdown 2026-05-25). Map old preview name
        // to the stable GA id.
        "gemini-3.1-flash-lite-preview" => "gemini-3.1-flash-lite".to_string(),
        "gemini-3.1-flash" | "gemini-3.1-flash-preview" => "gemini-3-flash-preview".to_string(),
        "gemma-4-26b" => "gemma-4-26b-a4b-it".to_string(),
        _ => id.to_string(),
    }
}

/// Normalize a Together model id.
pub fn normalize_together_model_id(id: &str) -> String {
    if id == "moonshotai/Kimi-K2.5" {
        "moonshotai/Kimi-K2.6".to_string()
    } else {
        id.to_string()
    }
}

/// Normalize Antigravity preview model id (bare pro -> -low variant).
pub fn normalize_antigravity_preview_model_id(id: &str) -> String {
    if antigravity_bare_pro_ids_contains(id) {
        format!("{}-low", id)
    } else {
        id.to_string()
    }
}