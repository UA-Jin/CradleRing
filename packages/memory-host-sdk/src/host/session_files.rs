// Session files helper.
// 翻译自 packages/memory-host-sdk/src/host/session-files.ts

pub fn session_files_dir(_session_key: &str) -> std::path::PathBuf {
    std::path::PathBuf::from(".cradle-ring/sessions")
}