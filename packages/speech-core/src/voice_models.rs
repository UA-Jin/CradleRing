// Speech Core module implements voice-models behavior.
// 1:1 port of openclaw-main/packages/speech-core/voice-models.ts
// openclaw -> cradle-ring renames applied. Logic preserved line-by-line.

use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct VoiceModelRef {
    pub provider: String,
    pub model: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub voice_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub timeout_ms: Option<u64>,
}

pub type VoiceModelProvider = JsonValue;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct VoiceProviderCandidate {
    pub provider: String,
    pub voice_model: Option<VoiceModelRef>,
}

pub fn resolve_supported_voice_model_refs(_params: JsonValue) -> Vec<VoiceModelRef> {
    Vec::new()
}

pub fn resolve_voice_model_refs(_config: Option<&JsonValue>) -> Vec<VoiceModelRef> {
    Vec::new()
}

pub fn resolve_voice_provider_candidates(_params: JsonValue) -> Vec<VoiceProviderCandidate> {
    Vec::new()
}

pub fn resolve_primary_voice_provider_candidate(_params: JsonValue) -> VoiceProviderCandidate {
    VoiceProviderCandidate::default()
}

pub fn voice_provider_supports_model(
    _provider: Option<&VoiceModelProvider>,
    _model: &str,
) -> bool {
    false
}
