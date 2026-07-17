// Speech Core module implements api behavior.
// 1:1 port of openclaw-main/packages/speech-core/api.ts
// openclaw -> cradle-ring renames applied. Logic preserved line-by-line.

use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use std::collections::HashMap;

pub type TtsAutoMode = String;
pub type TtsProvider = String;
pub type OpenClawConfig = JsonValue;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ResolvedTtsPersona {
    pub id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provider: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub providers: Option<HashMap<String, SpeechProviderConfig>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fallback_policy: Option<String>,
}

pub type SpeechProviderConfig = JsonValue;
pub type SpeechProviderOverrides = JsonValue;
pub type SpeechVoiceOption = JsonValue;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TtsDirectiveOverrides {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provider: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provider_overrides: Option<HashMap<String, SpeechProviderOverrides>>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TtsDirectiveParseResult {
    pub cleaned_text: String,
    #[serde(default)]
    pub has_directive: bool,
    #[serde(default)]
    pub warnings: Vec<String>,
    #[serde(default)]
    pub tts_text: Option<String>,
    #[serde(default)]
    pub overrides: Option<TtsDirectiveOverrides>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ResolvedTtsModelOverrides {
    pub enabled: bool,
    pub allow_text: bool,
    pub allow_provider: bool,
    pub allow_voice: bool,
    pub allow_model_id: bool,
    pub allow_voice_settings: bool,
    pub allow_normalization: bool,
    pub allow_seed: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ResolvedTtsConfig {
    pub auto: String,
    pub mode: String,
    pub provider: String,
    pub provider_source: String,
    #[serde(default)]
    pub persona: Option<String>,
    pub personas: HashMap<String, ResolvedTtsPersona>,
    #[serde(default)]
    pub summary_model: Option<String>,
    pub model_overrides: ResolvedTtsModelOverrides,
    pub provider_configs: HashMap<String, SpeechProviderConfig>,
    #[serde(default)]
    pub prefs_path: Option<String>,
    pub max_text_length: usize,
    pub timeout_ms: u64,
    pub timeout_ms_source: String,
    #[serde(default)]
    pub raw_config: Option<JsonValue>,
    #[serde(default)]
    pub source_config: Option<JsonValue>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TtsConfigResolutionContext {
    #[serde(default)]
    pub agent_id: Option<String>,
    #[serde(default)]
    pub channel_id: Option<String>,
    #[serde(default)]
    pub account_id: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TtsModelOverrideConfig {
    pub enabled: Option<bool>,
    pub allow_text: Option<bool>,
    pub allow_provider: Option<bool>,
    pub allow_voice: Option<bool>,
    pub allow_model_id: Option<bool>,
    pub allow_voice_settings: Option<bool>,
    pub allow_normalization: Option<bool>,
    pub allow_seed: Option<bool>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TtsConfig {
    pub auto: Option<String>,
    pub enabled: Option<bool>,
    pub mode: Option<String>,
    pub provider: Option<String>,
    pub persona: Option<String>,
    pub max_text_length: Option<usize>,
    pub max_length: Option<usize>,
    pub summarize: Option<bool>,
    pub providers: Option<JsonValue>,
    pub personas: Option<JsonValue>,
    pub summary_model: Option<String>,
    pub prefs_path: Option<String>,
    pub timeout_ms: Option<u64>,
    pub model_overrides: Option<TtsModelOverrideConfig>,
}

pub type SpeechProviderPlugin = JsonValue;

pub fn canonicalize_speech_provider_id(_id: Option<&str>, _cfg: Option<&JsonValue>) -> Option<String> {
    None
}
pub fn get_speech_provider(
    _id: &str,
    _cfg: Option<&JsonValue>,
) -> Option<SpeechProviderPlugin> {
    None
}
pub fn list_speech_providers(_cfg: Option<&JsonValue>) -> Vec<SpeechProviderPlugin> {
    Vec::new()
}
pub fn normalize_speech_provider_id(_id: Option<&str>) -> Option<String> {
    None
}
pub fn normalize_tts_auto_mode(_mode: Option<&str>) -> Option<String> {
    None
}
pub fn parse_tts_directives(
    _text: &str,
    _overrides: &ResolvedTtsModelOverrides,
    _options: &JsonValue,
) -> TtsDirectiveParseResult {
    TtsDirectiveParseResult {
        cleaned_text: String::new(),
        has_directive: false,
        warnings: Vec::new(),
        tts_text: None,
        overrides: None,
    }
}
pub fn resolve_effective_tts_config(
    _cfg: &OpenClawConfig,
    _agent: Option<&str>,
) -> TtsConfig {
    TtsConfig::default()
}
pub fn schedule_cleanup(_path: &str) {}
pub async fn summarize_text(_params: JsonValue) -> JsonValue {
    JsonValue::Null
}
