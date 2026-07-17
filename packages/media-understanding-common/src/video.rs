// Media Understanding Common module implements video behavior.
// 翻译自 packages/media-understanding-common/src/video.ts

use crate::defaults::DEFAULT_VIDEO_MAX_BASE64_BYTES;

// Video payload size helpers for base64-expanded request bodies.

/** Estimate base64 size for a byte count. */
pub fn estimate_base64_size(bytes: usize) -> usize {
    ((bytes + 2) / 3) * 4
}

/** Resolve video base64 byte limit from raw byte limit and global cap. */
pub fn resolve_video_max_base64_bytes(max_bytes: usize) -> usize {
    let expanded = estimate_base64_size(max_bytes);
    expanded.min(DEFAULT_VIDEO_MAX_BASE64_BYTES)
}