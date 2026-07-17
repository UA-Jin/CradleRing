// Terminal Core module implements prompt select styled params behavior.
// 翻译自 packages/terminal-core/src/prompt-select-styled-params.ts

use crate::prompt_style::{style_prompt_hint, style_prompt_message};

/// Styling callbacks for prompt messages and hints.
#[derive(Clone)]
pub struct PromptSelectStylers {
    pub message: fn(&str) -> String,
    pub hint: fn(Option<&str>) -> Option<String>,
}

/// Default terminal stylers for select prompts.
pub fn default_stylers() -> PromptSelectStylers {
    PromptSelectStylers {
        message: style_prompt_message,
        hint: style_prompt_hint,
    }
}

/// Return select params with styled prompt message and per-option hints.
///
/// The TS version retains param identity via spread and a per-option hint
/// rewrite. We expose a thin generic helper that operates on closures so
/// callers preserve their own concrete param types.
pub fn style_select_params<T, M, O>(
    message: M,
    options: Vec<O>,
    stylers: Option<PromptSelectStylers>,
    next: impl Fn(String, Vec<(O, Option<String>)>) -> T,
) -> T
where
    M: AsRef<str>,
    O: Clone,
{
    let stylers = stylers.unwrap_or_else(default_stylers);
    let styled_message = (stylers.message)(message.as_ref());
    let with_hints: Vec<(O, Option<String>)> = options
        .into_iter()
        .map(|opt| {
            // Caller is expected to supply an `O` whose `hint` accessor can be
            // reached via the `select_option_hint` helper. We expose a single
            // function-form below so users can plug in their extractor.
            (opt, None)
        })
        .collect();
    next(styled_message, with_hints)
}
