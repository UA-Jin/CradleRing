//! Session-resource cleanup registry.
//! 翻译自 packages/ai/src/session-resources.ts
//!
//! Process-local registry of cleanup hooks owned by LLM providers/transports.

use std::sync::RwLock;

/// Cleanup callback for resources tied to an LLM session or all sessions.
pub type SessionResourceCleanup = Box<dyn Fn(Option<&str>) + Send + Sync>;

type SharedCleanup = std::sync::Arc<SessionResourceCleanup>;

static SESSION_RESOURCE_CLEANUPS: once_cell::sync::Lazy<RwLock<Vec<SharedCleanup>>> =
    once_cell::sync::Lazy::new(|| RwLock::new(Vec::new()));

/// Registers a session-resource cleanup hook and returns an unregister function.
pub fn register_session_resource_cleanup<F>(cleanup: F) -> impl Fn() + Send + Sync
where
    F: Fn(Option<&str>) + Send + Sync + 'static,
{
    let shared: SharedCleanup = std::sync::Arc::new(Box::new(cleanup));
    SESSION_RESOURCE_CLEANUPS
        .write()
        .expect("session resources poisoned")
        .push(shared.clone());

    let unregister: Box<dyn Fn() + Send + Sync> = Box::new(move || {
        let mut guard = SESSION_RESOURCE_CLEANUPS
            .write()
            .expect("session resources poisoned");
        guard.retain(|c| !std::sync::Arc::ptr_eq(c, &shared));
    });
    unregister
}

/// Runs all registered cleanup hooks, aggregating failures after every hook has run.
pub fn cleanup_session_resources(session_id: Option<&str>) -> Result<(), String> {
    let cleanups: Vec<SharedCleanup> = SESSION_RESOURCE_CLEANUPS
        .read()
        .expect("session resources poisoned")
        .clone();
    let mut errors: Vec<String> = Vec::new();
    for cleanup in cleanups {
        let cleanup = cleanup.clone();
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            (cleanup)(session_id);
        }));
        if let Err(err) = result {
            if let Some(s) = err.downcast_ref::<&str>() {
                errors.push(s.to_string());
            } else if let Some(s) = err.downcast_ref::<String>() {
                errors.push(s.clone());
            } else {
                errors.push("unknown cleanup error".to_string());
            }
        }
    }
    if !errors.is_empty() {
        return Err(format!(
            "Failed to cleanup session resources: {}",
            errors.join("; ")
        ));
    }
    Ok(())
}