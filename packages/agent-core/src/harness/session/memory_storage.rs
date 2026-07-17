// Harness session memory storage.
// 翻译自 packages/agent-core/src/harness/session/memory-storage.ts

use std::collections::HashMap;
use std::sync::Mutex;

use crate::harness::session::storage_base::SessionStorage;
use crate::harness::types::SessionEntry;

#[derive(Default)]
pub struct InMemorySessionStorage {
    entries: Mutex<HashMap<String, SessionEntry>>,
}

impl InMemorySessionStorage {
    pub fn new() -> Self {
        Self::default()
    }
}

impl SessionStorage for InMemorySessionStorage {
    fn load(&self, session_key: &str) -> Option<SessionEntry> {
        self.entries.lock().unwrap().get(session_key).cloned()
    }
    fn save(&self, entry: &SessionEntry) -> Result<(), String> {
        let key = entry.session_key.clone().unwrap_or_else(|| entry.id.clone());
        self.entries.lock().unwrap().insert(key, entry.clone());
        Ok(())
    }
    fn delete(&self, session_key: &str) -> Result<bool, String> {
        Ok(self.entries.lock().unwrap().remove(session_key).is_some())
    }
    fn list(&self) -> Result<Vec<SessionEntry>, String> {
        Ok(self.entries.lock().unwrap().values().cloned().collect())
    }
}