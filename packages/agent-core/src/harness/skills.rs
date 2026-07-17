// Harness skills.
// 翻译自 packages/agent-core/src/harness/skills.ts

use serde_json::Value;

#[derive(Debug, Clone)]
pub struct SkillDefinition {
    pub name: String,
    pub description: Option<String>,
    pub body: String,
    pub metadata: Option<Value>,
}