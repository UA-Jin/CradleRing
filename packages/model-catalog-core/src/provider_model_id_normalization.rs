// Provider model id normalization policies.
// 翻译自 packages/model-catalog-core/src/provider-model-id-normalization.ts

use std::collections::{BTreeMap, HashMap};

use crate::model_catalog_refs::{build_model_catalog_merge_key, parse_model_catalog_ref};
use crate::provider_id::normalize_lowercase_string_or_empty;
use crate::provider_model_id_normalize::{
    normalize_google_preview_model_id, normalize_together_model_id,
};

/// Manifest-defined normalization rules for one provider.
#[derive(Debug, Clone, Default)]
pub struct ManifestModelIdNormalizationProvider {
    pub aliases: Option<BTreeMap<String, String>>,
    pub strip_prefixes: Option<Vec<String>>,
    pub prefix_when_bare: Option<String>,
    pub prefix_when_bare_after_alias_starts_with: Option<Vec<PrefixWhenBareAfterAlias>>,
}

#[derive(Debug, Clone)]
pub struct PrefixWhenBareAfterAlias {
    pub model_prefix: String,
    pub prefix: String,
}

/// Manifest fragment that can define provider model-id normalization policies.
#[derive(Debug, Clone, Default)]
pub struct ManifestModelIdNormalizationRecord {
    pub model_id_normalization: Option<ManifestModelIdNormalizationInner>,
}

#[derive(Debug, Clone, Default)]
pub struct ManifestModelIdNormalizationInner {
    pub providers: Option<BTreeMap<String, ManifestModelIdNormalizationProvider>>,
}

/// Collect provider model-id normalization policies from plugin manifests.
pub fn collect_manifest_model_id_normalization_policies(
    plugins: &[ManifestModelIdNormalizationRecord],
) -> HashMap<String, ManifestModelIdNormalizationProvider> {
    let mut policies: HashMap<String, ManifestModelIdNormalizationProvider> = HashMap::new();
    for plugin in plugins {
        let providers = match plugin
            .model_id_normalization
            .as_ref()
            .and_then(|m| m.providers.as_ref())
        {
            Some(p) => p,
            None => continue,
        };
        for (provider, policy) in providers {
            policies.insert(normalize_lowercase_string_or_empty(provider.as_str()), policy.clone());
        }
    }
    policies
}

/// Static snapshot of manifest normalization policies. Used by
/// `normalize_configured_provider_catalog_model_id` to apply cross-cutting
/// provider/model rules.
use std::sync::{Mutex, OnceLock};

static CURRENT_MANIFEST_POLICIES: OnceLock<Mutex<Option<HashMap<String, ManifestModelIdNormalizationProvider>>>> =
    OnceLock::new();

fn policies_slot() -> &'static Mutex<Option<HashMap<String, ManifestModelIdNormalizationProvider>>> {
    CURRENT_MANIFEST_POLICIES.get_or_init(|| Mutex::new(None))
}

/// Replace the process-local manifest normalization policy snapshot.
pub fn set_current_manifest_model_id_normalization_records(
    plugins: Option<&[ManifestModelIdNormalizationRecord]>,
) {
    let mut slot = policies_slot().lock().expect("manifest policy mutex poisoned");
    *slot = plugins.map(collect_manifest_model_id_normalization_policies);
}

/// Return the current process-local manifest normalization policy snapshot.
fn get_current_manifest_model_id_normalization_policies(
) -> Option<HashMap<String, ManifestModelIdNormalizationProvider>> {
    policies_slot().lock().expect("manifest policy mutex poisoned").clone()
}

/// Return true when a model id already includes a provider namespace.
fn has_provider_prefix(model_id: &str) -> bool {
    model_id.contains('/')
}

/// Join a provider prefix and model id with exactly one slash.
fn format_prefixed_model_id(prefix: &str, model_id: &str) -> String {
    let trimmed_prefix = prefix.trim_end_matches('/');
    let trimmed_model = model_id.trim_start_matches('/');
    format!("{}/{}", trimmed_prefix, trimmed_model)
}

/// Strip a duplicated self-provider prefix from a model id.
pub fn strip_self_provider_model_prefix(provider: &str, model: &str) -> String {
    let prefix = format!("{}/", normalize_lowercase_string_or_empty(provider));
    let trimmed = model.trim();
    if normalize_lowercase_string_or_empty(trimmed).starts_with(&prefix) {
        trimmed[prefix.len()..].to_string()
    } else {
        model.to_string()
    }
}

