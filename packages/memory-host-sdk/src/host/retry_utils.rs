// Retry utilities.
// 翻译自 packages/memory-host-sdk/src/host/retry-utils.ts

pub async fn retry_with_backoff<F, T>(mut op: F, attempts: u32) -> Result<T, String>
where
    F: FnMut() -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<T, String>> + Send>>,
{
    let mut last_err = String::new();
    for i in 0..attempts {
        match op().await {
            Ok(v) => return Ok(v),
            Err(e) => {
                last_err = e;
                tokio::time::sleep(std::time::Duration::from_millis(100 * (1 << i))).await;
            }
        }
    }
    Err(last_err)
}