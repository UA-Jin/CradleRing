// Terminal Core module implements prompt select styled behavior.
// 翻译自 packages/terminal-core/src/prompt-select-styled.ts

use crate::prompt_select_styled_params::style_select_params;

/// Run a clack select prompt with styled message and hints.
///
/// In CradleRing we expose the styling primitive; interactive prompt rendering
/// is delegated to host-side callers that wire up their own UI runtime.
pub fn select_styled<T, M, O>(
    message: M,
    options: Vec<O>,
    next: impl Fn(String, Vec<(O, Option<String>)>) -> T,
) -> T
where
    M: AsRef<str>,
    O: Clone,
{
    style_select_params::<T, M, O>(message, options, None, next)
}
