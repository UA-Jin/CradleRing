// QMD process helper.
// 翻译自 packages/memory-host-sdk/src/host/qmd-process.ts

pub fn start_qmd_process() -> std::io::Result<std::process::Child> {
    std::process::Command::new("qmd").spawn()
}