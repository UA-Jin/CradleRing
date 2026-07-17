// Harness session UUID generator.
// 翻译自 packages/agent-core/src/harness/session/uuid.ts

use uuid::Uuid;

/// Generates a UUID v4 string for new session identifiers (v7 unavailable in this uuid crate).
pub fn uuidv7() -> String {
    Uuid::new_v4().to_string()
}