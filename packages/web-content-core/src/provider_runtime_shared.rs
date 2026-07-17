// Web Content Core module implements provider runtime shared behavior.
// 翻译自 packages/web-content-core/src/provider-runtime-shared.ts
use once_cell::sync::Lazy;
use regex::Regex;

#[derive(Debug, Clone, Default)]
pub struct WebProviderConfigSource {
    pub tools: Option<WebToolsConfig>,
}

#[derive(Debug, Clone, Default)]
pub struct WebToolsConfig {
    pub web: Option<WebToolConfig>,
}

#[derive(Debug, Clone, Default)]
pub struct WebToolConfig {
    pub search: Option<serde_json::Value>,
    pub fetch: Option<serde_json::Value>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SecretRefSource {
    Env,
    File,
    Exec,
}

impl SecretRefSource {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "env" => Some(SecretRefSource::Env),
            "file" => Some(SecretRefSource::File),
            "exec" => Some(SecretRefSource::Exec),
            _ => None,
        }
    }
    pub fn as_str(&self) -> &'static str {
        match self {
            SecretRefSource::Env => "env",
            SecretRefSource::File => "file",
            SecretRefSource::Exec => "exec",
        }
    }
}

#[derive(Debug, Clone)]
pub struct SecretRef {
    pub source: SecretRefSource,
    pub provider: String,
    pub id: String,
}

const DEFAULT_SECRET_PROVIDER_ALIAS: &str = "default";
const LEGACY_SECRETREF_ENV_MARKER_PREFIX: &str = "secretref-env:";
const LEGACY_DOUBLE_UNDERSCORE_ENV_MARKER_PREFIX: &str = "__env__:";

static ENV_SECRET_REF_ID_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^[A-Z][A-Z0-9_]{0,127}$").unwrap());

static ENV_SECRET_TEMPLATE_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^\$\{([A-Z][A-Z0-9_]{0,127})\}$").unwrap());

static ENV_SECRET_SHORTHAND_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^\$([A-Z][A-Z0-9_]{0,127})$").unwrap());

