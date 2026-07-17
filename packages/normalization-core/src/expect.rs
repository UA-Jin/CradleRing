// expect helpers.
// 翻译自 packages/normalization-core/src/expect.ts

use crate::error_coercion::CrError;

/// Returns the value or throws with the named context; use for genuine invariants only.
pub fn expect_defined<T>(value: Option<T>, context: &str) -> Result<T, CrError> {
    value.ok_or_else(|| CrError::new(format!("expected {} to be defined", context)))
}

/// First element with honest optionality; callers own the absent case.
pub fn first<T: Clone>(values: &[T]) -> Option<T> {
    values.first().cloned()
}

/// Last element with honest optionality; callers own the absent case.
pub fn last<T: Clone>(values: &[T]) -> Option<T> {
    values.last().cloned()
}
