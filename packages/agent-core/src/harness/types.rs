// Harness shared types.
// 翻译自 packages/agent-core/src/harness/types.ts

use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionEntry {
    pub id: String,
    pub session_key: Option<String>,
    pub parent_session_key: Option<String>,
    pub spawned_by: Option<String>,
    pub spawn_depth: Option<i64>,
    pub created_at: Option<i64>,
    pub updated_at: Option<i64>,
    pub title: Option<String>,
    pub workspace_dir: Option<String>,
    pub cwd: Option<String>,
    pub metadata: Option<Value>,
}