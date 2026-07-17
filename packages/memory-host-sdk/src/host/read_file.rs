// Read memory file helper.
// 翻译自 packages/memory-host-sdk/src/host/read-file.ts

use std::path::Path;

use crate::host::read_file_shared::{build_memory_read_result, MemoryReadResult};

pub async fn read_memory_file(path: &Path) -> std::io::Result<MemoryReadResult> {
    let text = std::fs::read_to_string(path)?;
    Ok(build_memory_read_result(text, path.to_string_lossy().as_ref(), None, None))
}