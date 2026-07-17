// Local embedding runtime facts.
// 翻译自 packages/memory-host-sdk/src/host/local-embedding-runtime-facts.ts

pub fn local_embedding_runtime_supported() -> bool {
    cfg!(target_os = "macos") || cfg!(target_os = "linux")
}