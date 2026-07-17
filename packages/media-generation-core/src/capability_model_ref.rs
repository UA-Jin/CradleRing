// Media Generation Core module implements capability model ref behavior.
// 翻译自 packages/media-generation-core/src/capability-model-ref.ts

use model_catalog_core::model_catalog_refs::ProviderModelRef;

use crate::string::normalize_optional_string;

/// Provider catalog entry shape used when resolving capability-scoped model references.
pub trait CapabilityModelProviderCandidate {
    fn id(&self) -> &str;
    fn aliases(&self) -> &[String];
    fn default_model(&self) -> Option<&str>;
    fn models(&self) -> &[String];
}

/// Find a provider by id or alias using the caller's provider-id normalization rules.
/// Returns the index of the matched provider in the slice.
pub fn find_capability_provider_by_id<T, F>(
    providers: &[T],
    provider_id: Option<&str>,
    normalize_provider_id: Option<F>,
) -> Option<usize>
where
    T: CapabilityModelProviderCandidate,
    F: Fn(&str) -> Option<String>,
{
    let selected_provider = normalize_provider_for_match(provider_id, normalize_provider_id.as_ref())?;
    let normalizer_ref = normalize_provider_id.as_ref();
    providers.iter().position(|provider| {
        let pid = normalize_provider_for_match(Some(provider.id()), normalizer_ref);
        pid.as_deref() == Some(selected_provider.as_str())
            || provider
                .aliases()
                .iter()
                .any(|alias| {
                    normalize_provider_for_match(Some(alias), normalizer_ref).as_deref()
                        == Some(selected_provider.as_str())
                })
    })
}

fn normalize_provider_for_match<F>(
    value: Option<&str>,
    normalize_provider_id: Option<&F>,
) -> Option<String>
where
    F: Fn(&str) -> Option<String>,
{
    let normalized = value.and_then(normalize_optional_string);
    match (normalized, normalize_provider_id) {
        (Some(n), Some(f)) => f(&n).or(Some(n)),
        (Some(n), None) => Some(n),
        (None, _) => None,
    }
}

/// Provider/model pair resolved by capability-scoped lookup.
pub struct CandidateProviderModel {
    pub provider: String,
    pub model: String,
}

/// Generic provider model for ref parsing (no normalization applied).
pub fn resolve_capability_provider_model_only_ref<T>(
    providers: &[T],
    raw: Option<&str>,
) -> Option<CandidateProviderModel>
where
    T: CapabilityModelProviderCandidate,
{
    let model_str = raw.and_then(normalize_optional_string)?;
    let provider = providers.iter().find(|candidate| {
        let mut models: Vec<Option<String>> = Vec::new();
        models.push(candidate.default_model().map(|s| s.to_string()));
        for m in candidate.models() {
            models.push(Some(m.clone()));
        }
        models
            .iter()
            .any(|entry| entry.as_deref().and_then(normalize_optional_string).as_deref() == Some(model_str.as_str()))
    });
    provider.map(|p| CandidateProviderModel {
        provider: p.id().to_string(),
        model: model_str,
    })
}

/// Resolves provider/model refs first, then falls back to model-only catalog matching.
pub fn resolve_capability_model_ref_for_providers<T, P, F>(
    providers: &[T],
    raw: Option<&str>,
    parse_model_ref: P,
    normalize_provider_id: Option<F>,
) -> Option<ProviderModelRef>
where
    T: CapabilityModelProviderCandidate,
    P: Fn(Option<&str>) -> Option<ProviderModelRef>,
    F: Fn(&str) -> Option<String>,
{
    let raw_str = raw.and_then(normalize_optional_string)?;
    let parsed = parse_model_ref(Some(&raw_str));
    if let Some(ref p) = parsed {
        if find_capability_provider_by_id(providers, Some(&p.provider), normalize_provider_id.as_ref()).is_some() {
            return parsed;
        }
    }
    let model_only = resolve_capability_provider_model_only_ref(providers, Some(&raw_str));
    if let Some(m) = model_only {
        return Some(ProviderModelRef {
            provider: m.provider,
            model: m.model,
        });
    }
    parsed
}