// Media Understanding Common module implements defaults behavior.
// 翻译自 packages/media-understanding-common/src/defaults.ts

use crate::types::MediaUnderstandingCapability;

const MB: usize = 1024 * 1024;

/** Default max response characters for bounded text outputs. */
pub const DEFAULT_MAX_CHARS: usize = 500;
/** Default max response characters by capability. */
pub fn default_max_chars_by_capability(cap: MediaUnderstandingCapability) -> Option<usize> {
    match cap {
        MediaUnderstandingCapability::Image => Some(DEFAULT_MAX_CHARS),
        MediaUnderstandingCapability::Audio => None,
        MediaUnderstandingCapability::Video => Some(DEFAULT_MAX_CHARS),
    }
}
/** Default input byte limits by capability. */
pub fn default_max_bytes(cap: MediaUnderstandingCapability) -> usize {
    match cap {
        MediaUnderstandingCapability::Image => 10 * MB,
        MediaUnderstandingCapability::Audio => 20 * MB,
        MediaUnderstandingCapability::Video => 50 * MB,
    }
}
/** Default request timeout by capability. */
pub fn default_timeout_seconds(cap: MediaUnderstandingCapability) -> usize {
    match cap {
        MediaUnderstandingCapability::Image => 60,
        MediaUnderstandingCapability::Audio => 60,
        MediaUnderstandingCapability::Video => 120,
    }
}
/** Default prompts by capability. */
pub fn default_prompt(cap: MediaUnderstandingCapability) -> &'static str {
    match cap {
        MediaUnderstandingCapability::Image => "Describe the image.",
        MediaUnderstandingCapability::Audio => "Transcribe the audio.",
        MediaUnderstandingCapability::Video => "Describe the video.",
    }
}
/** Upper bound for base64-expanded video payloads. */
pub const DEFAULT_VIDEO_MAX_BASE64_BYTES: usize = 70 * MB;
/** CLI output buffer used by provider child processes. */
pub const CLI_OUTPUT_MAX_BUFFER: usize = 5 * MB;
/** Default parallel media-understanding request count. */
pub const DEFAULT_MEDIA_CONCURRENCY: usize = 2;
/** Minimum bytes for audio files before transcription is attempted. */
pub const MIN_AUDIO_FILE_BYTES: usize = 1024;