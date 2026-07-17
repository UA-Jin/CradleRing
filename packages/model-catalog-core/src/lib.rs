//! model-catalog-core crate
//! 翻译自 packages/model-catalog-core/src/index.ts (barrel)
//!
//! Provides model catalog normalization, refs, types, and provider/model id
//! normalization policies used by both provider plugins and runtime lookup.

pub mod configured_model_refs;
pub mod model_catalog_normalize;
pub mod model_catalog_refs;
pub mod model_catalog_types;
pub mod provider_id;
pub mod provider_model_id_normalization;
pub mod provider_model_id_normalize;

pub use configured_model_refs::*;
pub use model_catalog_normalize::*;
pub use model_catalog_refs::*;
pub use model_catalog_types::*;
pub use provider_id::*;
pub use provider_model_id_normalization::*;
pub use provider_model_id_normalize::*;