// Model catalog normalization (raw provider catalogs -> stable rows).
// 翻译自 packages/model-catalog-core/src/model-catalog-normalize.ts

use std::collections::{BTreeMap, HashSet};

use serde_json::Value;

use crate::model_catalog_refs::{build_model_catalog_merge_key, build_model_catalog_ref};
use crate::model_catalog_types::{
    is_model_catalog_thinking_format, ModelCatalog, ModelCatalogAlias,
    ModelCatalogCompatConfig, ModelCatalogCost, ModelCatalogImageInputConfig,
    ModelCatalogMediaInputConfig, ModelCatalogModel, ModelCatalogOpenRouterRouting,
    ModelCatalogProvider, ModelCatalogSuppression,
    ModelCatalogSuppressionWhen, ModelCatalogThinkingLevelMap, ModelCatalogTieredCost,
    ModelCatalogTieredRange, ModelCatalogVercelGatewayRouting, NormalizedModelCatalogRow,
    OpenRouterMaxPrice, OpenRouterMetricPreference, OpenRouterPercentileCutoffs, OpenRouterSort,
    MODEL_CATALOG_APIS, MODEL_CATALOG_THINKING_LEVELS,
};
use crate::provider_id::normalize_lowercase_string_or_empty;

#[allow(dead_code)]
fn is_record(value: &Value) -> bool {
    value.is_object()
}

fn is_blocked_object_key(key: &str) -> bool {
    key == "__proto__" || key == "prototype" || key == "constructor"
}

