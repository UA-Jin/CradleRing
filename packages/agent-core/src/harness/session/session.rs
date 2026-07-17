// Harness session helper facade.
// 翻译自 packages/agent-core/src/harness/session/session.ts

use crate::harness::session::storage_base::SessionStorage;
use crate::harness::types::SessionEntry;

pub struct SessionHelper<'a> {
    pub storage: &'a dyn SessionStorage,
}

impl<'a> SessionHelper<'a> {
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