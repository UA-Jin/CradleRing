// Harness Node.js execution env.
// 翻译自 packages/agent-core/src/harness/env/nodejs.ts

use std::process::Command;

use crate::harness::env::kill_tree::kill_process_tree;

#[derive(Debug, Clone, Default)]
pub struct NodeExecutionEnv;

impl NodeExecutionEnv {
    pub fn new() -> Self {
        Self
    }

    pub fn run(&self, command: &str, args: &[&str], cwd: Option<&str>) -> std::io::Result<std::process::Output> {
        let mut cmd = Command::new(command);
        cmd.args(args);
        if let Some(dir) = cwd {
            cmd.current_dir(dir);
        }
        cmd.output()
    }

    pub fn kill_tree(&self, pid: u32) -> std::io::Result<()> {
        kill_process_tree(pid)
    }
}