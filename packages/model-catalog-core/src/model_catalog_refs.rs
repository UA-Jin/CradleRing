// Model catalog ref and merge-key builders.
// 翻译自 packages/model-catalog-core/src/model-catalog-refs.ts

use crate::provider_id::normalize_lowercase_string_or_empty;

/// A strict provider/model catalog reference (without normalization on model id).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProviderModelRef {
    pub provider: String,
    pub model: String,
}

/// A normalized provider/model catalog reference (provider lowercased).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelCatalogRef {
    pub provider: String,
    pub model_id: String,
}

/// Normalize provider ids for catalog refs.
pub fn normalize_model_catalog_provider_id(provider: &str) -> String {
    normalize_lowercase_string_or_empty(provider)
}

/// Build a provider/model catalog reference.
pub fn build_model_catalog_ref(provider: &str, model_id: &str) -> String {
    format!("{}/{}", normalize_model_catalog_provider_id(provider), model_id)
}

/// Parse a strict provider/model reference without normalizing either segment.
pub fn parse_provider_model_ref(value: &str) -> Option<ProviderModelRef> {
    let trimmed = value.trim();
    let slash_index = trimmed.find('/')?;
    if slash_index == 0 || slash_index >= trimmed.len() - 1 {
        return None;
    }
    let provider = trimmed[..slash_index].trim().to_string();
    let model = trimmed[slash_index + 1..].trim().to_string();
    if provider.is_empty() || model.is_empty() {
        None
    } else {
        Some(ProviderModelRef { provider, model })
    }
}

/// Parse a strict provider/model catalog reference.
pub fn parse_model_catalog_ref(value: &str) -> Option<ModelCatalogRef> {
    let parsed = parse_provider_model_ref(value)?;
    Some(ModelCatalogRef {
        provider: normalize_model_catalog_provider_id(&parsed.provider),
        model_id: parsed.model,
    })
}

/// Build a case-insensitive merge key for provider/model rows.
pub fn build_model_catalog_merge_key(provider: &str, model_id: &str) -> String {
    format!(
        "{}::{}",
        normalize_model_catalog_provider_id(provider),
        normalize_lowercase_string_or_empty(model_id)
    )
}