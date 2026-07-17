//! Cloudflare provider metadata.
//! 翻译自 packages/ai/src/providers/cloudflare.ts

use llm_core::types::Model;
use once_cell::sync::Lazy;
use regex::Regex;

static PLACEHOLDER_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"\{([A-Z_][A-Z0-9_]*)\}").unwrap());

/// Returns true if `provider` is a Cloudflare-hosted provider.
pub fn is_cloudflare_provider(provider: &str) -> bool {
    provider == "cloudflare-workers-ai" || provider == "cloudflare-ai-gateway"
}

/// Substitutes `{VAR}` placeholders in a Cloudflare base URL from env.
pub fn resolve_cloudflare_base_url(model: &Model) -> Result<String, String> {
    let url = &model.base_url;
    if !url.contains('{') {
        return Ok(url.clone());
    }
    let mut err: Option<String> = None;
    let replaced = PLACEHOLDER_RE.replace_all(url, |caps: &regex::Captures<'_>| {
        let name = caps.get(1).map(|m| m.as_str()).unwrap_or("");
        match std::env::var(name) {
            Ok(value) => value,
            Err(_) => {
                err = Some(format!(
                    "{} is required for provider {} but is not set.",
                    name, model.provider
                ));
                String::new()
            }
        }
    });
    if let Some(e) = err {
        Err(e)
    } else {
        Ok(replaced.into_owned())
    }
}