// Public barrel for shared coercion and normalization helpers.
// 翻译自 packages/normalization-core/src/index.ts

pub mod boolean_coercion;
pub mod error_coercion;
pub mod expect;
pub mod format;
pub mod json_coercion;
pub mod number_coercion;
pub mod record_coerce;
pub mod string_coerce;
pub mod string_normalization;
pub mod text_decoding;
pub mod utf16_slice;

pub use boolean_coercion::*;
pub use error_coercion::*;
pub use expect::*;
pub use format::*;
pub use json_coercion::*;
pub use number_coercion::*;
pub use record_coerce::*;
pub use string_coerce::*;
pub use string_normalization::*;
pub use text_decoding::*;
pub use utf16_slice::*;
