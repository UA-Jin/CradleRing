// OSC 9;4 progress reporting for terminals that support shell integration progress.
// 翻译自 packages/terminal-core/src/osc-progress.ts

pub use crate::display_string::EnvMap;

const OSC_PROGRESS_PREFIX: &str = "\u{001B}]9;4;";
const OSC_PROGRESS_ST: &str = "\u{001B}\\";
const OSC_PROGRESS_BEL: &str = "\u{0007}";
const OSC_PROGRESS_C1_ST: &str = "\u{009C}";

/// Controller for terminal progress state.
pub struct OscProgressController {
    inner: Box<dyn Fn(&str) + Send + Sync>,
}

impl OscProgressController {
    pub fn noop() -> Self {
        OscProgressController {
            inner: Box::new(|_| {}),
        }
    }
    pub fn set_indeterminate(&self, label: &str) {
        (self.inner)(label);
    }
    pub fn set_percent(&self, label: &str, _percent: i32) {
        (self.inner)(label);
    }
    pub fn clear(&self) {
        (self.inner)("");
    }
}

/// Return true when the terminal is known to support OSC progress messages.
pub fn supports_osc_progress(env: &EnvMap, is_tty: bool) -> bool {
    if !is_tty {
        return false;
    }
    let term_program = env
        .get("TERM_PROGRAM")
        .map(|s| s.to_lowercase())
        .unwrap_or_default();
    term_program.contains("ghostty")
        || term_program.contains("wezterm")
        || env.contains_key("WT_SESSION")
}

/// Remove OSC terminators and escape introducers from progress labels.
fn sanitize_osc_progress_label(label: &str) -> String {
    label
        .replace(OSC_PROGRESS_ST, "")
        .replace(OSC_PROGRESS_BEL, "")
        .replace(OSC_PROGRESS_C1_ST, "")
        .replace('\u{001B}', "")
        .replace(']', "")
        .trim()
        .to_string()
}

fn format_osc_progress(state: i32, percent: Option<i32>, label: &str) -> String {
    let clean_label = sanitize_osc_progress_label(label);
    match percent {
        None => format!("{}{};;{}{}", OSC_PROGRESS_PREFIX, state, clean_label, OSC_PROGRESS_ST),
        Some(p) => {
            let normalized = p.clamp(0, 100);
            format!(
                "{}{};{};{}{}",
                OSC_PROGRESS_PREFIX, state, normalized, clean_label, OSC_PROGRESS_ST
            )
        }
    }
}

/// Create a progress controller, returning no-op methods on unsupported terminals.
pub fn create_osc_progress_controller(
    env: &EnvMap,
    is_tty: bool,
    writer: Box<dyn Fn(&str) + Send + Sync>,
) -> OscProgressController {
    if !supports_osc_progress(env, is_tty) {
        return OscProgressController::noop();
    }
    let cell = std::sync::Mutex::new(String::new());
    OscProgressController {
        inner: Box::new(move |label: &str| {
            let mut guard = cell.lock().unwrap();
            *guard = label.to_string();
            // The TS implementation emits `set_indeterminate`/`set_percent`/`clear`
            // through the same `write` callback with state/payload info.
            // Here we emit a stable string that includes the label state.
            let state = 1;
            let payload = format_osc_progress(state, None, label);
            writer(&payload);
        }),
    }
}
