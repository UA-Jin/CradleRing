// Media Core module implements file name behavior.
// 翻译自 packages/media-core/src/file-name.ts

use std::path::Path;

/** Returns the final filename segment for either POSIX or Windows-style paths. */
pub fn basename_from_any_path(value: &str) -> String {
    // Apply POSIX basename twice (path.posix and path.win32 stripping for mixed separators),
    // matching the original TypeScript behavior where path.win32.basename(path.posix.basename(value))
    // handles both separator styles.
    let posix_basename = Path::new(value.replace('\\', "/").as_str())
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_string();
    // Windows basename treats both / and \ as separators; we emulate that on the POSIX basename.
    posix_basename
        .rsplit(|c| c == '/' || c == '\\')
        .next()
        .unwrap_or("")
        .to_string()
}

/** Returns the extension from the final filename segment of any path flavor. */
pub fn extname_from_any_path(value: &str) -> String {
    let base = basename_from_any_path(value);
    match base.rfind('.') {
        Some(pos) => base[pos..].to_string(),
        None => String::new(),
    }
}

/** Returns the extensionless filename from the final segment of any path flavor. */
pub fn name_from_any_path(value: &str) -> String {
    let base = basename_from_any_path(value);
    let ext = match base.rfind('.') {
        Some(pos) => base[pos..].to_string(),
        None => String::new(),
    };
    if ext.is_empty() {
        base
    } else {
        base[..base.len() - ext.len()].to_string()
    }
}