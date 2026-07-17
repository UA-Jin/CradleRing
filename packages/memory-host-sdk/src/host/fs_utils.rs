// Filesystem helpers.
// 翻译自 packages/memory-host-sdk/src/host/fs-utils.ts

use std::path::Path;

pub fn is_file_missing_error(error: &std::io::Error) -> bool {
    error.kind() == std::io::ErrorKind::NotFound
}

pub fn stat_regular_file(path: &Path) -> std::io::Result<std::fs::Metadata> {
    let meta = std::fs::metadata(path)?;
    if !meta.is_file() {
        return Err(std::io::Error::new(std::io::ErrorKind::Other, "not a regular file"));
    }
    Ok(meta)
}