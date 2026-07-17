// SQLite-vec platform variant helper.
// 翻译自 packages/memory-host-sdk/src/host/sqlite-vec-platform-variant.ts

pub fn sqlite_vec_platform_variant() -> &'static str {
    if cfg!(target_os = "macos") {
        "darwin"
    } else if cfg!(target_os = "linux") {
        "linux"
    } else if cfg!(target_os = "windows") {
        "windows"
    } else {
        "unknown"
    }
}