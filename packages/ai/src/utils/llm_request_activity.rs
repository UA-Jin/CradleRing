//! LLM request activity tracker.
//! 翻译自 packages/ai/src/utils/llm-request-activity.ts
//!
//! Lightweight in-memory accumulator for in-flight and completed request
//! counts. Useful for diagnostics dashboards and metrics.

use std::sync::atomic::{AtomicI64, Ordering};

#[derive(Debug, Default)]
pub struct LlmRequestActivity {
    in_flight: AtomicI64,
    completed: AtomicI64,
    failed: AtomicI64,
}

impl LlmRequestActivity {
    /// Create a new activity tracker.
    pub fn new() -> Self {
        Self::default()
    }

    /// Mark one request as started.
    pub fn start(&self) {
        self.in_flight.fetch_add(1, Ordering::SeqCst);
    }

    /// Mark one request as completed successfully.
    pub fn complete(&self) {
        self.in_flight.fetch_sub(1, Ordering::SeqCst);
        self.completed.fetch_add(1, Ordering::SeqCst);
    }

    /// Mark one request as failed.
    pub fn fail(&self) {
        self.in_flight.fetch_sub(1, Ordering::SeqCst);
        self.failed.fetch_add(1, Ordering::SeqCst);
    }

    /// Snapshot of the current activity counts.
    pub fn snapshot(&self) -> LlmRequestActivitySnapshot {
        LlmRequestActivitySnapshot {
            in_flight: self.in_flight.load(Ordering::SeqCst),
            completed: self.completed.load(Ordering::SeqCst),
            failed: self.failed.load(Ordering::SeqCst),
        }
    }
}

/// Snapshot of activity counters.
#[derive(Debug, Clone, Copy, Default, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LlmRequestActivitySnapshot {
    pub in_flight: i64,
    pub completed: i64,
    pub failed: i64,
}