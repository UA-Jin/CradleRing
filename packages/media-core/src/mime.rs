// Media Core module implements mime behavior.
// 翻译自 packages/media-core/src/mime.ts

use std::collections::HashMap;
use std::path::Path;
use std::sync::OnceLock;

use regex::Regex;

use crate::constants::{media_kind_from_mime, MediaKind};
use crate::lazy_import::{create_lazy_import_loader, LazyPromiseLoader};

/** Maximum byte prefix passed to dependency MIME sniffers for bounded memory/CPU work. */
pub const FILE_TYPE_SNIFF_MAX_BYTES: usize = 1024 * 1024;

// Map common mimes to preferred file extensions.
fn ext_by_mime() -> &'static HashMap<&'static str, &'static str> {
    static MAP: OnceLock<HashMap<&'static str, &'static str>> = OnceLock::new();
    MAP.get_or_init(|| {
        let mut m: HashMap<&'static str, &'static str> = HashMap::new();
        m.insert("image/heic", ".heic");
        m.insert("image/heif", ".heif");
        m.insert("image/bmp", ".bmp");
        m.insert("image/jpg", ".jpg");
        m.insert("image/jpeg", ".jpg");
        m.insert("image/png", ".png");
        m.insert("image/svg+xml", ".svg");
        m.insert("image/webp", ".webp");
        m.insert("image/gif", ".gif");
        m.insert("audio/ogg", ".ogg");
        m.insert("audio/mpeg", ".mp3");
        m.insert("audio/mp3", ".mp3");
        m.insert("audio/wav", ".wav");
        m.insert("audio/wave", ".wav");
        m.insert("audio/x-wav", ".wav");
        m.insert("audio/flac", ".flac");
        m.insert("audio/aac", ".aac");
        m.insert("audio/opus", ".opus");
        m.insert("audio/webm", ".webm");
        m.insert("audio/x-m4a", ".m4a");
        m.insert("audio/mp4", ".m4a");
        m.insert("audio/x-caf", ".caf");
        m.insert("video/x-msvideo", ".avi");
        m.insert("video/mp4", ".mp4");
        m.insert("video/x-matroska", ".mkv");
        m.insert("video/webm", ".webm");
        m.insert("video/x-flv", ".flv");
        m.insert("video/x-ms-wmv", ".wmv");
        m.insert("video/quicktime", ".mov");
        m.insert("application/pdf", ".pdf");
        m.insert("application/json", ".json");
        m.insert("application/yaml", ".yaml");
        m.insert("application/zip", ".zip");
        m.insert("application/gzip", ".gz");
        m.insert("application/x-tar", ".tar");
        m.insert("application/x-7z-compressed", ".7z");
        m.insert("application/vnd.rar", ".rar");
        m.insert("application/msword", ".doc");
        m.insert("application/vnd.ms-excel", ".xls");
        m.insert("application/vnd.ms-powerpoint", ".ppt");
        m.insert(
            "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
            ".docx",
        );
        m.insert(
            "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
            ".xlsx",
        );
        m.insert(
            "application/vnd.openxmlformats-officedocument.presentationml.presentation",
            ".pptx",
        );
        m.insert("text/csv", ".csv");
        m.insert("text/plain", ".txt");
        m.insert("text/markdown", ".md");
        m.insert("text/html", ".html");
        m.insert("text/xml", ".xml");
        m.insert("text/css", ".css");
        m.insert("application/xml", ".xml");
        m
    })
}

fn build_mime_by_ext() -> HashMap<String, String> {
    let mut by_ext: HashMap<String, String> = HashMap::new();
    for (mime, ext) in ext_by_mime().iter() {
        by_ext.entry(ext.to_string()).or_insert_with(|| mime.to_string());
    }
    by_ext
}

