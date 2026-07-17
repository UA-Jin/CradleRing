// Query expansion helper.
// 翻译自 packages/memory-host-sdk/src/host/query-expansion.ts

pub fn expand_query(query: &str) -> Vec<String> {
    query.split_whitespace().map(|s| s.to_string()).collect()
}