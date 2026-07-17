// QMD query parser helper.
// 翻译自 packages/memory-host-sdk/src/host/qmd-query-parser.ts

pub fn parse_qmd_query(query: &str) -> Vec<String> {
    query.split_whitespace().map(|s| s.to_string()).collect()
}