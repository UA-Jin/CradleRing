// Public memory host contracts shared by runtime, QMD, builtin search, and
// package consumers.
// 翻译自 packages/memory-host-sdk/src/host/types.ts

use serde::{Deserialize, Serialize};
use serde_json::Value;

pub type MemorySource = String;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemorySearchResult {
    pub path: String,
    pub start_line: i64,
    pub end_line: i64,
    pub score: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vector_score: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text_score: Option<f64>,
    pub snippet: String,
    pub source: MemorySource,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub citation: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MemoryEmbeddingProbeResult {
    pub ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub checked: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cached: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub checked_at_ms: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_expires_at_ms: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MemorySyncProgressUpdate {
    pub completed: i64,
    pub total: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MemorySessionSyncTarget {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent_id: Option<String>,
    pub session_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_key: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MemorySyncParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub force: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sessions: Option<Vec<MemorySessionSyncTarget>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub archive_files: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub progress: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MemorySearchRuntimeQmdCollectionValidationDebug {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_state: Option<String>,
    pub elapsed_ms: i64,
    pub collection_count: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub list_calls: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub show_calls: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MemorySearchRuntimeQmdMultiCollectionProbeDebug {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_state: Option<String>,
    pub elapsed_ms: i64,
    pub supported: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MemorySearchRuntimeQmdSearchPlanDebug {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub command: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub collection_count: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub group_count: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sources: Option<Vec<MemorySource>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MemorySearchRuntimeQmdDebug {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub collection_validation: Option<MemorySearchRuntimeQmdCollectionValidationDebug>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub multi_collection_probe: Option<MemorySearchRuntimeQmdMultiCollectionProbeDebug>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub search_plan: Option<MemorySearchRuntimeQmdSearchPlanDebug>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MemorySearchRuntimeDebug {
    pub backend: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub configured_mode: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub effective_mode: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fallback: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub qmd: Option<MemorySearchRuntimeQmdDebug>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MemoryReadResult {
    pub text: String,
    pub path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub truncated: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lines: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_from: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MemoryProviderStatus {
    pub backend: String,
    pub provider: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub requested_provider: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub files: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chunks: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dirty: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workspace_dir: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub db_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extra_paths: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sources: Option<Vec<MemorySource>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_counts: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fts: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fallback: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vector: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub batch: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom: Option<Value>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MemorySearchOpts {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_results: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_score: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub qmd_search_mode_override: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub on_debug: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sources: Option<Vec<MemorySource>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signal: Option<Value>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MemoryReadParams {
    pub rel_path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lines: Option<i64>,
}

/// Search/read/sync/status contract implemented by memory managers.
pub trait MemorySearchManager: Send + Sync {
    fn search(
        &self,
        query: &str,
        opts: Option<MemorySearchOpts>,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<Vec<MemorySearchResult>, String>> + Send>,
    >;
    fn read_file(
        &self,
        params: MemoryReadParams,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<MemoryReadResult, String>> + Send>,
    >;
    fn status(&self) -> MemoryProviderStatus;
    fn probe_embedding_availability(
        &self,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<MemoryEmbeddingProbeResult, String>> + Send>,
    >;
    fn probe_vector_availability(
        &self,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<bool, String>> + Send>>;
}