// Error utils.
// 翻译自 packages/memory-host-sdk/src/host/error-utils.ts

pub fn format_error_chain(error: &dyn std::error::Error) -> String {
    let mut chain = vec![error.to_string()];
    let mut source = error.source();
    while let Some(s) = source {
        chain.push(s.to_string());
        source = s.source();
    }
    chain.join(" <- ")
}