// Read retry helpers.
// 翻译自 packages/memory-host-sdk/src/host/read-retry.ts

pub fn is_transient_memory_read_error(error: &dyn std::error::Error) -> bool {
    let msg = error.to_string().to_lowercase();
    msg.contains("timeout") || msg.contains("temporarily unavailable") || msg.contains("busy")
}

pub async fn retry_transient_memory_read<F, T>(mut op: F) -> Result<T, String>
where
    F: FnMut() -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<T, String>> + Send>>,
{
    let mut attempts = 0;
    loop {
        match op().await {
            Ok(v) => return Ok(v),
            Err(e) if attempts < 3 => {
                let transient_marker = std::io::Error::new(std::io::ErrorKind::Other, e.clone());
                if is_transient_memory_read_error(&transient_marker) {
                    attempts += 1;
                    tokio::time::sleep(std::time::Duration::from_millis(100 * (1 << attempts))).await;
                } else {
                    return Err(e);
                }
            }
            Err(e) => return Err(e),
        }
    }
}