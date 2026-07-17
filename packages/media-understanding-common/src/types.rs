// Shared media-understanding provider, attachment, output, and capability contracts.
// 翻译自 packages/media-understanding-common/src/types.ts

use std::collections::HashMap;

/** Kind of media-understanding output produced for an attachment. */
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MediaUnderstandingKind {
    AudioTranscription,
    VideoDescription,
    ImageDescription,
}

/** Capability exposed by a media-understanding provider. */
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MediaUnderstandingCapability {
    Image,
    Audio,
    Video,
}

impl MediaUnderstandingCapability {
    pub fn as_str(&self) -> &'static str {
        match self {
            MediaUnderstandingCapability::Image => "image",
            MediaUnderstandingCapability::Audio => "audio",
            MediaUnderstandingCapability::Video => "video",
        }
    }
}

/** Capability registry keyed by provider id. */
pub type MediaUnderstandingCapabilityRegistry =
    HashMap<String, MediaUnderstandingProviderCapabilities>;

#[derive(Debug, Clone, Default)]
pub struct MediaUnderstandingProviderCapabilities {
    pub capabilities: Option<Vec<MediaUnderstandingCapability>>,
}

/** Media attachment passed to understanding providers. */
#[derive(Debug, Clone, Default)]
pub struct MediaAttachment {
    pub path: Option<String>,
    pub url: Option<String>,
    pub mime: Option<String>,
    pub index: usize,
    pub already_transcribed: Option<bool>,
}

/** Normalized text output produced by media understanding. */
#[derive(Debug, Clone)]
pub struct MediaUnderstandingOutput {
    pub kind: MediaUnderstandingKind,
    pub attachment_index: usize,
    pub text: String,
    pub provider: String,
    pub model: Option<String>,
}

/** Provider shape used for capability discovery and dispatch. */
#[derive(Debug, Clone, Default)]
pub struct MediaUnderstandingProvider {
    pub id: String,
    pub capabilities: Option<Vec<MediaUnderstandingCapability>>,
    pub transcribe_audio: Option<serde_json::Value>,
    pub describe_video: Option<serde_json::Value>,
    pub describe_image: Option<serde_json::Value>,
    pub describe_images: Option<serde_json::Value>,
    pub extract_structured: Option<serde_json::Value>,
}