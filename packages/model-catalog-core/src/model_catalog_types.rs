// Shared model catalog data contracts for provider manifests and normalized rows.
// 翻译自 packages/model-catalog-core/src/model-catalog-types.ts

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Supported API protocols for model catalog entries.
pub const MODEL_CATALOG_APIS: &[&str] = &[
    "openai-completions",
    "openai-responses",
    "openai-chatgpt-responses",
    "anthropic-messages",
    "google-generative-ai",
    "google-vertex",
    "github-copilot",
    "bedrock-converse-stream",
    "ollama",
    "azure-openai-responses",
];

/// API protocol for a model catalog entry.
pub type ModelCatalogApi = &'static str;

/// Supported model thinking/reasoning wire formats.
pub const MODEL_CATALOG_THINKING_FORMATS: &[&str] = &[
    "openai",
    "openrouter",
    "deepseek",
    "together",
    "qwen",
    "qwen-chat-template",
    "zai",
];

/// Thinking/reasoning wire format for model compatibility.
pub type ModelCatalogThinkingFormat = &'static str;

/// Narrow a string to a supported model catalog thinking format.
pub fn is_model_catalog_thinking_format(value: &str) -> bool {
    MODEL_CATALOG_THINKING_FORMATS.contains(&value)
}

/// Compatibility flags and provider-specific routing metadata for one model.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelCatalogCompatConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub supports_store: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub supports_developer_role: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub supports_reasoning_effort: Option<bool>,
    /// Whether the model accepts the temperature parameter (GPT-5.6 family rejects it).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub supports_temperature: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub supports_usage_in_streaming: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub supports_strict_mode: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens_field: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub requires_tool_result_name: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub requires_assistant_after_tool_result: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub requires_thinking_as_text: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub requires_reasoning_content_on_assistant_messages: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub open_router_routing: Option<ModelCatalogOpenRouterRouting>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vercel_gateway_routing: Option<ModelCatalogVercelGatewayRouting>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub zai_tool_stream: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_control_format: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub send_session_affinity_headers: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub send_session_id_header: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub supports_eager_tool_input_streaming: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub supports_long_cache_retention: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub supports_prompt_cache_key: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub supports_tools: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub requires_string_content: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub strict_message_keys: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_schema_profile: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unsupported_tool_schema_keywords: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub native_web_search_tool: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_call_arguments_encoding: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub requires_mistral_tool_ids: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub requires_open_ai_anthropic_tool_payload: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thinking_format: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub supported_reasoning_efforts: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning_effort_map: Option<BTreeMap<String, String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub visible_reasoning_detail_types: Option<Vec<String>>,
}

/// OpenRouter routing preferences copied into request metadata.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ModelCatalogOpenRouterRouting {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allow_fallbacks: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub require_parameters: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data_collection: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub zdr: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enforce_distillable_text: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub only: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ignore: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quantizations: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sort: Option<OpenRouterSort>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_price: Option<OpenRouterMaxPrice>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preferred_min_throughput: Option<OpenRouterMetricPreference>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preferred_max_latency: Option<OpenRouterMetricPreference>,
}

/// OpenRouter sort option (string or structured form).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum OpenRouterSort {
    Simple(String),
    Structured {
        #[serde(skip_serializing_if = "Option::is_none")]
        by: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        partition: Option<serde_json::Value>,
    },
}

/// OpenRouter max_price substructure.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct OpenRouterMaxPrice {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completion: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub audio: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request: Option<serde_json::Value>,
}

/// OpenRouter throughput/latency percentile cutoffs (number or structured).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum OpenRouterMetricPreference {
    Number(f64),
    Cutoffs(OpenRouterPercentileCutoffs),
}

/// OpenRouter percentile cutoffs.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct OpenRouterPercentileCutoffs {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub p50: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub p75: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub p90: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub p99: Option<f64>,
}

/// Vercel AI Gateway routing preferences.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ModelCatalogVercelGatewayRouting {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub only: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order: Option<Vec<String>>,
}

/// Image input limits for a model.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelCatalogImageInputConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_bytes: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_pixels: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_side_px: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preferred_side_px: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token_mode: Option<String>,
}

/// Media input limits for a model.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ModelCatalogMediaInputConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image: Option<ModelCatalogImageInputConfig>,
}

/// Supported input modality for a model.
pub type ModelCatalogInput = &'static str;
/// Model-level thinking settings carried by provider catalog metadata.
pub const MODEL_CATALOG_THINKING_LEVELS: &[&str] =
    &["off", "minimal", "low", "medium", "high", "xhigh", "max"];

