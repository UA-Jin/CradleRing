//! Environment-based API key resolver.
//! 翻译自 packages/ai/src/env-api-keys.ts
//!
//! Mirrors TS semantics for finding the env-var name(s) that hold an API
//! key for a given provider. In Rust we directly read `std::env` and the
//! filesystem; no dynamic import dance is required.

use std::collections::HashMap;
use std::path::PathBuf;

use once_cell::sync::Lazy;

use crate::host::get_ai_transport_host;

/// Maps provider id -> ordered list of env-var candidates that may hold an API key.
fn get_api_key_env_vars(provider: &str) -> Option<Vec<&'static str>> {
    if provider == "github-copilot" {
        return Some(vec!["COPILOT_GITHUB_TOKEN"]);
    }
    if provider == "anthropic" {
        // ANTHROPIC_OAUTH_TOKEN takes precedence over ANTHROPIC_API_KEY.
        return Some(vec!["ANTHROPIC_OAUTH_TOKEN", "ANTHROPIC_API_KEY"]);
    }
    if provider == "moonshot" {
        return Some(vec!["MOONSHOT_API_KEY", "KIMI_API_KEY"]);
    }
    if provider == "kimi" || provider == "kimi-coding" {
        return Some(vec!["KIMI_API_KEY", "KIMICODE_API_KEY"]);
    }

    let env_map: HashMap<&str, &str> = [
        ("openai", "OPENAI_API_KEY"),
        ("meta", "MODEL_API_KEY"),
        ("azure-openai-responses", "AZURE_OPENAI_API_KEY"),
        ("deepseek", "DEEPSEEK_API_KEY"),
        ("google", "GEMINI_API_KEY"),
        ("google-vertex", "GOOGLE_CLOUD_API_KEY"),
        ("groq", "GROQ_API_KEY"),
        ("cerebras", "CEREBRAS_API_KEY"),
        ("xai", "XAI_API_KEY"),
        ("openrouter", "OPENROUTER_API_KEY"),
        ("vercel-ai-gateway", "AI_GATEWAY_API_KEY"),
        ("zai", "ZAI_API_KEY"),
        ("mistral", "MISTRAL_API_KEY"),
        ("minimax", "MINIMAX_API_KEY"),
        ("minimax-cn", "MINIMAX_CN_API_KEY"),
        ("moonshotai", "MOONSHOT_API_KEY"),
        ("moonshotai-cn", "MOONSHOT_API_KEY"),
        ("huggingface", "HF_TOKEN"),
        ("fireworks", "FIREWORKS_API_KEY"),
        ("together", "TOGETHER_API_KEY"),
        ("opencode", "OPENCODE_API_KEY"),
        ("opencode-go", "OPENCODE_API_KEY"),
        ("cloudflare-workers-ai", "CLOUDFLARE_API_KEY"),
        ("cloudflare-ai-gateway", "CLOUDFLARE_API_KEY"),
        ("xiaomi", "XIAOMI_API_KEY"),
        ("xiaomi-token-plan-cn", "XIAOMI_TOKEN_PLAN_CN_API_KEY"),
        ("xiaomi-token-plan-ams", "XIAOMI_TOKEN_PLAN_AMS_API_KEY"),
        ("xiaomi-token-plan-sgp", "XIAOMI_TOKEN_PLAN_SGP_API_KEY"),
    ]
    .iter()
    .copied()
    .collect();

    env_map.get(provider).map(|v| vec![*v])
}

/// Resolve the value of an environment variable.
fn get_env_value(key: &str) -> Option<String> {
    let _ = get_ai_transport_host; // reserved for host-side sentinel resolution parity
    std::env::var(key).ok()
}

/// Fallback for environments where `process.env` is empty (e.g. some Bun sandboxes).
fn get_proc_env(key: &str) -> Option<String> {
    static CACHE: Lazy<Option<HashMap<String, String>>> = Lazy::new(|| {
        let content = std::fs::read_to_string("/proc/self/environ").ok()?;
        Some(
            content
                .split('\0')
                .filter_map(|entry| {
                    let idx = entry.find('=')?;
                    Some((entry[..idx].to_string(), entry[idx + 1..].to_string()))
                })
                .collect(),
        )
    });

    if let Some(cache) = CACHE.as_ref() {
        return cache.get(key).cloned();
    }
    None
}

/// Returns the env var value, falling back to /proc/self/environ.
fn get_env_value_with_proc(key: &str) -> Option<String> {
    get_env_value(key).or_else(|| get_proc_env(key))
}

/// Checks whether Vertex AI Application Default Credentials are present on disk.
fn has_vertex_adc_credentials() -> bool {
    let gac_path = get_env_value_with_proc("GOOGLE_APPLICATION_CREDENTIALS");
    if let Some(path) = gac_path {
        return std::path::Path::new(&path).exists();
    }
    let home = std::env::var_os("HOME").map(PathBuf::from).unwrap_or_default();
    let default_path = home
        .join(".config")
        .join("gcloud")
        .join("application_default_credentials.json");
    default_path.exists()
}

/// Find configured environment variables that can provide an API key for a provider.
pub fn find_env_keys(provider: &str) -> Option<Vec<String>> {
    let env_vars = get_api_key_env_vars(provider)?;
    let found: Vec<String> = env_vars
        .iter()
        .filter_map(|v| get_env_value_with_proc(v).map(|_| v.to_string()))
        .collect();
    if found.is_empty() {
        None
    } else {
        Some(found)
    }
}

/// Get API key for provider from known environment variables.
pub fn get_env_api_key(provider: &str) -> Option<String> {
    let env_keys = find_env_keys(provider);
    if let Some(keys) = env_keys {
        if let Some(first) = keys.first() {
            if let Some(value) = get_env_value_with_proc(first) {
                return Some(value);
            }
        }
    }

    if provider == "google-vertex" {
        let has_credentials = has_vertex_adc_credentials();
        let has_project =
            get_env_value_with_proc("GOOGLE_CLOUD_PROJECT").is_some()
                || get_env_value_with_proc("GCLOUD_PROJECT").is_some();
        let has_location = get_env_value_with_proc("GOOGLE_CLOUD_LOCATION").is_some();
        if has_credentials && has_project && has_location {
            return Some("<authenticated>".to_string());
        }
    }

    if provider == "amazon-bedrock" {
        if get_env_value_with_proc("AWS_PROFILE").is_some()
            || (get_env_value_with_proc("AWS_ACCESS_KEY_ID").is_some()
                && get_env_value_with_proc("AWS_SECRET_ACCESS_KEY").is_some())
            || get_env_value_with_proc("AWS_BEARER_TOKEN_BEDROCK").is_some()
            || get_env_value_with_proc("AWS_CONTAINER_CREDENTIALS_RELATIVE_URI").is_some()
            || get_env_value_with_proc("AWS_CONTAINER_CREDENTIALS_FULL_URI").is_some()
            || get_env_value_with_proc("AWS_WEB_IDENTITY_TOKEN_FILE").is_some()
        {
            return Some("<authenticated>".to_string());
        }
    }

    None
}