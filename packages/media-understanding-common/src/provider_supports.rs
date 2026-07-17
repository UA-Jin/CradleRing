// Media Understanding Common module implements provider supports behavior.
// 翻译自 packages/media-understanding-common/src/provider-supports.ts

use crate::types::{MediaUnderstandingCapability, MediaUnderstandingProvider};

// Capability checks for media-understanding provider objects.

/** Return true when a provider exposes the method for a media capability. */
pub fn provider_supports_capability(
    provider: Option<&MediaUnderstandingProvider>,
    capability: MediaUnderstandingCapability,
) -> bool {
    let provider = match provider {
        Some(p) => p,
        None => return false,
    };
    match capability {
        MediaUnderstandingCapability::Audio => provider.transcribe_audio.is_some(),
        MediaUnderstandingCapability::Image => provider.describe_image.is_some(),
        MediaUnderstandingCapability::Video => provider.describe_video.is_some(),
    }
}