/// Discovery lifecycle for a provider catalog.
pub type ModelCatalogDiscovery = &'static str;
/// Availability state for a model.
pub type ModelCatalogStatus = &'static str;
/// Source of a model catalog row.
pub type ModelCatalogSource = &'static str;

/// Unified catalog kind across text and generated media models.
pub type UnifiedModelCatalogKind = &'static str;

/// Source for unified model catalog entries.
pub type UnifiedModelCatalogSource = &'static str;

/// Unified model catalog entry for provider/model pickers.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UnifiedModelCatalogEntry {
    pub kind: String,
    pub provider: String,
    pub model: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    pub source: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub configured: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub capabilities: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub modes: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auth_env_vars: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub docs_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fetched_at: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub warnings: Option<Vec<String>>,
}

/// Tiered token cost row.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ModelCatalogTieredRange {
    One([f64; 1]),
    Two([f64; 2]),
}

/// Tiered token cost row.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelCatalogTieredCost {
    pub input: f64,
    pub output: f64,
    pub cache_read: f64,
    pub cache_write: f64,
    pub range: ModelCatalogTieredRange,
}

/// Token cost metadata for one model.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelCatalogCost {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_read: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_write: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tiered_pricing: Option<Vec<ModelCatalogTieredCost>>,
}

/// Model thinking level mapping.
pub type ModelCatalogThinkingLevelMap = BTreeMap<String, Option<String>>;

/// Provider manifest model entry.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelCatalogModel {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub base_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub headers: Option<BTreeMap<String, String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context_window: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context_tokens: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thinking_level_map: Option<ModelCatalogThinkingLevelMap>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cost: Option<ModelCatalogCost>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub compat: Option<ModelCatalogCompatConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub media_input: Option<ModelCatalogMediaInputConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status_reason: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub replaces: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub replaced_by: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,
}

/// Provider manifest catalog entry.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelCatalogProvider {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub base_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub headers: Option<BTreeMap<String, String>>,
    /// Provider-recommended small model id for short internal utility tasks.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_utility_model: Option<String>,
    pub models: Vec<ModelCatalogModel>,
}

/// Provider alias entry.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ModelCatalogAlias {
    pub provider: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub base_url: Option<String>,
}

/// Suppression rule for hiding a provider/model under matching config.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelCatalogSuppressionWhen {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub base_url_hosts: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider_config_api_in: Option<Vec<String>>,
}

/// Suppression rule for hiding a provider/model under matching config.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelCatalogSuppression {
    pub provider: String,
    pub model: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub when: Option<ModelCatalogSuppressionWhen>,
}

/// Raw model catalog manifest shape.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelCatalog {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub providers: Option<BTreeMap<String, ModelCatalogProvider>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub aliases: Option<BTreeMap<String, ModelCatalogAlias>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suppressions: Option<Vec<ModelCatalogSuppression>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub discovery: Option<BTreeMap<String, String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub runtime_augment: Option<bool>,
}

/// Normalized model catalog row used by runtime lookup and UI surfaces.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NormalizedModelCatalogRow {
    pub provider: String,
    pub id: String,
    pub ref_: String,
    pub merge_key: String,
    pub name: String,
    pub source: String,
    pub input: Vec<String>,
    pub reasoning: bool,
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub base_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub headers: Option<BTreeMap<String, String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context_window: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context_tokens: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thinking_level_map: Option<ModelCatalogThinkingLevelMap>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cost: Option<ModelCatalogCost>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub compat: Option<ModelCatalogCompatConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub media_input: Option<ModelCatalogMediaInputConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status_reason: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub replaces: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub replaced_by: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,
}

impl NormalizedModelCatalogRow {
    /// Constructor used by the normalize path; aligns camelCase ref/mergeKey fields.
    pub fn new(
        provider: String,
        id: String,
        ref_value: String,
        merge_key: String,
        name: String,
        source: String,
        input: Vec<String>,
        reasoning: bool,
        status: String,
    ) -> Self {
        Self {
            provider,
            id,
            ref_: ref_value,
            merge_key,
            name,
            source,
            input,
            reasoning,
            status,
            api: None,
            base_url: None,
            headers: None,
            context_window: None,
            context_tokens: None,
            max_tokens: None,
            thinking_level_map: None,
            cost: None,
            compat: None,
            media_input: None,
            status_reason: None,
            replaces: None,
            replaced_by: None,
            tags: None,
        }
    }
}