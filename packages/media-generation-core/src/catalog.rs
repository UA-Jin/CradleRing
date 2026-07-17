// Media Generation Core module implements catalog behavior.
// 翻译自 packages/media-generation-core/src/catalog.ts

use serde::{Deserialize, Serialize};

use crate::string::unique_trimmed_strings;

/// Catalog kind for generated media model entries.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MediaGenerationCatalogKind {
    #[serde(rename = "image_generation")]
    ImageGeneration,
    #[serde(rename = "video_generation")]
    VideoGeneration,
    #[serde(rename = "music_generation")]
    MusicGeneration,
}

/// Source for a media generation catalog entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MediaGenerationCatalogSource {
    #[serde(rename = "static")]
    Static,
    #[serde(rename = "live")]
    Live,
    #[serde(rename = "cache")]
    Cache,
    #[serde(rename = "configured")]
    Configured,
}

/// Media generation model catalog entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaGenerationCatalogEntry<TCapabilities = serde_json::Value> {
    pub kind: MediaGenerationCatalogKind,
    pub provider: String,
    pub model: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    pub source: MediaGenerationCatalogSource,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub configured: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub capabilities: Option<TCapabilities>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub modes: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auth_env_vars: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub docs_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fetched_at: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub warnings: Option<Vec<String>>,
}

/// Static catalog metadata that overrides provider defaults for one model.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MediaGenerationCatalogModelEntry<TCapabilities = serde_json::Value> {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub capabilities: Option<TCapabilities>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub modes: Option<Vec<String>>,
}

/// Provider metadata used to synthesize static media generation catalog entries.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaGenerationCatalogProvider<TCapabilities = serde_json::Value> {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub aliases: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub models: Option<Vec<String>>,
    pub capabilities: TCapabilities,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub catalog_by_model: Option<std::collections::HashMap<String, MediaGenerationCatalogModelEntry<TCapabilities>>>,
}

/// Provider-like shape with optional defaults/models list.
struct ProviderShape<'a> {
    default_model: Option<&'a str>,
    models: Option<&'a [String]>,
}

/// Return unique configured models with default model first when present.
fn unique_models(provider: ProviderShape<'_>) -> Vec<String> {
    let mut values: Vec<Option<String>> = Vec::new();
    values.push(provider.default_model.map(|s| s.to_string()));
    if let Some(models) = provider.models {
        for m in models {
            values.push(Some(m.clone()));
        }
    }
    unique_trimmed_strings(&values)
}

/// Synthesize static catalog entries from provider metadata.
pub fn synthesize_media_generation_catalog_entries<TCapabilities>(
    kind: MediaGenerationCatalogKind,
    provider: &MediaGenerationCatalogProvider<TCapabilities>,
    modes: Option<&[String]>,
) -> Vec<MediaGenerationCatalogEntry<TCapabilities>>
where
    TCapabilities: Clone,
{
    let default_raw: Option<String> = provider.default_model.clone();
    let default_model = unique_trimmed_strings(&[default_raw]).into_iter().next();

    let models = unique_models(ProviderShape {
        default_model: provider.default_model.as_deref(),
        models: provider.models.as_deref(),
    });

    models
        .into_iter()
        .map(|model| {
            let model_catalog_entry = provider
                .catalog_by_model
                .as_ref()
                .and_then(|m| m.get(&model));
            let mut entry = MediaGenerationCatalogEntry {
                kind,
                provider: provider.id.clone(),
                model: model.clone(),
                source: MediaGenerationCatalogSource::Static,
                default: None,
                configured: None,
                label: None,
                capabilities: model_catalog_entry
                    .and_then(|e| e.capabilities.clone())
                    .or_else(|| Some(provider.capabilities.clone())),
                modes: None,
                auth_env_vars: None,
                docs_path: None,
                fetched_at: None,
                expires_at: None,
                warnings: None,
            };
            if let Some(label) = &provider.label {
                entry.label = Some(label.clone());
            }
            if Some(&model) == default_model.as_ref() {
                entry.default = Some(true);
            }
            let entry_modes = model_catalog_entry.and_then(|e| e.modes.clone()).or_else(|| modes.map(|m| m.to_vec()));
            if let Some(m) = entry_modes {
                entry.modes = Some(m);
            }
            entry
        })
        .collect()
}

/// Return unique model ids exposed by a media generation provider.
pub fn list_media_generation_provider_models(provider: &MediaGenerationCatalogProvider) -> Vec<String> {
    unique_models(ProviderShape {
        default_model: provider.default_model.as_deref(),
        models: provider.models.as_deref(),
    })
}