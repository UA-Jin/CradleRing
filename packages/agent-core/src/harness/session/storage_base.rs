// Harness session storage base trait.
// 翻译自 packages/agent-core/src/harness/session/storage-base.ts

use crate::harness::types::SessionEntry;

pub trait SessionStorage: Send + Sync {
    fn load(&self, session_key: &str) -> Option<SessionEntry>;
    fn save(&self, entry: &SessionEntry) -> Result<(), String>;
    fn delete(&self, session_key: &str) -> Result<bool, String>;
    fn list(&self) -> Result<Vec<SessionEntry>, String>;
}