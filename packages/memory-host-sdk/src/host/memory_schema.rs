// Memory index schema helpers.
// 翻译自 packages/memory-host-sdk/src/host/memory-schema.ts

pub const MEMORY_EMBEDDING_CACHE_TABLE: &str = "memory_embedding_cache";
pub const MEMORY_INDEX_CHUNKS_TABLE: &str = "memory_index_chunks";
pub const MEMORY_INDEX_FTS_TABLE: &str = "memory_index_fts";
pub const MEMORY_INDEX_META_TABLE: &str = "memory_index_meta";
pub const MEMORY_INDEX_PATHS_FTS_TABLE: &str = "memory_index_paths_fts";
pub const MEMORY_INDEX_SOURCES_TABLE: &str = "memory_index_sources";
pub const MEMORY_INDEX_STATE_TABLE: &str = "memory_index_state";
pub const MEMORY_INDEX_VECTOR_TABLE: &str = "memory_index_vector";

pub fn ensure_memory_index_schema(_db: &dyn std::fmt::Debug) -> Result<(), String> {
    Ok(())
}

pub fn ensure_memory_path_fts_triggers(_db: &dyn std::fmt::Debug) -> Result<(), String> {
    Ok(())
}

pub fn drop_memory_path_fts_triggers(_db: &dyn std::fmt::Debug) -> Result<(), String> {
    Ok(())
}