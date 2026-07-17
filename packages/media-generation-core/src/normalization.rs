// Primitive value types reported in media generation normalization metadata.
// 翻译自 packages/media-generation-core/src/normalization.ts

use serde::{Deserialize, Serialize};

/// Primitive value types reported in media generation normalization metadata.
pub type MediaNormalizationValue = String;

/// Requested/applied value pair plus provenance for a normalized media option.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MediaNormalizationEntry {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub requested: Option<MediaNormalizationValue>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub applied: Option<MediaNormalizationValue>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub derived_from: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub supported_values: Option<Vec<MediaNormalizationValue>>,
}

/// Normalization metadata shared by media generation responses.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MediaGenerationNormalizationMetadataInput {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<MediaNormalizationEntry>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub aspect_ratio: Option<MediaNormalizationEntry>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resolution: Option<MediaNormalizationEntry>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration_seconds: Option<MediaNormalizationEntry>,
}

/// True when a normalization entry contains any user-visible normalization metadata.
pub fn has_media_normalization_entry(entry: &Option<MediaNormalizationEntry>) -> bool {
    match entry {
        None => false,
        Some(e) => {
            e.requested.is_some()
                || e.applied.is_some()
                || e.derived_from.is_some()
                || e.supported_values
                    .as_ref()
                    .map(|v| !v.is_empty())
                    .unwrap_or(false)
        }
    }
}