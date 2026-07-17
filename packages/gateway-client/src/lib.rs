//! gateway-client crate
//! 翻译自 packages/gateway-client/src/index.ts (barrel) + sibling modules.
//!
//! Public gateway-client package surface: connection client, device auth,
//! readiness helpers, event-loop readiness, and timeout utilities.

pub mod client;
pub mod device_auth;
pub mod event_loop_ready;
pub mod readiness;
pub mod timeouts;

pub use client::*;
pub use device_auth::*;
pub use event_loop_ready::*;
pub use readiness::*;
pub use timeouts::*;