fn mime_by_ext() -> &'static HashMap<String, String> {
    static MAP: OnceLock<HashMap<String, String>> = OnceLock::new();
    MAP.get_or_init(|| {
        let mut m = build_mime_by_ext();
        // Canonical extension mappings for common MIME aliases
        m.insert(".jpg".to_string(), "image/jpeg".to_string());
        m.insert(".m2a".to_string(), "audio/mpeg".to_string());
        m.insert(".mp3".to_string(), "audio/mpeg".to_string());
        m.insert(".oga".to_string(), "audio/ogg".to_string());
        m.insert(".wav".to_string(), "audio/wav".to_string());
        m.insert(".webm".to_string(), "video/webm".to_string());
        // Additional extension aliases
        m.insert(".jpeg".to_string(), "image/jpeg".to_string());
        m.insert(".js".to_string(), "text/javascript".to_string());
        m.insert(".log".to_string(), "text/plain".to_string());
        m.insert(".htm".to_string(), "text/html".to_string());
        m.insert(".xml".to_string(), "text/xml".to_string());
        m.insert(".yml".to_string(), "application/yaml".to_string());
        m
    })
}

#[derive(Clone)]
struct FileTypeResult {
    mime: String,
}

// Replacement for the node `file-type` package: sniff common magic bytes in-buffer.
fn file_type_from_buffer(buffer: &[u8]) -> Option<FileTypeResult> {
    if buffer.len() >= 8 && &buffer[..8] == b"\x89PNG\r\n\x1a\n" {
        return Some(FileTypeResult { mime: "image/png".to_string() });
    }
    if buffer.len() >= 3 && buffer[0] == 0xff && buffer[1] == 0xd8 && buffer[2] == 0xff {
        return Some(FileTypeResult { mime: "image/jpeg".to_string() });
    }
    if buffer.len() >= 12 && &buffer[..4] == b"RIFF" && &buffer[8..12] == b"WEBP" {
        return Some(FileTypeResult { mime: "image/webp".to_string() });
    }
    if buffer.len() >= 6 {
        if &buffer[..6] == b"GIF87a" || &buffer[..6] == b"GIF89a" {
            return Some(FileTypeResult { mime: "image/gif".to_string() });
        }
    }
    if buffer.len() >= 4 && &buffer[..4] == b"caff" {
        return Some(FileTypeResult { mime: "audio/x-caf".to_string() });
    }
    if buffer.len() >= 4 && &buffer[..4] == b"OggS" {
        return Some(FileTypeResult { mime: "audio/ogg".to_string() });
    }
    if buffer.len() >= 4 && &buffer[..4] == b"fLaC" {
        return Some(FileTypeResult { mime: "audio/flac".to_string() });
    }
    if buffer.len() >= 2 && buffer[0] == 0x42 && buffer[1] == 0x4d {
        return Some(FileTypeResult { mime: "image/bmp".to_string() });
    }
    if buffer.len() >= 4 && &buffer[..4] == b"%PDF" {
        return Some(FileTypeResult { mime: "application/pdf".to_string() });
    }
    if buffer.len() >= 4 && &buffer[..4] == b"PK\x03\x04" {
        return Some(FileTypeResult { mime: "application/zip".to_string() });
    }
    if buffer.len() >= 12 && &buffer[4..8] == b"ftyp" {
        let brand = &buffer[8..12];
        if brand == b"mp42" || brand == b"isom" || brand == b"M4V " {
            return Some(FileTypeResult { mime: "video/mp4".to_string() });
        }
        if brand == b"M4A " || brand == b"M4B " {
            return Some(FileTypeResult { mime: "audio/mp4".to_string() });
        }
    }
    if buffer.len() >= 4 && &buffer[..4] == b"\x1aE\xdf\xa3" {
        return Some(FileTypeResult { mime: "video/x-matroska".to_string() });
    }
    None
}

fn file_type_module_loader(
) -> &'static LazyPromiseLoader<std::sync::Arc<std::sync::Mutex<HashMap<Vec<u8>, String>>>> {
    // The original TS uses createLazyImportLoader to dynamically import `file-type`.
    // In our Rust translation, file_type_from_buffer is a sync function, so we wrap
    // it in a lazy loader that returns immediately with a sentinel type. This
    // preserves the call shape so the surrounding code stays 1:1.
    static LOADER: OnceLock<LazyPromiseLoader<std::sync::Arc<std::sync::Mutex<HashMap<Vec<u8>, String>>>>> =
        OnceLock::new();
    LOADER.get_or_init(|| {
        create_lazy_import_loader(
            Box::new(|| {
                Box::pin(async {
                    Ok(std::sync::Arc::new(std::sync::Mutex::new(HashMap::new())))
                })
            }),
            Default::default(),
        )
    })
}

