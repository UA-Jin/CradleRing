// Media Generation Core module implements model ref behavior.
// 翻译自 packages/media-generation-core/src/model-ref.ts

use model_catalog_core::model_catalog_refs::{parse_provider_model_ref, ProviderModelRef};

/// Provider/model pair parsed from a generation model reference like `provider/model`.
pub type ParsedGenerationModelRef = ProviderModelRef;

/// Parses strict generation model refs and rejects missing provider or model segments.
pub fn parse_generation_model_ref(raw: Option<&str>) -> Option<ProviderModelRef> {
    raw.and_then(parse_provider_model_ref)
}