/// Apply manifest normalization policies for one provider/model id.
pub fn normalize_provider_model_id_with_policies(params: NormalizeWithPoliciesParams) -> Option<String> {
    let policy = match params
        .policies
        .get(&normalize_lowercase_string_or_empty(&params.provider))
    {
        Some(p) => p,
        None => return None,
    };

    let mut model_id = params.context.model_id.trim().to_string();
    if model_id.is_empty() {
        return Some(model_id);
    }

    if let Some(prefixes) = policy.strip_prefixes.as_ref() {
        for prefix in prefixes {
            let normalized_prefix = normalize_lowercase_string_or_empty(prefix);
            if !normalized_prefix.is_empty()
                && normalize_lowercase_string_or_empty(&model_id).starts_with(&normalized_prefix)
            {
                model_id = model_id[normalized_prefix.len()..].to_string();
                break;
            }
        }
    }

    if let Some(aliases) = policy.aliases.as_ref() {
        let key = normalize_lowercase_string_or_empty(&model_id);
        if let Some(replacement) = aliases.get(&key) {
            model_id = replacement.clone();
        }
    }

    if !has_provider_prefix(&model_id) {
        if let Some(rules) = policy.prefix_when_bare_after_alias_starts_with.as_ref() {
            for rule in rules {
                if normalize_lowercase_string_or_empty(&model_id)
                    .starts_with(&rule.model_prefix.to_lowercase())
                {
                    return Some(format_prefixed_model_id(&rule.prefix, &model_id));
                }
            }
        }
        if let Some(prefix) = policy.prefix_when_bare.as_ref() {
            return Some(format_prefixed_model_id(prefix, &model_id));
        }
    }

    Some(model_id)
}

/// Parameters for `normalize_provider_model_id_with_policies`.
pub struct NormalizeWithPoliciesParams<'a> {
    pub provider: String,
    pub policies: &'a HashMap<String, ManifestModelIdNormalizationProvider>,
    pub context: NormalizeWithPoliciesContext,
}

pub struct NormalizeWithPoliciesContext {
    pub model_id: String,
}

/// Apply built-in provider-specific model id normalization rules.
pub fn normalize_built_in_provider_model_id(provider: &str, model: &str) -> String {
    let normalized_provider = normalize_lowercase_string_or_empty(provider);
    if normalized_provider == "google"
        || normalized_provider == "google-gemini-cli"
        || normalized_provider == "google-vertex"
    {
        return normalize_google_preview_model_id(model);
    }
    if normalized_provider == "openrouter" {
        let trimmed = model.trim();
        return if !trimmed.is_empty() && !trimmed.contains('/') {
            format!("openrouter/{}", trimmed)
        } else {
            model.to_string()
        };
    }
    if normalized_provider == "anthropic" {
        let anthropic_aliases: &[(&str, &str)] = &[
            ("opus-4.8", "claude-opus-4-8"),
            ("opus", "claude-opus-4-8"),
            ("opus-4.6", "claude-opus-4-6"),
            ("sonnet-5", "claude-sonnet-5"),
            ("sonnet", "claude-sonnet-5"),
            ("sonnet-4.6", "claude-sonnet-4-6"),
        ];
        let anthropic_prefix = "anthropic/";
        let normalized_model = normalize_lowercase_string_or_empty(model);
        let provider_model = if normalized_model.starts_with(anthropic_prefix) {
            model.trim()[anthropic_prefix.len()..].to_string()
        } else {
            model.to_string()
        };
        let key = normalize_lowercase_string_or_empty(&provider_model);
        for (alias, target) in anthropic_aliases {
            if *alias == key {
                return target.to_string();
            }
        }
        return provider_model;
    }
    if normalized_provider == "vercel-ai-gateway" {
        let vercel_aliases: &[(&str, &str)] = &[
            ("opus-4.6", "claude-opus-4-6"),
            ("sonnet-5", "claude-sonnet-5"),
            ("sonnet", "claude-sonnet-4-6"),
            ("sonnet-4.6", "claude-sonnet-4-6"),
        ];
        let key = normalize_lowercase_string_or_empty(model);
        let mut aliased = model.to_string();
        for (alias, target) in vercel_aliases {
            if *alias == key {
                aliased = target.to_string();
                break;
            }
        }
        if normalize_lowercase_string_or_empty(&aliased).starts_with("claude-") {
            return format!("anthropic/{}", aliased);
        }
        return aliased;
    }
    if normalized_provider == "huggingface" {
        let prefix = "huggingface/";
        return if normalize_lowercase_string_or_empty(model).starts_with(prefix) {
            model[prefix.len()..].to_string()
        } else {
            model.to_string()
        };
    }
    if normalized_provider == "nvidia" {
        let trimmed = model.trim();
        return if !trimmed.is_empty() && !trimmed.contains('/') {
            format!("nvidia/{}", trimmed)
        } else {
            model.to_string()
        };
    }
    if normalized_provider == "xai" {
        let xai_aliases: &[(&str, &str)] = &[
            ("grok-4.3-latest", "grok-4.3"),
            ("grok-4.5-latest", "grok-4.5"),
            ("grok-build-latest", "grok-4.5"),
            ("grok-4-fast-reasoning", "grok-4-fast"),
            ("grok-4-1-fast-reasoning", "grok-4-1-fast"),
        ];
        let key = normalize_lowercase_string_or_empty(model);
        for (alias, target) in xai_aliases {
            if *alias == key {
                return target.to_string();
            }
        }
        return model.to_string();
    }
    if normalized_provider == "openai" {
        return model.to_string();
    }
    if normalized_provider == "together" {
        return normalize_together_model_id(model);
    }
    model.to_string()
}

