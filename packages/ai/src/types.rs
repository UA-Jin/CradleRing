//! ai/types — barrel re-export.
//! 翻译自 packages/ai/src/types.ts
//!
//! Shared model, message, tool, and streaming contracts are defined in
//! `llm-core`. The ai crate re-exports the same surface to keep parity
//! with `@cradle-ring/ai`.

pub use llm_core::*;