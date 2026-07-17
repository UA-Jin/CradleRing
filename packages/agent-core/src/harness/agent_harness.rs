// Harness orchestrator facade.
// 翻译自 packages/agent-core/src/harness/agent-harness.ts

use crate::harness::session::storage_base::SessionStorage;
use crate::harness::types::SessionEntry;

pub struct AgentHarness<'a> {
    pub storage: &'a dyn SessionStorage,
}

impl<'a> AgentHarness<'a> {
    pub fn new(storage: &'a dyn SessionStorage) -> Self {
        Self { storage }
    }

    pub fn load(&self, session_key: &str) -> Option<SessionEntry> {
        self.storage.load(session_key)
    }

    pub fn save(&self, entry: &SessionEntry) -> Result<(), String> {
        self.storage.save(entry)
    }
}