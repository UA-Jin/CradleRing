//! Streaming byte guard.
//! 翻译自 packages/ai/src/utils/streaming-byte-guard.ts
//!
//! Prevents runaway responses from consuming unbounded memory by capping
//! total observed bytes and surfacing an error when the cap is hit.

use std::sync::atomic::{AtomicUsize, Ordering};

/// Guard that tracks observed bytes and enforces a soft cap.
pub struct StreamingByteGuard {
    cap: usize,
    observed: AtomicUsize,
    exceeded: std::sync::atomic::AtomicBool,
}

impl StreamingByteGuard {
    /// Create a new guard with the given cap (in bytes).
    pub fn new(cap: usize) -> Self {
        Self {
            cap,
            observed: AtomicUsize::new(0),
            exceeded: std::sync::atomic::AtomicBool::new(false),
        }
    }

    /// Record `n` bytes as observed. Returns `true` if the cap is exceeded.
    pub fn observe(&self, n: usize) -> bool {
        let prev = self.observed.fetch_add(n, Ordering::SeqCst);
        if prev + n > self.cap {
            self.exceeded.store(true, Ordering::SeqCst);
            true
        } else {
            false
        }
    }

    /// Returns total bytes observed so far.
    pub fn total(&self) -> usize {
        self.observed.load(Ordering::SeqCst)
    }

    /// Returns the configured cap.
    pub fn cap(&self) -> usize {
        self.cap
    }

    /// Returns true if the guard has been exceeded.
    pub fn exceeded(&self) -> bool {
        self.exceeded.load(Ordering::SeqCst)
    }

    /// Returns true if the guard is still within the cap.
    pub fn ok(&self) -> bool {
        !self.exceeded()
    }
}