// 翻译自 packages/media-core/src/constants.ts

/** Default outbound image payload cap shared by media loaders and adapters. */
pub const MAX_IMAGE_BYTES: usize = 6 * 1024 * 1024; // 6MB
/** Default outbound audio payload cap shared by media loaders and adapters. */
pub const MAX_AUDIO_BYTES: usize = 16 * 1024 * 1024; // 16MB
/** Default outbound video payload cap shared by media loaders and adapters. */
pub const MAX_VIDEO_BYTES: usize = 16 * 1024 * 1024; // 16MB
/** Default outbound document payload cap shared by media loaders and adapters. */
pub const MAX_DOCUMENT_BYTES: usize = 100 * 1024 * 1024; // 100MB

/** Media families that share size-policy and MIME-classification behavior. */
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MediaKind {
    Image,
    Audio,
    Video,
    Document,
}

/** Maps a MIME type to the media family used for size limits and routing. */
pub fn media_kind_from_mime(mime: Option<&str>) -> Option<MediaKind> {
    let mime = match mime {
        Some(m) => m,
        None => return None,
    };
    if mime.starts_with("image/") {
        return Some(MediaKind::Image);
    }
    if mime.starts_with("audio/") {
        return Some(MediaKind::Audio);
    }
    if mime.starts_with("video/") {
        return Some(MediaKind::Video);
    }
    if mime == "application/pdf" {
        return Some(MediaKind::Document);
    }
    if mime.starts_with("text/") {
        return Some(MediaKind::Document);
    }
    if mime.starts_with("application/") {
        return Some(MediaKind::Document);
    }
    None
}

/** Returns the default byte cap for a classified media family. */
pub fn max_bytes_for_kind(kind: MediaKind) -> usize {
    match kind {
        MediaKind::Image => MAX_IMAGE_BYTES,
        MediaKind::Audio => MAX_AUDIO_BYTES,
        MediaKind::Video => MAX_VIDEO_BYTES,
        MediaKind::Document => MAX_DOCUMENT_BYTES,
    }
}