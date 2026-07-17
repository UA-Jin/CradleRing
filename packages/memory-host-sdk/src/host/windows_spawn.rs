// Windows spawn helper.
// 翻译自 packages/memory-host-sdk/src/host/windows-spawn.ts

pub fn windows_spawn(_command: &str, _args: &[&str]) -> std::io::Result<std::process::Child> {
    std::process::Command::new(_command).args(_args).spawn()
}