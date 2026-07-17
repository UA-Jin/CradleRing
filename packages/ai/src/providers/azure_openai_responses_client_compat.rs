//! Azure OpenAI Responses client compatibility helpers.
//! 翻译自 packages/ai/src/providers/azure-openai-responses-client-compat.ts

/// Returns true when `hostname` looks like a traditional Azure OpenAI endpoint.
pub fn is_traditional_azure_openai_host(hostname: &str) -> bool {
    hostname.ends_with(".openai.azure.com") || hostname.ends_with(".cognitiveservices.azure.com")
}

/// Returns true when the base URL targets an OpenAI-compatible Azure Responses endpoint.
pub fn is_openai_compatible_azure_responses_base_url(base_url: &str) -> bool {
    let Ok(url) = url_parse(base_url) else {
        return false;
    };
    if is_traditional_azure_openai_host(&url.host) {
        return false;
    }
    let hostname = url.host.to_lowercase();
    let is_foundry_host = hostname.ends_with(".services.ai.azure.com")
        || hostname.ends_with(".api.cognitive.microsoft.com");
    if !is_foundry_host {
        return false;
    }
    let normalized_path = url.path.trim_end_matches('/').to_string();
    normalized_path == "/openai/v1" || normalized_path.ends_with("/openai/v1")
}

#[derive(Debug, Clone, Default)]
struct ParsedUrl {
    host: String,
    path: String,
}

/// Minimal URL parser for compatibility detection. We avoid pulling in the
/// `url` crate; the relevant fields are host + path.
fn url_parse(input: &str) -> Result<ParsedUrl, ()> {
    let rest = input
        .strip_prefix("https://")
        .or_else(|| input.strip_prefix("http://"))
        .ok_or(())?;
    let (authority, path) = match rest.find('/') {
        Some(idx) => (&rest[..idx], rest[idx..].to_string()),
        None => (rest, "/".to_string()),
    };
    let host = authority.split(':').next().unwrap_or(authority).to_string();
    Ok(ParsedUrl {
        host,
        path: if path.is_empty() { "/".to_string() } else { path },
    })
}