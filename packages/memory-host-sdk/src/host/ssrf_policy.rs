// SSRF policy helper.
// 翻译自 packages/memory-host-sdk/src/host/ssrf-policy.ts

pub fn is_ssrf_safe_host(host: &str) -> bool {
    let lowered = host.to_lowercase();
    !(lowered == "localhost" || lowered == "127.0.0.1" || lowered == "::1" || lowered.starts_with("169.254."))
}