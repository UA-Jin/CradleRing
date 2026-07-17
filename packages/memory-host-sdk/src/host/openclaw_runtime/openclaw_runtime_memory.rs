// CradleRing runtime memory helpers.
// 翻译自 packages/memory-host-sdk/src/host/openclaw-runtime-memory.ts

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MemoryFlushPlan {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sections: Option<Vec<String>>,
}

pub type MemoryFlushPlanResolver = Box<dyn Fn() -> Option<MemoryFlushPlan> + Send + Sync>;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MemoryPluginCapability {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MemoryPluginPublicArtifact {
    pub id: String,
    pub kind: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
}

pub type MemoryPluginPublicArtifactsProvider =
    Box<dyn Fn() -> Vec<MemoryPluginPublicArtifact> + Send + Sync>;

pub type MemoryPromptSectionBuilder = Box<dyn Fn() -> Option<String> + Send + Sync>;

#[derive(Default)]
pub struct MemoryPluginRuntime {
    pub capability: Option<MemoryPluginCapability>,
    pub flush_plan_resolver: Option<MemoryFlushPlanResolver>,
    pub public_artifacts_provider: Option<MemoryPluginPublicArtifactsProvider>,
    pub prompt_section_builder: Option<MemoryPromptSectionBuilder>,
}

pub fn empty_plugin_config_schema() -> Value {
    Value::Null
}

pub fn build_active_memory_prompt_section() -> Option<String> {
    None
}

pub fn get_memory_capability_registration() -> Option<MemoryPluginCapability> {
    None
}

pub fn list_active_memory_public_artifacts() -> Vec<MemoryPluginPublicArtifact> {
    vec![]
}

#[derive(Default)]
pub struct CradleRingPluginApi {
    pub state_dir: String,
    pub config: HashMap<String, Value>,
}