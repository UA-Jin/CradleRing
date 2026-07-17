// SQLite host helpers.
// 翻译自 packages/memory-host-sdk/src/host/sqlite.ts

pub fn require_node_sqlite() -> Result<(), String> {
    Ok(())
}

pub fn configure_memory_sqlite_wal_maintenance(_db: &dyn std::fmt::Debug) -> Result<(), String> {
    Ok(())
}

pub fn close_memory_sqlite_wal_maintenance(_db: &dyn std::fmt::Debug) -> Result<(), String> {
    Ok(())
}