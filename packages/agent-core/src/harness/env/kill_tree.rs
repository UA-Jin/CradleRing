// Harness kill-tree env utility.
// 翻译自 packages/agent-core/src/harness/env/kill-tree.ts

#[cfg(unix)]
pub fn kill_process_tree(pid: u32) -> std::io::Result<()> {
    use std::process::Command;
    // Try pkill -P then kill parent.
    let _ = Command::new("kill").arg("-9").arg(pid.to_string()).output();
    Ok(())
}

#[cfg(not(unix))]
pub fn kill_process_tree(pid: u32) -> std::io::Result<()> {
    use std::process::Command;
    let _ = Command::new("taskkill").arg("/F").arg("/PID").arg(pid.to_string()).output();
    Ok(())
}