#[derive(Debug, Clone, Default)]
pub struct RuntimeWebProviderMetadata {
    pub provider_configured: Option<String>,
    pub selected_provider: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct ProviderWithCredential {
    pub env_vars: Vec<String>,
    pub auth_provider_id: Option<String>,
    pub requires_credential: Option<bool>,
}

pub type WebContentProcessEnv = std::collections::HashMap<String, String>;

fn is_record(value: &serde_json::Value) -> bool {
    value.is_object()
}

fn normalize_secret_input_string(value: &serde_json::Value) -> Option<String> {
    if let Some(s) = value.as_str() {
        let trimmed = s.trim();
        if !trimmed.is_empty() {
            return Some(trimmed.to_string());
        }
    }
    None
}

fn normalize_secret_input(value: &serde_json::Value) -> String {
    let s = match value.as_str() {
        Some(s) => s,
        None => return String::new(),
    };
    let collapsed: String = s
        .chars()
        .filter(|c| *c != '\r' && *c != '\n' && *c != '\u{2028}' && *c != '\u{2029}')
        .collect();
    let mut latin1_only = String::new();
    for c in collapsed.chars() {
        let cp = c as u32;
        let is_control = (cp <= 0x1f) || cp == 0x7f || (cp >= 0x80 && cp <= 0x9f);
        if cp <= 0xff && !is_control {
            latin1_only.push(c);
        }
    }
    latin1_only.trim().to_string()
}

fn is_secret_ref(value: &serde_json::Value) -> bool {
    let obj = match value.as_object() {
        Some(o) => o,
        None => return false,
    };
    if obj.len() != 3 {
        return false;
    }
    let source_ok = obj
        .get("source")
        .and_then(|v| v.as_str())
        .map(|s| matches!(s, "env" | "file" | "exec"))
        .unwrap_or(false);
    let provider_ok = obj
        .get("provider")
        .and_then(|v| v.as_str())
        .map(|s| !s.trim().is_empty())
        .unwrap_or(false);
    let id_ok = obj
        .get("id")
        .and_then(|v| v.as_str())
        .map(|s| !s.trim().is_empty())
        .unwrap_or(false);
    source_ok && provider_ok && id_ok
}

pub fn coerce_secret_ref(value: &serde_json::Value) -> Option<SecretRef> {
    if is_secret_ref(value) {
        let obj = value.as_object().unwrap();
        let source = obj.get("source").and_then(|v| v.as_str()).unwrap();
        let provider = obj.get("provider").and_then(|v| v.as_str()).unwrap().trim().to_string();
        let id = obj.get("id").and_then(|v| v.as_str()).unwrap().trim().to_string();
        return Some(SecretRef {
            source: SecretRefSource::from_str(source).unwrap(),
            provider,
            id,
        });
    }
    if let Some(s) = value.as_str() {
        let trimmed = s.trim();
        let legacy_prefix = if trimmed.starts_with(LEGACY_SECRETREF_ENV_MARKER_PREFIX) {
            Some(LEGACY_SECRETREF_ENV_MARKER_PREFIX)
        } else if trimmed.starts_with(LEGACY_DOUBLE_UNDERSCORE_ENV_MARKER_PREFIX) {
            Some(LEGACY_DOUBLE_UNDERSCORE_ENV_MARKER_PREFIX)
        } else {
            None
        };
        if let Some(prefix) = legacy_prefix {
            let id = &trimmed[prefix.len()..];
            if ENV_SECRET_REF_ID_RE.is_match(id) {
                return Some(SecretRef {
                    source: SecretRefSource::Env,
                    provider: DEFAULT_SECRET_PROVIDER_ALIAS.to_string(),
                    id: id.to_string(),
                });
            }
            return None;
        }
        let id = ENV_SECRET_TEMPLATE_RE
            .captures(trimmed)
            .or_else(|| ENV_SECRET_SHORTHAND_RE.captures(trimmed))
            .and_then(|c| c.get(1))
            .map(|m| m.as_str().to_string());
        if let Some(id) = id {
            return Some(SecretRef {
                source: SecretRefSource::Env,
                provider: DEFAULT_SECRET_PROVIDER_ALIAS.to_string(),
                id,
            });
        }
        return None;
    }
    if is_record(value) {
        let obj = value.as_object().unwrap();
        let source = obj.get("source").and_then(|v| v.as_str());
        if let Some(source) = source {
            if let Some(s) = SecretRefSource::from_str(source) {
                let id_ok = obj
                    .get("id")
                    .and_then(|v| v.as_str())
                    .map(|s| !s.trim().is_empty())
                    .unwrap_or(false);
                let provider_is_undefined = obj.get("provider").map(|v| v.is_null()).unwrap_or(true);
                if id_ok && provider_is_undefined {
                    let id = obj.get("id").and_then(|v| v.as_str()).unwrap().trim().to_string();
                    return Some(SecretRef {
                        source: s,
                        provider: DEFAULT_SECRET_PROVIDER_ALIAS.to_string(),
                        id,
                    });
                }
            }
        }
    }
    None
}

pub fn resolve_web_provider_config(
    cfg: Option<&WebProviderConfigSource>,
    kind: &str,
) -> Option<serde_json::Value> {
    let web_config = cfg?.tools.as_ref()?.web.as_ref()?;
    let tool_config = match kind {
        "search" => web_config.search.clone(),
        "fetch" => web_config.fetch.clone(),
        _ => None,
    }?;
    if !tool_config.is_object() {
        return None;
    }
    Some(tool_config)
}

pub fn read_web_provider_env_value(
    env_vars: &[String],
    process_env: &WebContentProcessEnv,
) -> Option<String> {
    for env_var in env_vars {
        if let Some(v) = process_env.get(env_var) {
            let normalized = normalize_secret_input(&serde_json::Value::String(v.clone()));
            if !normalized.is_empty() {
                return Some(normalized);
            }
        }
    }
    None
}

pub fn provider_requires_credential(provider: &ProviderWithCredential) -> bool {
    provider.requires_credential.unwrap_or(true)
}

pub struct HasWebProviderEntryCredentialParams<'a, TProvider, TConfigSource, TConfig> {
    pub provider: &'a TProvider,
    pub config: Option<&'a TConfigSource>,
    pub tool_config: Option<&'a TConfig>,
    pub resolve_raw_value: fn(&'a TProvider, Option<&'a TConfigSource>, Option<&'a TConfig>) -> serde_json::Value,
    pub resolve_fallback_raw_value:
        Option<fn(&'a TProvider, Option<&'a TConfigSource>, Option<&'a TConfig>) -> serde_json::Value>,
    pub resolve_env_value: fn(&'a TProvider, Option<&str>) -> Option<String>,
    pub resolve_provider_auth_value: Option<fn(&str) -> bool>,
}

pub fn has_web_provider_entry_credential<'a, TProvider, TConfigSource, TConfig>(
    params: HasWebProviderEntryCredentialParams<'a, TProvider, TConfigSource, TConfig>,
) -> bool
where
    TProvider: WithCredentialMarker,
{
    let provider = params.provider;
    if !provider_requires_credential(provider.as_credential()) {
        return true;
    }
    let raw_value = (params.resolve_raw_value)(provider, params.config, params.tool_config);
    let configured_ref = coerce_secret_ref(&raw_value);
    if let Some(ref r) = configured_ref {
        if r.source != SecretRefSource::Env {
            return true;
        }
    }
    let from_config = if configured_ref.is_some() {
        String::new()
    } else {
        normalize_secret_input(&normalize_secret_input_string(&raw_value).map(serde_json::Value::String).unwrap_or(serde_json::Value::Null))
    };
    if !from_config.is_empty() {
        return true;
    }
    if let Some(auth_id) = provider.auth_provider_id() {
        if let Some(f) = params.resolve_provider_auth_value {
            if f(auth_id) {
                return true;
            }
        }
    }
    let env_value = (params.resolve_env_value)(
        provider,
        configured_ref.as_ref().map(|r| r.id.as_str()),
    );
    if env_value.is_some() {
        return true;
    }
    let fallback_raw_value = if let Some(f) = params.resolve_fallback_raw_value {
        f(provider, params.config, params.tool_config)
    } else {
        serde_json::Value::Null
    };
    let fallback_ref = coerce_secret_ref(&fallback_raw_value);
    if let Some(ref r) = fallback_ref {
        if r.source != SecretRefSource::Env {
            return true;
        }
    }
    let fallback_config = if fallback_ref.is_some() {
        String::new()
    } else {
        normalize_secret_input(
            &normalize_secret_input_string(&fallback_raw_value)
                .map(serde_json::Value::String)
                .unwrap_or(serde_json::Value::Null),
        )
    };
    if !fallback_config.is_empty() {
        return true;
    }
    if let Some(ref r) = fallback_ref {
        if r.source == SecretRefSource::Env {
            if (params.resolve_env_value)(provider, Some(&r.id)).is_some() {
                return true;
            }
        }
    }
    false
}

pub trait WithCredentialMarker {
    fn requires_credential(&self) -> Option<bool>;
    fn auth_provider_id(&self) -> Option<&str>;
    fn as_credential(&self) -> &ProviderWithCredential;
}

impl WithCredentialMarker for ProviderWithCredential {
    fn requires_credential(&self) -> Option<bool> {
        self.requires_credential
    }
    fn auth_provider_id(&self) -> Option<&str> {
        self.auth_provider_id.as_deref()
    }
    fn as_credential(&self) -> &ProviderWithCredential {
        self
    }
}

pub struct ResolveWebProviderDefinitionParams<'a, TProvider, TConfigSource, TConfig, TRuntimeMetadata, TDefinition> {
    pub config: Option<&'a TConfigSource>,
    pub tool_config: Option<&'a TConfig>,
    pub runtime_metadata: Option<&'a TRuntimeMetadata>,
    pub sandboxed: Option<bool>,
    pub provider_id: Option<String>,
    pub providers: &'a [TProvider],
    pub resolve_enabled: fn(tool_config: Option<&'a TConfig>, sandboxed: Option<bool>) -> bool,
    pub resolve_auto_provider_id: fn(
        config: Option<&'a TConfigSource>,
        tool_config: Option<&'a TConfig>,
        providers: &'a [TProvider],
    ) -> Option<String>,
    pub resolve_fallback_provider_id: Option<
        fn(
            config: Option<&'a TConfigSource>,
            tool_config: Option<&'a TConfig>,
            providers: &'a [TProvider],
            provider_id: &str,
        ) -> Option<String>,
    >,
    pub create_tool: fn(
        &'a TProvider,
        Option<&'a TConfigSource>,
        Option<&'a TConfig>,
        Option<&'a TRuntimeMetadata>,
    ) -> Option<TDefinition>,
}

pub struct ResolvedWebProvider<'a, TProvider, TDefinition> {
    pub provider: &'a TProvider,
    pub definition: TDefinition,
}

pub fn resolve_web_provider_definition<
    'a,
    TProvider,
    TConfigSource,
    TConfig,
    TRuntimeMetadata,
    TDefinition,
>(
    params: ResolveWebProviderDefinitionParams<
        'a,
        TProvider,
        TConfigSource,
        TConfig,
        TRuntimeMetadata,
        TDefinition,
    >,
) -> Option<ResolvedWebProvider<'a, TProvider, TDefinition>>
where
    TProvider: HasId,
    TRuntimeMetadata: RuntimeMetadataLike,
{
    let enabled = (params.resolve_enabled)(params.tool_config, params.sandboxed);
    if !enabled {
        return None;
    }
    let providers: Vec<&TProvider> = params.providers.iter().filter(|p| p.id().is_some()).collect();
    if providers.is_empty() {
        return None;
    }
    let auto_provider_id = (params.resolve_auto_provider_id)(
        params.config,
        params.tool_config,
        params.providers,
    );
    let provider_id_str = params
        .provider_id
        .clone()
        .or_else(|| {
            params
                .runtime_metadata
                .and_then(|m| m.selected_provider())
        })
        .or(auto_provider_id);
    let provider_id = match provider_id_str {
        Some(s) => s,
        None => return None,
    };
    let provider = providers
        .iter()
        .copied()
        .find(|p| p.id() == Some(provider_id.as_str()))
        .or_else(|| {
            let fallback_id = params.resolve_fallback_provider_id.and_then(|f| {
                f(params.config, params.tool_config, params.providers, &provider_id)
            });
            match fallback_id {
                Some(fid) => providers
                    .iter()
                    .copied()
                    .find(|p| p.id() == Some(fid.as_str())),
                None => None,
            }
        });
    let provider = match provider {
        Some(p) => p,
        None => return None,
    };
    let definition = (params.create_tool)(
        provider,
        params.config,
        params.tool_config,
        params.runtime_metadata,
    );
    let definition = match definition {
        Some(d) => d,
        None => return None,
    };
    Some(ResolvedWebProvider {
        provider,
        definition,
    })
}

pub trait HasId {
    fn id(&self) -> Option<&str>;
}

impl HasId for WebProvider {
    fn id(&self) -> Option<&str> {
        Some(&self.id)
    }
}

#[derive(Debug, Clone, Default)]
pub struct WebProvider {
    pub id: String,
}

impl RuntimeWebProviderMetadata {
    pub fn selected_provider(&self) -> Option<String> {
        self.selected_provider.clone()
    }
}

pub trait RuntimeMetadataLike {
    fn selected_provider(&self) -> Option<String>;
}

impl RuntimeMetadataLike for RuntimeWebProviderMetadata {
    fn selected_provider(&self) -> Option<String> {
        self.selected_provider.clone()
    }
}