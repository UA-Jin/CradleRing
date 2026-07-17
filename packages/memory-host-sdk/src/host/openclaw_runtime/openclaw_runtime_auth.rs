// CradleRing runtime auth helper.
// 翻译自 packages/memory-host-sdk/src/host/openclaw-runtime-auth.ts

pub fn read_token_from_env() -> Option<String> {
    std::env::var("CRADLE_RING_TOKEN").ok()
}