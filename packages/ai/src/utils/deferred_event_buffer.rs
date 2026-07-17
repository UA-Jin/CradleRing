//! Deferred event buffer for stream consumers that attach late.
//! 翻译自 packages/ai/src/utils/deferred-event-buffer.ts
//!
//! Holds stream events in memory until a downstream consumer attaches and
//! then flushes the buffered events in order.

use std::sync::Mutex;

use llm_core::types::AssistantMessageEvent;

/// Inner state for the deferred buffer.
struct Inner {
    events: Vec<AssistantMessageEvent>,
    closed: bool,
}

impl Default for Inner {
    fn default() -> Self {
        Self {
            events: Vec::new(),
            closed: false,
        }
    }
}

/// Buffers assistant-message events until they are drained.
#[derive(Default)]
pub struct DeferredEventBuffer {
    inner: Mutex<Inner>,
}

impl DeferredEventBuffer {
    /// Create a new empty buffer.
    pub fn new() -> Self {
        Self::default()
    }

    /// Push one event into the buffer.
    pub fn push(&self, event: AssistantMessageEvent) {
        let mut guard = self.inner.lock().expect("deferred buffer poisoned");
        guard.events.push(event);
    }

    /// Mark the buffer as closed (no more events will arrive).
    pub fn close(&self) {
        let mut guard = self.inner.lock().expect("deferred buffer poisoned");
        guard.closed = true;
    }

    /// Returns true when the buffer has been closed.
    pub fn is_closed(&self) -> bool {
        self.inner.lock().expect("deferred buffer poisoned").closed
    }

    /// Drain all buffered events, leaving the buffer empty.
    pub fn drain(&self) -> Vec<AssistantMessageEvent> {
        let mut guard = self.inner.lock().expect("deferred buffer poisoned");
        std::mem::take(&mut guard.events)
    }
}