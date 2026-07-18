// build.rs: 编译时读取 git commit sha 和时间，注入到二进制
// 这样网关运行时能精确知道自己是哪个版本构建的

use std::process::Command;

fn main() {
    // 读取 git commit short sha
    let commit = run_git(&["rev-parse", "--short", "HEAD"])
        .unwrap_or_else(|| "unknown".to_string());

    // 读取 git commit 时间（ISO 8601）
    let commit_date = run_git(&["log", "-1", "--format=%cI", "HEAD"])
        .unwrap_or_else(|| "unknown".to_string());

    // 读取是否有未提交修改
    let dirty = Command::new("git")
        .args(["status", "--porcelain"])
        .output()
        .ok()
        .map(|o| o.status.success() && !o.stdout.is_empty())
        .unwrap_or(false);

    println!("cargo:rustc-env=CRADLE_BUILD_COMMIT={}", commit);
    println!("cargo:rustc-env=CRADLE_BUILD_DATE={}", commit_date);
    println!("cargo:rustc-env=CRADLE_BUILD_DIRTY={}", if dirty { "1" } else { "0" });
    // 让 build.rs 在 git 状态变化时重新运行
    println!("cargo:rerun-if-changed=.git/HEAD");
    println!("cargo:rerun-if-changed=.git/refs");
}

/// 运行 git 命令，成功返回 stdout，失败返回 None
fn run_git(args: &[&str]) -> Option<String> {
    let out = Command::new("git").args(args).output().ok()?;
    if !out.status.success() { return None; }
    let s = String::from_utf8(out.stdout).ok()?.trim().to_string();
    if s.is_empty() { None } else { Some(s) }
}
