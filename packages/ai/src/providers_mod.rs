//! ai/providers — barrel re-export.
//! 翻译自 packages/ai/src/providers.ts
//!
//! Lazy built-in protocol adapter registration helpers. The actual
//! provider registration logic lives in `providers/register_builtins.rs`.

pub use crate::providers::register_builtins::*;