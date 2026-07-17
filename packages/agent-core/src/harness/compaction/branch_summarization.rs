// Harness branch summarization.
// 翻译自 packages/agent-core/src/harness/compaction/branch-summarization.ts

use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BranchPathEntry {
    pub id: String,
    pub from_id: Option<String>,
    pub content: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BranchPreparation {
    pub path: Vec<BranchPathEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BranchSummaryDetails {
    pub from_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectBranchPathEntriesResult {
    pub entries: Vec<BranchPathEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectEntriesResult {
    pub entries: Vec<Value>,
}

pub fn collect_entries_for_branch_summary(entries: Vec<Value>) -> CollectEntriesResult {
    CollectEntriesResult { entries }
}

pub fn collect_entries_for_branch_summary_from_branches(
    branches: Vec<Vec<Value>>,
) -> CollectEntriesResult {
    CollectEntriesResult {
        entries: branches.into_iter().flatten().collect(),
    }
}

pub fn prepare_branch_entries(entries: Vec<Value>) -> BranchPreparation {
    BranchPreparation {
        path: entries
            .into_iter()
            .map(|e| BranchPathEntry {
                id: e
                    .get("id")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                from_id: e
                    .get("fromId")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string()),
                content: e,
            })
            .collect(),
    }
}

pub async fn generate_branch_summary(details: BranchSummaryDetails, text: &str) -> String {
    format!("Branch summary for {}: {}", details.from_id, text)
}