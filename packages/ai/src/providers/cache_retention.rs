//! Cache retention preference resolver.
//! 翻译自 packages/ai/src/providers/cache-retention.ts

use llm_core::types::CacheRetention;

/// Resolve cache retention preference. Defaults to "short".
pub fn resolve_cache_retention(cache_retention: Option<&CacheRetention>) -> CacheRetention {
    if let Some(c) = cache_retention {
        if !c.is_empty() {
            return c.clone();
        }
    }
    if std::env::var("OPENCLAW_CACHE_RETENTION").as_deref() == Ok("long") {
        return "long".to_string();
    }
    "short".to_string()
}