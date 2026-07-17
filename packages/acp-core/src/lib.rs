//! acp-core crate
//! 翻译自 packages/acp-core/src/index.ts (barrel) + sibling modules.
//!
//! Provides shared ACP session, metadata, and runtime helper contracts.

pub mod error_format;
pub mod meta;
pub mod normalize_text;
pub mod numeric_options;
pub mod record_shared;
pub mod runtime;
pub mod session;
pub mod session_interaction_mode;
pub mod session_lineage_meta;
pub mod types;

pub use error_format::*;
pub use meta::*;
pub use normalize_text::*;
pub use numeric_options::*;
pub use record_shared::*;
pub use runtime::*;
pub use session::*;
pub use session_interaction_mode::*;
pub use session_lineage_meta::*;
pub use types::*;