/** Normalizes MIME strings by dropping parameters, lowercasing, and folding APNG to PNG. */
pub fn normalize_mime_type(mime: Option<&str>) -> Option<String> {
    let mime = mime?;
    let cleaned = mime.split(';').next()?.trim().to_lowercase();
    if cleaned == "image/apng" {
        return Some("image/png".to_string());
    }
    if cleaned.is_empty() {
        None
    } else {
        Some(cleaned)
    }
}

/** Returns the bounded buffer prefix used for dependency MIME sniffing. */
pub fn slice_mime_sniff_buffer(buffer: &[u8]) -> Vec<u8> {
    if buffer.len() <= FILE_TYPE_SNIFF_MAX_BYTES {
        return buffer.to_vec();
    }
    buffer[..FILE_TYPE_SNIFF_MAX_BYTES].to_vec()
}

async fn sniff_mime(buffer: Option<&[u8]>) -> Option<String> {
    let buffer = buffer?;
    // Touch the lazy loader to preserve 1:1 shape with the original implementation.
    let _ = file_type_module_loader().load().await;
    if let Some(ft) = file_type_from_buffer(buffer) {
        return normalize_mime_type(Some(&ft.mime));
    }
    sniff_known_audio_magic(buffer)
}

// Fallbacks for audio containers `file-type` doesn't recognize natively (e.g.
// Apple's CAF, used by iMessage voice memos when produced by `afconvert`).
// Without this the host-local-media validator drops these buffers as unknown
// binary blobs because the sniff returns undefined, even though the file is
// a valid audio container.
fn sniff_known_audio_magic(buffer: &[u8]) -> Option<String> {
    if buffer.len() >= 4 && &buffer[..4] == b"caff" {
        return Some("audio/x-caf".to_string());
    }
    None
}

/** Extracts a lowercase extension from a local path or HTTP URL pathname. */
pub fn get_file_extension(file_path: Option<&str>) -> Option<String> {
    let file_path = file_path?;
    if let Ok(url) = url_parse(file_path) {
        if url.starts_with("http://") || url.starts_with("https://") {
            let scheme_end = url.find("://")? + 3;
            let after_scheme = &url[scheme_end..];
            let path_start = after_scheme.find('/').unwrap_or(after_scheme.len());
            let path = &after_scheme[path_start..];
            let last_slash = path.rfind('/').unwrap_or(0);
            let mut filename = path[last_slash + 1..].to_string();
            let decodable = filename.replace("%2f", "%252F").replace("%5c", "%255C");
            if let Ok(decoded) = percent_decode(&decodable) {
                filename = decoded;
            }
            let ext = path_extname(&filename).to_lowercase();
            return if ext.is_empty() { None } else { Some(ext) };
        }
    }
    let ext = Path::new(file_path)
        .extension()
        .and_then(|s| s.to_str())
        .map(|s| format!(".{}", s.to_lowercase()))
        .unwrap_or_default();
    if ext.is_empty() {
        None
    } else {
        Some(ext)
    }
}

fn path_extname(p: &str) -> String {
    let last_slash = p.rfind('/').map(|i| i + 1).unwrap_or(0);
    let last_backslash = p.rfind('\\').map(|i| i + 1).unwrap_or(0);
    let start = last_slash.max(last_backslash);
    let name = &p[start..];
    match name.rfind('.') {
        Some(i) => name[i..].to_string(),
        None => String::new(),
    }
}

fn url_parse(s: &str) -> Result<&str, ()> {
    // Minimal URL parser: just return the string if it has a scheme prefix.
    if s.contains("://") {
        Ok(s)
    } else {
        Err(())
    }
}

fn percent_decode(s: &str) -> Result<String, ()> {
    // Minimal percent-decoder: decode %XX sequences into bytes.
    let bytes = s.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            let hex = std::str::from_utf8(&bytes[i + 1..i + 3]).map_err(|_| ())?;
            let value = u8::from_str_radix(hex, 16).map_err(|_| ())?;
            out.push(value);
            i += 3;
        } else {
            out.push(bytes[i]);
            i += 1;
        }
    }
    String::from_utf8(out).map_err(|_| ())
}

