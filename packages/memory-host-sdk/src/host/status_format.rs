// Status formatting helper.
// 翻译自 packages/memory-host-sdk/src/host/status-format.ts

pub fn format_status_line(label: &str, value: &str) -> String {
    format!("{}: {}", label, value)
}