/// Apply manifest policies and built-in normalization to a static provider/model id.
pub fn normalize_static_provider_model_id_with_policies(
    provider: &str,
    model: &str,
    policies: Option<&HashMap<String, ManifestModelIdNormalizationProvider>>,
) -> String {
    let normalized_provider = normalize_lowercase_string_or_empty(provider);
    let manifest_model_id = if let Some(policies) = policies {
        normalize_provider_model_id_with_policies(NormalizeWithPoliciesParams {
            provider: normalized_provider.clone(),
            policies,
            context: NormalizeWithPoliciesContext {
                model_id: model.to_string(),
            },
        })
        .unwrap_or_else(|| model.to_string())
    } else {
        model.to_string()
    };
    normalize_built_in_provider_model_id(&normalized_provider, &manifest_model_id)
}

/// Normalize a configured provider/model catalog reference using current policies.
pub fn normalize_configured_provider_catalog_model_id(
    provider: &str,
    model: &str,
    policies: Option<&HashMap<String, ManifestModelIdNormalizationProvider>>,
) -> String {
    let policies_owned;
    let policies_ref: Option<&HashMap<String, ManifestModelIdNormalizationProvider>> = match policies
    {
        Some(p) => Some(p),
        None => {
            policies_owned = get_current_manifest_model_id_normalization_policies();
            policies_owned.as_ref()
        }
    };
    let provider_model =
        normalize_static_provider_model_id_with_policies(provider, model, policies_ref);
    normalize_configured_provider_catalog_model_ref(&provider_model)
}

/// Normalize embedded Google model aliases inside provider/model catalog refs.
pub fn normalize_configured_provider_catalog_model_ref(provider_model: &str) -> String {
    let google_prefix = "google/";
    if !provider_model.starts_with(google_prefix) {
        let parsed = match parse_model_catalog_ref(provider_model) {
            Some(p) => p,
            None => return provider_model.to_string(),
        };
        if !parsed.model_id.starts_with(google_prefix) {
            return provider_model.to_string();
        }
        let normalized_model_id = normalize_google_preview_model_id(&parsed.model_id);
        return if normalized_model_id == parsed.model_id {
            provider_model.to_string()
        } else {
            format!("{}/{}", parsed.provider, normalized_model_id)
        };
    }
    let model_id = &provider_model[google_prefix.len()..];
    let normalized_model_id = normalize_google_preview_model_id(model_id);
    if normalized_model_id == model_id {
        provider_model.to_string()
    } else {
        format!("{}{}", google_prefix, normalized_model_id)
    }
}

/// Re-export for `build_model_catalog_merge_key` to keep call-sites aligned.
#[allow(dead_code)]
pub(crate) fn _merge_key_for(provider: &str, model_id: &str) -> String {
    build_model_catalog_merge_key(provider, model_id)
}