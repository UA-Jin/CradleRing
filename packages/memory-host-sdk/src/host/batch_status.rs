// Batch status helper.
// 翻译自 packages/memory-host-sdk/src/host/batch-status.ts

#[derive(Debug, Clone, Default)]
pub struct BatchStatus {
    pub completed: i64,
    pub failed: i64,
    pub in_flight: i64,
}