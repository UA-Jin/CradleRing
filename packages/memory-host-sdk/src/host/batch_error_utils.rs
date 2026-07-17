// Batch error utils.
// 翻译自 packages/memory-host-sdk/src/host/batch-error-utils.ts

pub fn is_batch_error(error: &str) -> bool {
    error.contains("batch") || error.contains("rate limit")
}