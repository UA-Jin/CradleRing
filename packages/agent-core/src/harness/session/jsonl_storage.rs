// Harness session JSONL storage.
// 翻译自 packages/agent-core/src/harness/session/jsonl-storage.ts

use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;

use crate::harness::session::storage_base::SessionStorage;
use crate::harness::types::SessionEntry;

pub struct JsonlSessionStorage {
    base_dir: PathBuf,
    cache: Mutex<HashMap<String, SessionEntry>>,
}

impl JsonlSessionStorage {
    pub fn new(base_dir: PathBuf) -> Self {
        Self {
            base_dir,
            cache: Mutex::new(HashMap::new()),
        }
    }

    fn file_for(&self, session_key: &str) -> PathBuf {
        self.base_dir.join(format!("{}.jsonl", session_key))
    }
}

impl SessionStorage for JsonlSessionStorage {
    fn load(&self, session_key: &str) -> Option<SessionEntry> {
        self.cache.lock().unwrap().get(session_key).cloned()
    }

    fn save(&self, entry: &SessionEntry) -> Result<(), String> {
        let session_key = entry.session_key.clone().unwrap_or_else(|| entry.id.clone());
        let path = self.file_for(&session_key);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        let mut entry_with_key = entry.clone();
        if entry_with_key.session_key.is_none() {
            entry_with_key.session_key = Some(session_key.clone());
        }
        let line = serde_json::to_string(&entry_with_key).map_err(|e| e.to_string())?;
        fs::write(&path, format!("{}\n", line)).map_err(|e| e.to_string())?;
        self.cache
            .lock()
            .unwrap()
            .insert(session_key, entry_with_key);
        Ok(())
    }

    fn delete(&self, session_key: &str) -> Result<bool, String> {
        let path = self.file_for(session_key);
        let removed = self.cache.lock().unwrap().remove(session_key).is_some();
        if path.exists() {
            fs::remove_file(&path).map_err(|e| e.to_string())?;
            return Ok(true);
        }
        Ok(removed)
    }

    fn list(&self) -> Result<Vec<SessionEntry>, String> {
        let cache = self.cache.lock().unwrap();
        Ok(cache.values().cloned().collect())
    }
}