/** Maps a file path or URL extension to the preferred MIME type when known. */
pub fn mime_type_from_file_path(file_path: Option<&str>) -> Option<String> {
    let ext = get_file_extension(file_path)?;
    mime_by_ext().get(&ext).cloned()
}

/** Returns true when a filename extension is a supported audio container. */
pub fn is_audio_file_name(file_name: Option<&str>) -> bool {
    media_kind_from_mime(mime_type_from_file_path(file_name).as_deref()) == Some(MediaKind::Audio)
}

pub struct DetectMimeOpts<'a> {
    pub buffer: Option<&'a [u8]>,
    pub header_mime: Option<&'a str>,
    pub file_path: Option<&'a str>,
}

/** Detects the best MIME type from bytes, file path, and header metadata. */
pub async fn detect_mime(opts: DetectMimeOpts<'_>) -> Option<String> {
    let ext = get_file_extension(opts.file_path);
    let ext_mime = ext.as_ref().and_then(|e| mime_by_ext().get(e).cloned());

    let header_mime = normalize_mime_type(opts.header_mime);
    let sniffed = sniff_mime(opts.buffer).await;
    let sniffed_generic_container = sniffed.as_ref().map(|s| is_generic_mime(Some(s))).unwrap_or(false);
    let trusted_ext_mime = if sniffed_generic_container && is_image_mime(ext_mime.as_deref()) {
        None
    } else {
        ext_mime
    };
    let trusted_header_mime = if sniffed_generic_container && is_image_mime(header_mime.as_deref()) {
        None
    } else {
        header_mime
    };

    // Prefer sniffed types, but don't let generic container types override a more
    // specific extension mapping (e.g. XLSX vs ZIP).
    if let Some(ref s) = sniffed {
        if !is_generic_mime(Some(s)) || trusted_ext_mime.is_none() {
            return Some(s.clone());
        }
    }
    if let Some(ref e) = trusted_ext_mime {
        return Some(e.clone());
    }
    if let Some(ref h) = trusted_header_mime {
        if !is_generic_mime(Some(h)) {
            return Some(h.clone());
        }
    }
    if let Some(ref s) = sniffed {
        return Some(s.clone());
    }
    trusted_header_mime
}

fn is_generic_mime(mime: Option<&str>) -> bool {
    match mime {
        None => true,
        Some(m) => {
            let m = m.to_lowercase();
            m == "application/octet-stream" || m == "application/zip"
        }
    }
}

fn is_image_mime(mime: Option<&str>) -> bool {
    media_kind_from_mime(mime) == Some(MediaKind::Image)
}

/** Returns the preferred file extension for a normalized or raw MIME string. */
pub fn extension_for_mime(mime: Option<&str>) -> Option<String> {
    let normalized = normalize_mime_type(mime)?;
    ext_by_mime().get(normalized.as_str()).map(|s| s.to_string())
}

pub struct IsGifMediaOpts<'a> {
    pub content_type: Option<&'a str>,
    pub file_name: Option<&'a str>,
}

/** Returns true when content type or filename identifies GIF media. */
pub fn is_gif_media(opts: IsGifMediaOpts<'_>) -> bool {
    if normalize_mime_type(opts.content_type).as_deref() == Some("image/gif") {
        return true;
    }
    let ext = get_file_extension(opts.file_name);
    ext.as_deref() == Some(".gif")
}

/** Maps image format labels from encoders/probes to MIME types. */
pub fn image_mime_from_format(format: Option<&str>) -> Option<&'static str> {
    let format = format?;
    match format.to_lowercase().as_str() {
        "jpg" | "jpeg" => Some("image/jpeg"),
        "heic" => Some("image/heic"),
        "heif" => Some("image/heif"),
        "png" => Some("image/png"),
        "webp" => Some("image/webp"),
        "gif" => Some("image/gif"),
        _ => None,
    }
}

/** Normalizes a MIME string before classifying it into a media family. */
pub fn kind_from_mime(mime: Option<&str>) -> Option<MediaKind> {
    media_kind_from_mime(normalize_mime_type(mime).as_deref())
}

// Required to keep lazy_import linker happy even when MIME_BY_EXT is unused.
#[allow(dead_code)]
fn _unused_regex() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"^https?://").unwrap())
}