fn normalize_optional_string(value: &Value) -> Option<String> {
    let s = value.as_str()?;
    let trimmed = s.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

fn normalize_trimmed_string_list(value: &Value) -> Vec<String> {
    let Some(arr) = value.as_array() else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for entry in arr {
        if let Some(s) = normalize_optional_string(entry) {
            out.push(s);
        }
    }
    out
}

fn normalize_optional_trimmed_string_list(value: &Value) -> Option<Vec<String>> {
    let list = normalize_trimmed_string_list(value);
    if list.is_empty() {
        None
    } else {
        Some(list)
    }
}

fn normalize_model_catalog_thinking_level_map(value: &Value) -> Option<ModelCatalogThinkingLevelMap> {
    let Some(obj) = value.as_object() else {
        return None;
    };
    let mut normalized: ModelCatalogThinkingLevelMap = BTreeMap::new();
    for level in MODEL_CATALOG_THINKING_LEVELS {
        let Some(mapped) = obj.get(*level) else {
            continue;
        };
        if mapped.is_null() {
            normalized.insert((*level).to_string(), None);
            continue;
        }
        if let Some(s) = normalize_optional_string(mapped) {
            normalized.insert((*level).to_string(), Some(s));
        }
    }
    if normalized.is_empty() {
        None
    } else {
        Some(normalized)
    }
}

fn normalize_safe_record_key(value: &Value) -> String {
    let key = normalize_optional_string(value).unwrap_or_default();
    if key.is_empty() || is_blocked_object_key(&key) {
        String::new()
    } else {
        key
    }
}

fn normalize_owned_provider_set(providers: &HashSet<String>) -> HashSet<String> {
    let mut out: HashSet<String> = HashSet::new();
    for provider in providers {
        let provider_id = normalize_lowercase_string_or_empty(provider.as_str());
        if !provider_id.is_empty() {
            out.insert(provider_id);
        }
    }
    out
}

fn normalize_string_map(value: &Value) -> Option<BTreeMap<String, String>> {
    let Some(obj) = value.as_object() else {
        return None;
    };
    let mut out: BTreeMap<String, String> = BTreeMap::new();
    for (raw_key, raw_value) in obj {
        let key = normalize_safe_record_key(&Value::String(raw_key.clone()));
        let map_value = normalize_optional_string(raw_value).unwrap_or_default();
        if !key.is_empty() && !map_value.is_empty() {
            out.insert(key, map_value);
        }
    }
    if out.is_empty() {
        None
    } else {
        Some(out)
    }
}

fn merge_string_maps(
    base: Option<BTreeMap<String, String>>,
    override_: Option<BTreeMap<String, String>>,
) -> Option<BTreeMap<String, String>> {
    match (base, override_) {
        (None, None) => None,
        (Some(b), None) => Some(b),
        (None, Some(o)) => Some(o),
        (Some(mut b), Some(o)) => {
            for (k, v) in o {
                b.insert(k, v);
            }
            Some(b)
        }
    }
}

fn normalize_model_catalog_api(value: &Value) -> Option<String> {
    let api = normalize_optional_string(value).unwrap_or_default();
    if MODEL_CATALOG_APIS.contains(&api.as_str()) {
        Some(api)
    } else {
        None
    }
}

fn normalize_model_catalog_inputs(value: &Value) -> Option<Vec<String>> {
    let allowed: HashSet<&str> = ["text", "image", "document"].iter().copied().collect();
    let list: Vec<String> = normalize_trimmed_string_list(value)
        .into_iter()
        .filter(|s| allowed.contains(s.as_str()))
        .collect();
    if list.is_empty() {
        None
    } else {
        Some(list)
    }
}

fn normalize_non_negative_number(value: &Value) -> Option<f64> {
    let n = value.as_f64()?;
    if n.is_finite() && n >= 0.0 {
        Some(n)
    } else {
        None
    }
}

fn normalize_finite_number(value: &Value) -> Option<f64> {
    let n = value.as_f64()?;
    if n.is_finite() {
        Some(n)
    } else {
        None
    }
}

fn normalize_string_or_number(value: &Value) -> Option<Value> {
    if let Some(s) = normalize_optional_string(value) {
        Some(Value::String(s))
    } else {
        normalize_finite_number(value).map(|n| serde_json::json!(n))
    }
}

fn normalize_positive_number(value: &Value) -> Option<f64> {
    let n = value.as_f64()?;
    if n.is_finite() && n > 0.0 {
        Some(n)
    } else {
        None
    }
}

fn normalize_positive_integer(value: &Value) -> Option<i64> {
    let n = value.as_i64()?;
    if n > 0 {
        Some(n)
    } else {
        None
    }
}

fn normalize_model_catalog_tiered_cost(value: &Value) -> Option<Vec<ModelCatalogTieredCost>> {
    let Some(arr) = value.as_array() else {
        return None;
    };
    let mut out: Vec<ModelCatalogTieredCost> = Vec::new();
    for entry in arr {
        let Some(obj) = entry.as_object() else {
            continue;
        };
        let Some(range_value) = obj.get("range").and_then(|v| v.as_array()) else {
            continue;
        };
        if range_value.len() < 1 || range_value.len() > 2 {
            continue;
        }
        let input = match obj.get("input").and_then(normalize_non_negative_number) {
            Some(n) => n,
            None => continue,
        };
        let output = match obj.get("output").and_then(normalize_non_negative_number) {
            Some(n) => n,
            None => continue,
        };
        let cache_read = match obj.get("cacheRead").and_then(normalize_non_negative_number) {
            Some(n) => n,
            None => continue,
        };
        let cache_write = match obj.get("cacheWrite").and_then(normalize_non_negative_number) {
            Some(n) => n,
            None => continue,
        };
        let mut range_values: Vec<f64> = Vec::new();
        for range_value in range_value {
            match normalize_non_negative_number(range_value) {
                Some(n) => range_values.push(n),
                None => {
                    range_values.clear();
                    break;
                }
            }
        }
        if range_values.is_empty() {
            continue;
        }
        let range = if range_values.len() == 1 {
            ModelCatalogTieredRange::One([range_values[0]])
        } else {
            ModelCatalogTieredRange::Two([range_values[0], range_values[1]])
        };
        out.push(ModelCatalogTieredCost {
            input,
            output,
            cache_read,
            cache_write,
            range,
        });
    }
    if out.is_empty() {
        None
    } else {
        Some(out)
    }
}

fn normalize_model_catalog_cost(value: &Value) -> Option<ModelCatalogCost> {
    let Some(obj) = value.as_object() else {
        return None;
    };
    let mut cost = ModelCatalogCost::default();
    if let Some(n) = obj.get("input").and_then(normalize_non_negative_number) {
        cost.input = Some(n);
    }
    if let Some(n) = obj.get("output").and_then(normalize_non_negative_number) {
        cost.output = Some(n);
    }
    if let Some(n) = obj.get("cacheRead").and_then(normalize_non_negative_number) {
        cost.cache_read = Some(n);
    }
    if let Some(n) = obj.get("cacheWrite").and_then(normalize_non_negative_number) {
        cost.cache_write = Some(n);
    }
    cost.tiered_pricing = obj.get("tieredPricing").and_then(normalize_model_catalog_tiered_cost);
    let has_any = cost.input.is_some()
        || cost.output.is_some()
        || cost.cache_read.is_some()
        || cost.cache_write.is_some()
        || cost.tiered_pricing.is_some();
    if has_any {
        Some(cost)
    } else {
        None
    }
}

fn normalize_open_router_price(value: &Value) -> Option<OpenRouterMaxPrice> {
    let Some(obj) = value.as_object() else {
        return None;
    };
    let mut max_price = OpenRouterMaxPrice::default();
    let mut any = false;
    if let Some(v) = obj.get("prompt").and_then(normalize_string_or_number) {
        max_price.prompt = Some(v);
        any = true;
    }
    if let Some(v) = obj.get("completion").and_then(normalize_string_or_number) {
        max_price.completion = Some(v);
        any = true;
    }
    if let Some(v) = obj.get("image").and_then(normalize_string_or_number) {
        max_price.image = Some(v);
        any = true;
    }
    if let Some(v) = obj.get("audio").and_then(normalize_string_or_number) {
        max_price.audio = Some(v);
        any = true;
    }
    if let Some(v) = obj.get("request").and_then(normalize_string_or_number) {
        max_price.request = Some(v);
        any = true;
    }
    if any {
        Some(max_price)
    } else {
        None
    }
}

fn normalize_open_router_percentile_cutoffs(value: &Value) -> Option<OpenRouterPercentileCutoffs> {
    let Some(obj) = value.as_object() else {
        return None;
    };
    let mut cutoffs = OpenRouterPercentileCutoffs::default();
    let mut any = false;
    if let Some(n) = obj.get("p50").and_then(normalize_finite_number) {
        cutoffs.p50 = Some(n);
        any = true;
    }
    if let Some(n) = obj.get("p75").and_then(normalize_finite_number) {
        cutoffs.p75 = Some(n);
        any = true;
    }
    if let Some(n) = obj.get("p90").and_then(normalize_finite_number) {
        cutoffs.p90 = Some(n);
        any = true;
    }
    if let Some(n) = obj.get("p99").and_then(normalize_finite_number) {
        cutoffs.p99 = Some(n);
        any = true;
    }
    if any {
        Some(cutoffs)
    } else {
        None
    }
}

fn normalize_open_router_metric_preference(value: &Value) -> Option<OpenRouterMetricPreference> {
    if let Some(n) = normalize_finite_number(value) {
        return Some(OpenRouterMetricPreference::Number(n));
    }
    if let Some(cutoffs) = normalize_open_router_percentile_cutoffs(value) {
        return Some(OpenRouterMetricPreference::Cutoffs(cutoffs));
    }
    None
}

fn normalize_open_router_sort(value: &Value) -> Option<OpenRouterSort> {
    if let Some(s) = normalize_optional_string(value) {
        return Some(OpenRouterSort::Simple(s));
    }
    let Some(obj) = value.as_object() else {
        return None;
    };
    let mut by: Option<String> = None;
    let mut partition: Option<Value> = None;
    let mut any = false;
    if let Some(b) = obj.get("by").and_then(normalize_optional_string) {
        by = Some(b);
        any = true;
    }
    if let Some(p) = obj.get("partition") {
        if p.is_null() {
            partition = Some(Value::Null);
            any = true;
        } else if let Some(s) = normalize_optional_string(p) {
            partition = Some(Value::String(s));
            any = true;
        }
    }
    if any {
        Some(OpenRouterSort::Structured {
            by,
            partition,
        })
    } else {
        None
    }
}

fn normalize_open_router_routing(value: &Value) -> Option<ModelCatalogOpenRouterRouting> {
    let Some(obj) = value.as_object() else {
        return None;
    };
    let mut routing = ModelCatalogOpenRouterRouting::default();
    let mut any = false;
    if let Some(b) = obj.get("allow_fallbacks").and_then(Value::as_bool) {
        routing.allow_fallbacks = Some(b);
        any = true;
    }
    if let Some(b) = obj.get("require_parameters").and_then(Value::as_bool) {
        routing.require_parameters = Some(b);
        any = true;
    }
    if let Some(d) = obj.get("data_collection").and_then(Value::as_str) {
        if d == "deny" || d == "allow" {
            routing.data_collection = Some(d.to_string());
            any = true;
        }
    }
    if let Some(b) = obj.get("zdr").and_then(Value::as_bool) {
        routing.zdr = Some(b);
        any = true;
    }
    if let Some(b) = obj.get("enforce_distillable_text").and_then(Value::as_bool) {
        routing.enforce_distillable_text = Some(b);
        any = true;
    }
    if let Some(list) = obj.get("order").and_then(normalize_optional_trimmed_string_list) {
        routing.order = Some(list);
        any = true;
    }
    if let Some(list) = obj.get("only").and_then(normalize_optional_trimmed_string_list) {
        routing.only = Some(list);
        any = true;
    }
    if let Some(list) = obj.get("ignore").and_then(normalize_optional_trimmed_string_list) {
        routing.ignore = Some(list);
        any = true;
    }
    if let Some(list) = obj.get("quantizations").and_then(normalize_optional_trimmed_string_list) {
        routing.quantizations = Some(list);
        any = true;
    }
    if let Some(sort) = obj.get("sort").and_then(normalize_open_router_sort) {
        routing.sort = Some(sort);
        any = true;
    }
    if let Some(max_price) = obj.get("max_price").and_then(normalize_open_router_price) {
        routing.max_price = Some(max_price);
        any = true;
    }
    if let Some(p) = obj
        .get("preferred_min_throughput")
        .and_then(normalize_open_router_metric_preference)
    {
        routing.preferred_min_throughput = Some(p);
        any = true;
    }
    if let Some(p) = obj
        .get("preferred_max_latency")
        .and_then(normalize_open_router_metric_preference)
    {
        routing.preferred_max_latency = Some(p);
        any = true;
    }
    if any {
        Some(routing)
    } else {
        None
    }
}

fn normalize_vercel_gateway_routing(value: &Value) -> Option<ModelCatalogVercelGatewayRouting> {
    let Some(obj) = value.as_object() else {
        return None;
    };
    let mut routing = ModelCatalogVercelGatewayRouting::default();
    let mut any = false;
    if let Some(list) = obj.get("only").and_then(normalize_optional_trimmed_string_list) {
        routing.only = Some(list);
        any = true;
    }
    if let Some(list) = obj.get("order").and_then(normalize_optional_trimmed_string_list) {
        routing.order = Some(list);
        any = true;
    }
    if any {
        Some(routing)
    } else {
        None
    }
}

fn normalize_model_catalog_compat(value: &Value) -> Option<ModelCatalogCompatConfig> {
    let Some(obj) = value.as_object() else {
        return None;
    };
    let mut compat = ModelCatalogCompatConfig::default();
    let mut any = false;

    let boolean_fields: &[&str] = &[
        "supportsStore",
        "supportsPromptCacheKey",
        "supportsDeveloperRole",
        "supportsReasoningEffort",
        "supportsTemperature",
        "supportsUsageInStreaming",
        "supportsTools",
        "supportsStrictMode",
        "requiresStringContent",
        "strictMessageKeys",
        "requiresToolResultName",
        "requiresAssistantAfterToolResult",
        "requiresThinkingAsText",
        "requiresReasoningContentOnAssistantMessages",
        "zaiToolStream",
        "sendSessionAffinityHeaders",
        "sendSessionIdHeader",
        "supportsEagerToolInputStreaming",
        "supportsLongCacheRetention",
        "nativeWebSearchTool",
        "requiresMistralToolIds",
        "requiresOpenAiAnthropicToolPayload",
    ];
    for field in boolean_fields {
        if let Some(b) = obj.get(*field).and_then(Value::as_bool) {
            match *field {
                "supportsStore" => compat.supports_store = Some(b),
                "supportsPromptCacheKey" => compat.supports_prompt_cache_key = Some(b),
                "supportsDeveloperRole" => compat.supports_developer_role = Some(b),
                "supportsReasoningEffort" => compat.supports_reasoning_effort = Some(b),
                "supportsTemperature" => compat.supports_temperature = Some(b),
                "supportsUsageInStreaming" => compat.supports_usage_in_streaming = Some(b),
                "supportsTools" => compat.supports_tools = Some(b),
                "supportsStrictMode" => compat.supports_strict_mode = Some(b),
                "requiresStringContent" => compat.requires_string_content = Some(b),
                "strictMessageKeys" => compat.strict_message_keys = Some(b),
                "requiresToolResultName" => compat.requires_tool_result_name = Some(b),
                "requiresAssistantAfterToolResult" => {
                    compat.requires_assistant_after_tool_result = Some(b)
                }
                "requiresThinkingAsText" => compat.requires_thinking_as_text = Some(b),
                "requiresReasoningContentOnAssistantMessages" => {
                    compat.requires_reasoning_content_on_assistant_messages = Some(b)
                }
                "zaiToolStream" => compat.zai_tool_stream = Some(b),
                "sendSessionAffinityHeaders" => compat.send_session_affinity_headers = Some(b),
                "sendSessionIdHeader" => compat.send_session_id_header = Some(b),
                "supportsEagerToolInputStreaming" => {
                    compat.supports_eager_tool_input_streaming = Some(b)
                }
                "supportsLongCacheRetention" => compat.supports_long_cache_retention = Some(b),
                "nativeWebSearchTool" => compat.native_web_search_tool = Some(b),
                "requiresMistralToolIds" => compat.requires_mistral_tool_ids = Some(b),
                "requiresOpenAiAnthropicToolPayload" => {
                    compat.requires_open_ai_anthropic_tool_payload = Some(b)
                }
                _ => {}
            }
            any = true;
        }
    }

    let string_fields: &[&str] = &["toolSchemaProfile", "toolCallArgumentsEncoding"];
    for field in string_fields {
        if let Some(s) = obj.get(*field).and_then(normalize_optional_string) {
            match *field {
                "toolSchemaProfile" => compat.tool_schema_profile = Some(s),
                "toolCallArgumentsEncoding" => compat.tool_call_arguments_encoding = Some(s),
                _ => {}
            }
            any = true;
        }
    }

    let string_list_fields: &[&str] = &[
        "visibleReasoningDetailTypes",
        "supportedReasoningEfforts",
        "unsupportedToolSchemaKeywords",
    ];
    for field in string_list_fields {
        let list = obj.get(*field).map(normalize_trimmed_string_list).unwrap_or_default();
        if !list.is_empty() {
            match *field {
                "visibleReasoningDetailTypes" => compat.visible_reasoning_detail_types = Some(list),
                "supportedReasoningEfforts" => compat.supported_reasoning_efforts = Some(list),
                "unsupportedToolSchemaKeywords" => compat.unsupported_tool_schema_keywords = Some(list),
                _ => {}
            }
            any = true;
        }
    }

    if let Some(reasoning_obj) = obj.get("reasoningEffortMap").and_then(Value::as_object) {
        let mut map: BTreeMap<String, String> = BTreeMap::new();
        for (k, v) in reasoning_obj {
            let key = k.trim();
            let mapped = v.as_str().map(|s| s.trim()).unwrap_or("");
            if !key.is_empty() && !mapped.is_empty() {
                map.insert(key.to_string(), mapped.to_string());
            }
        }
        if !map.is_empty() {
            compat.reasoning_effort_map = Some(map);
            any = true;
        }
    }

    if let Some(max_tokens_field) = obj.get("maxTokensField").and_then(normalize_optional_string) {
        if max_tokens_field == "max_completion_tokens" || max_tokens_field == "max_tokens" {
            compat.max_tokens_field = Some(max_tokens_field);
            any = true;
        }
    }

    if let Some(thinking_format) = obj.get("thinkingFormat").and_then(normalize_optional_string) {
        if is_model_catalog_thinking_format(&thinking_format) {
            compat.thinking_format = Some(thinking_format);
            any = true;
        }
    }

    if let Some(cache) = obj.get("cacheControlFormat").and_then(Value::as_str) {
        if cache == "anthropic" {
            compat.cache_control_format = Some("anthropic".to_string());
            any = true;
        }
    }

    if let Some(routing) = obj.get("openRouterRouting").and_then(normalize_open_router_routing) {
        compat.open_router_routing = Some(routing);
        any = true;
    }
    if let Some(routing) = obj.get("vercelGatewayRouting").and_then(normalize_vercel_gateway_routing) {
        compat.vercel_gateway_routing = Some(routing);
        any = true;
    }

    if any {
        Some(compat)
    } else {
        None
    }
}

fn normalize_model_catalog_status(value: &Value) -> Option<String> {
    let allowed: HashSet<&str> = ["available", "preview", "deprecated", "disabled"]
        .iter()
        .copied()
        .collect();
    let status = normalize_optional_string(value).unwrap_or_default();
    if allowed.contains(status.as_str()) {
        Some(status)
    } else {
        None
    }
}

fn normalize_model_catalog_image_token_mode(value: &Value) -> Option<String> {
    let token_mode = normalize_optional_string(value).unwrap_or_default();
    if token_mode == "tile" || token_mode == "detail" || token_mode == "provider" {
        Some(token_mode)
    } else {
        None
    }
}

fn normalize_model_catalog_media_input(value: &Value) -> Option<ModelCatalogMediaInputConfig> {
    let Some(obj) = value.as_object() else {
        return None;
    };
    let Some(image_value) = obj.get("image").and_then(Value::as_object) else {
        return None;
    };
    let mut image = ModelCatalogImageInputConfig::default();
    let mut any = false;
    if let Some(n) = image_value.get("maxBytes").and_then(normalize_positive_integer) {
        image.max_bytes = Some(n as u64);
        any = true;
    }
    if let Some(n) = image_value.get("maxPixels").and_then(normalize_positive_integer) {
        image.max_pixels = Some(n as u64);
        any = true;
    }
    if let Some(n) = image_value.get("maxSidePx").and_then(normalize_positive_integer) {
        image.max_side_px = Some(n as u64);
        any = true;
    }
    if let Some(n) = image_value.get("preferredSidePx").and_then(normalize_positive_integer) {
        image.preferred_side_px = Some(n as u64);
        any = true;
    }
    if let Some(mode) = image_value
        .get("tokenMode")
        .and_then(normalize_model_catalog_image_token_mode)
    {
        image.token_mode = Some(mode);
        any = true;
    }
    if any {
        Some(ModelCatalogMediaInputConfig { image: Some(image) })
    } else {
        None
    }
}

fn normalize_model_catalog_model(value: &Value) -> Option<ModelCatalogModel> {
    let Some(obj) = value.as_object() else {
        return None;
    };
    let id = normalize_optional_string(obj.get("id").unwrap_or(&Value::Null))?;
    let mut model = ModelCatalogModel {
        id,
        ..Default::default()
    };
    if let Some(name) = obj.get("name").and_then(normalize_optional_string) {
        model.name = Some(name);
    }
    if let Some(api) = obj.get("api").and_then(normalize_model_catalog_api) {
        model.api = Some(api);
    }
    if let Some(base_url) = obj.get("baseUrl").and_then(normalize_optional_string) {
        model.base_url = Some(base_url);
    }
    model.headers = obj.get("headers").and_then(normalize_string_map);
    model.input = obj.get("input").and_then(normalize_model_catalog_inputs);
    if let Some(reasoning) = obj.get("reasoning").and_then(Value::as_bool) {
        model.reasoning = Some(reasoning);
    }
    model.context_window = obj.get("contextWindow").and_then(normalize_positive_number);
    model.context_tokens = obj.get("contextTokens").and_then(normalize_positive_integer);
    model.max_tokens = obj.get("maxTokens").and_then(normalize_positive_number);
    model.thinking_level_map = obj
        .get("thinkingLevelMap")
        .and_then(normalize_model_catalog_thinking_level_map);
    model.cost = obj.get("cost").and_then(normalize_model_catalog_cost);
    model.compat = obj.get("compat").and_then(normalize_model_catalog_compat);
    model.media_input = obj.get("mediaInput").and_then(normalize_model_catalog_media_input);
    model.status = obj.get("status").and_then(normalize_model_catalog_status);
    if let Some(reason) = obj.get("statusReason").and_then(normalize_optional_string) {
        model.status_reason = Some(reason);
    }
    let replaces = normalize_trimmed_string_list(obj.get("replaces").unwrap_or(&Value::Null));
    if !replaces.is_empty() {
        model.replaces = Some(replaces);
    }
    if let Some(replaced_by) = obj.get("replacedBy").and_then(normalize_optional_string) {
        model.replaced_by = Some(replaced_by);
    }
    let tags = normalize_trimmed_string_list(obj.get("tags").unwrap_or(&Value::Null));
    if !tags.is_empty() {
        model.tags = Some(tags);
    }
    Some(model)
}

fn normalize_model_catalog_provider(value: &Value) -> Option<ModelCatalogProvider> {
    let Some(obj) = value.as_object() else {
        return None;
    };
    let mut models: Vec<ModelCatalogModel> = Vec::new();
    if let Some(arr) = obj.get("models").and_then(Value::as_array) {
        for entry in arr {
            if let Some(model) = normalize_model_catalog_model(entry) {
                models.push(model);
            }
        }
    }
    if models.is_empty() {
        return None;
    }
    let mut provider = ModelCatalogProvider {
        models,
        ..Default::default()
    };
    if let Some(base_url) = obj.get("baseUrl").and_then(normalize_optional_string) {
        provider.base_url = Some(base_url);
    }
    if let Some(api) = obj.get("api").and_then(normalize_model_catalog_api) {
        provider.api = Some(api);
    }
    provider.headers = obj.get("headers").and_then(normalize_string_map);
    if let Some(default_utility) = obj.get("defaultUtilityModel").and_then(normalize_optional_string) {
        provider.default_utility_model = Some(default_utility);
    }
    Some(provider)
}

fn normalize_model_catalog_providers(
    value: &Value,
    owned_providers: &HashSet<String>,
) -> Option<BTreeMap<String, ModelCatalogProvider>> {
    let Some(obj) = value.as_object() else {
        return None;
    };
    let mut providers: BTreeMap<String, ModelCatalogProvider> = BTreeMap::new();
    for (raw_provider_id, raw_provider) in obj {
        let provider_id = normalize_lowercase_string_or_empty(raw_provider_id.as_str());
        if provider_id.is_empty() || !owned_providers.contains(&provider_id) {
            continue;
        }
        if let Some(provider) = normalize_model_catalog_provider(raw_provider) {
            providers.insert(provider_id, provider);
        }
    }
    if providers.is_empty() {
        None
    } else {
        Some(providers)
    }
}

fn normalize_model_catalog_aliases(
    value: &Value,
    owned_providers: &HashSet<String>,
) -> Option<BTreeMap<String, ModelCatalogAlias>> {
    let Some(obj) = value.as_object() else {
        return None;
    };
    let mut aliases: BTreeMap<String, ModelCatalogAlias> = BTreeMap::new();
    for (raw_alias, raw_target) in obj {
        let alias = normalize_lowercase_string_or_empty(raw_alias.as_str());
        if alias.is_empty() {
            continue;
        }
        let Some(target) = raw_target.as_object() else {
            continue;
        };
        let provider = normalize_lowercase_string_or_empty(
            normalize_optional_string(target.get("provider").unwrap_or(&Value::Null))
                .unwrap_or_default()
                .as_str(),
        );
        if provider.is_empty() || !owned_providers.contains(&provider) {
            continue;
        }
        let api = target.get("api").and_then(normalize_model_catalog_api);
        let base_url = target.get("baseUrl").and_then(normalize_optional_string);
        let entry = ModelCatalogAlias {
            provider,
            api,
            base_url,
        };
        aliases.insert(alias, entry);
    }
    if aliases.is_empty() {
        None
    } else {
        Some(aliases)
    }
}

fn normalize_model_catalog_suppressions(value: &Value) -> Option<Vec<ModelCatalogSuppression>> {
    let Some(arr) = value.as_array() else {
        return None;
    };
    let mut suppressions: Vec<ModelCatalogSuppression> = Vec::new();
    for entry in arr {
        let Some(obj) = entry.as_object() else {
            continue;
        };
        let provider = normalize_lowercase_string_or_empty(
            normalize_optional_string(obj.get("provider").unwrap_or(&Value::Null))
                .unwrap_or_default()
                .as_str(),
        );
        let model = normalize_optional_string(obj.get("model").unwrap_or(&Value::Null))
            .unwrap_or_default();
        if provider.is_empty() || model.is_empty() {
            continue;
        }
        let reason = normalize_optional_string(obj.get("reason").unwrap_or(&Value::Null));
        let raw_when = obj.get("when").and_then(Value::as_object);
        let base_url_hosts = raw_when
            .and_then(|w| w.get("baseUrlHosts"))
            .map(normalize_trimmed_string_list)
            .unwrap_or_default()
            .into_iter()
            .map(|s| s.to_lowercase())
            .collect::<Vec<_>>();
        let provider_config_api_in = raw_when
            .and_then(|w| w.get("providerConfigApiIn"))
            .map(normalize_trimmed_string_list)
            .unwrap_or_default()
            .into_iter()
            .map(|s| s.to_lowercase())
            .collect::<Vec<_>>();
        let when = if !base_url_hosts.is_empty() || !provider_config_api_in.is_empty() {
            Some(ModelCatalogSuppressionWhen {
                base_url_hosts: if base_url_hosts.is_empty() {
                    None
                } else {
                    Some(base_url_hosts)
                },
                provider_config_api_in: if provider_config_api_in.is_empty() {
                    None
                } else {
                    Some(provider_config_api_in)
                },
            })
        } else {
            None
        };
        suppressions.push(ModelCatalogSuppression {
            provider,
            model,
            reason,
            when,
        });
    }
    if suppressions.is_empty() {
        None
    } else {
        Some(suppressions)
    }
}

fn normalize_model_catalog_discovery(
    value: &Value,
    owned_providers: &HashSet<String>,
) -> Option<BTreeMap<String, String>> {
    let Some(obj) = value.as_object() else {
        return None;
    };
    let allowed: HashSet<&str> = ["static", "refreshable", "runtime"].iter().copied().collect();
    let mut discovery: BTreeMap<String, String> = BTreeMap::new();
    for (raw_provider_id, raw_mode) in obj {
        let provider_id = normalize_lowercase_string_or_empty(raw_provider_id.as_str());
        let mode = normalize_optional_string(raw_mode).unwrap_or_default();
        if !provider_id.is_empty()
            && owned_providers.contains(&provider_id)
            && allowed.contains(mode.as_str())
        {
            discovery.insert(provider_id, mode);
        }
    }
    if discovery.is_empty() {
        None
    } else {
        Some(discovery)
    }
}

/// Normalize a raw model catalog object for the set of providers owned by a plugin/manifest.
pub fn normalize_model_catalog(
    value: &Value,
    owned_providers: &HashSet<String>,
) -> Option<ModelCatalog> {
    let Some(_) = value.as_object() else {
        return None;
    };
    let normalized_owned = normalize_owned_provider_set(owned_providers);
    let providers = normalize_model_catalog_providers(
        value.get("providers").unwrap_or(&Value::Null),
        &normalized_owned,
    );
    let aliases = normalize_model_catalog_aliases(
        value.get("aliases").unwrap_or(&Value::Null),
        &normalized_owned,
    );
    let suppressions = normalize_model_catalog_suppressions(value.get("suppressions").unwrap_or(&Value::Null));
    let discovery = normalize_model_catalog_discovery(
        value.get("discovery").unwrap_or(&Value::Null),
        &normalized_owned,
    );
    let runtime_augment = value.get("runtimeAugment").and_then(Value::as_bool);

    let mut catalog = ModelCatalog::default();
    catalog.providers = providers;
    catalog.aliases = aliases;
    catalog.suppressions = suppressions;
    catalog.discovery = discovery;
    catalog.runtime_augment = runtime_augment;

    let has_any = catalog.providers.is_some()
        || catalog.aliases.is_some()
        || catalog.suppressions.is_some()
        || catalog.discovery.is_some()
        || catalog.runtime_augment.is_some();
    if has_any {
        Some(catalog)
    } else {
        None
    }
}

/// Normalize one provider catalog into sorted runtime rows.
pub fn normalize_model_catalog_provider_rows(params: NormalizeProviderRowsParams) -> Vec<NormalizedModelCatalogRow> {
    let provider = normalize_lowercase_string_or_empty(&params.provider);
    if provider.is_empty() || !params.provider_catalog.models.iter().any(|_| true) {
        if provider.is_empty() {
            return Vec::new();
        }
    }

    let provider_api = params
        .provider_catalog
        .api
        .as_ref()
        .and_then(|api| normalize_model_catalog_api(&Value::String(api.clone())));
    let provider_base_url = params
        .provider_catalog
        .base_url
        .clone()
        .unwrap_or_default();
    let provider_headers = params.provider_catalog.headers.clone();

    let mut rows: Vec<NormalizedModelCatalogRow> = Vec::new();

    for model in &params.provider_catalog.models {
        let Some(id) = normalize_optional_string(&Value::String(model.id.clone())) else {
            continue;
        };
        let api = model
            .api
            .as_ref()
            .and_then(|v| normalize_model_catalog_api(&Value::String(v.clone())))
            .or(provider_api.clone());
        let base_url = model
            .base_url
            .clone()
            .unwrap_or_else(|| provider_base_url.clone());
        let headers = merge_string_maps(provider_headers.clone(), model.headers.clone());
        let context_window = model.context_window;
        let context_tokens = model.context_tokens;
        let max_tokens = model.max_tokens;
        let thinking_level_map = model.thinking_level_map.clone();
        let cost = model.cost.clone();
        let compat = model.compat.clone();
        let media_input = model.media_input.clone();
        let status_reason = model.status_reason.clone().unwrap_or_default();
        let replaced_by = model.replaced_by.clone().unwrap_or_default();
        let replaces = model.replaces.as_ref().and_then(|list| {
            if list.is_empty() {
                None
            } else {
                Some(list.clone())
            }
        });
        let tags = model.tags.as_ref().and_then(|list| {
            if list.is_empty() {
                None
            } else {
                Some(list.clone())
            }
        });

        let name = model.name.clone().unwrap_or_else(|| id.clone());
        let input = model
            .input
            .clone()
            .unwrap_or_else(|| vec!["text".to_string()]);
        let reasoning = model.reasoning.unwrap_or(false);
        let status = model
            .status
            .clone()
            .unwrap_or_else(|| "available".to_string());

        let mut row = NormalizedModelCatalogRow::new(
            provider.clone(),
            id.clone(),
            build_model_catalog_ref(&provider, &id),
            build_model_catalog_merge_key(&provider, &id),
            name,
            params.source.clone(),
            input,
            reasoning,
            status,
        );
        row.api = api;
        if !base_url.is_empty() {
            row.base_url = Some(base_url);
        }
        row.headers = headers;
        row.context_window = context_window;
        row.context_tokens = context_tokens;
        row.max_tokens = max_tokens;
        row.thinking_level_map = thinking_level_map;
        row.cost = cost;
        row.compat = compat;
        row.media_input = media_input;
        if !status_reason.is_empty() {
            row.status_reason = Some(status_reason);
        }
        row.replaces = replaces;
        if !replaced_by.is_empty() {
            row.replaced_by = Some(replaced_by);
        }
        row.tags = tags;

        rows.push(row);
    }

    rows.sort_by(|a, b| {
        a.provider
            .cmp(&b.provider)
            .then_with(|| a.id.cmp(&b.id))
    });
    rows
}

/// Parameters for `normalize_model_catalog_provider_rows`.
pub struct NormalizeProviderRowsParams {
    pub provider: String,
    pub provider_catalog: ModelCatalogProvider,
    pub source: String,
}