// Tracks the active terminal progress line so callers can clear it before other output.
// 翻译自 packages/terminal-core/src/progress-line.ts

use std::sync::Mutex;

static ACTIVE_STREAM: Mutex<Option<u64>> = Mutex::new(None);

fn stream_handle() -> u64 {
    use std::io::IsTerminal;
    let stdout = std::io::stdout();
    let is_tty = stdout.is_terminal();
    let bytes = format!("stdout:tty={}", is_tty);
    // Use the address of a static to make a stable handle per identity.
    // We use stdout's lock reference; here we approximate with a per-state hash.
    let mut state = std::collections::hash_map::DefaultHasher::new();
    use std::hash::{Hash, Hasher};
    bytes.hash(&mut state);
    state.finish()
}

/// Register the stream that currently owns an inline progress line.
pub fn register_active_progress_line() {
    if !stdout_is_tty() {
        return;
    }
    let mut g = ACTIVE_STREAM.lock().unwrap();
    *g = Some(stream_handle());
}

/// Clear the active progress line when it is attached to a TTY stream.
pub fn clear_active_progress_line() {
    if !stdout_is_tty() {
        return;
    }
    let g = ACTIVE_STREAM.lock().unwrap();
    if g.is_some() {
        // Emit `\r\x1b[2K` to stdout.
        print!("\r\u{001B}[2K");
    }
}

/// Unregister the active progress line, optionally only for a matching stream.
pub fn unregister_active_progress_line(_matching: bool) {
    let mut g = ACTIVE_STREAM.lock().unwrap();
    *g = None;
}

fn stdout_is_tty() -> bool {
    use std::io::IsTerminal;
    std::io::stdout().is_terminal()
}
