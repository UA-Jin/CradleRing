//! Anthropic auth headers helpers.
//! 翻译自 packages/ai/src/providers/anthropic-auth-headers.ts

use std::collections::BTreeMap;

use once_cell::sync::Lazy;
use regex::Regex;

use llm_core::types::Model;

/// Minimal model shape needed by these helpers.
pub trait AnthropicAuthModel {
    fn provider(&self) -> &str;
    fn auth_header(&self) -> Option<bool>;
    fn headers(&self) -> Option<&BTreeMap<String, String>>;
}

impl AnthropicAuthModel for Model {
    fn provider(&self) -> &str {
        &self.provider
    }
    fn auth_header(&self) -> Option<bool> {
        self.auth_header
    }
    fn headers(&self) -> Option<&BTreeMap<String, String>> {
        self.headers.as_ref()
    }
}

fn has_bearer_authorization_header(headers: Option<&BTreeMap<String, String>>) -> bool {
    static BEARER_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?i)^bearer\s+\S+").unwrap());
    let Some(headers) = headers else {
        return false;
    };
    headers.iter().any(|(k, v)| {
        k.to_lowercase() == "authorization" && BEARER_RE.is_match(v.trim())
    })
}

/// Returns true when the model uses Foundry bearer auth.
pub fn uses_foundry_bearer_auth(model: &Model) -> bool {
    model.provider == "microsoft-foundry"
        && (model.auth_header == Some(true) || has_bearer_authorization_header(model.headers.as_ref()))
}

/// Strips Foundry bearer credentials from the headers map.
pub fn omit_foundry_bearer_credential_headers(
    headers: Option<&BTreeMap<String, String>>,
) -> Option<BTreeMap<String, String>> {
    let headers = headers?;
    let mut next: BTreeMap<String, String> = BTreeMap::new();
    for (key, value) in headers {
        let lower = key.to_lowercase();
        if lower == "authorization" || lower == "x-api-key" || lower == "api-key" {
            continue;
        }
        next.insert(key.clone(), value.clone());
    }
    if next.is_empty() {
        None
    } else {
        Some(next)
    }
}