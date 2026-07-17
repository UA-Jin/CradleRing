// Batch utils.
// 翻译自 packages/memory-host-sdk/src/host/batch-utils.ts

pub fn batch_size(items: &[serde_json::Value]) -> usize {
    items.len()
}