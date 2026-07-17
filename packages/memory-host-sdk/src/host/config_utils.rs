// Config utilities.
// 翻译自 packages/memory-host-sdk/src/host/config-utils.ts

pub fn resolve_config_path() -> std::path::PathBuf {
    std::path::PathBuf::from(".cradle-ring/config.json")
}