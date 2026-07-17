// Gateway Client module implements readiness behavior.
// 翻译自 packages/gateway-client/src/readiness.ts

use crate::event_loop_ready::{wait_for_event_loop_ready, EventLoopReadyOptions, EventLoopReadyResult};
use crate::timeouts::{resolve_connect_challenge_timeout_ms, ResolveConnectChallengeParams};

pub trait GatewayClientStartable: Send + Sync {
    fn start(&self);
}

/// Injectable readiness waiter used by tests and alternate event-loop probes.
pub type EventLoopReadyWaiter = fn(options: Option<EventLoopReadyOptions>) -> std::pin::Pin<Box<dyn std::future::Future<Output = EventLoopReadyResult> + Send>>;

#[derive(Debug, Clone, Default)]
pub struct GatewayClientStartReadinessOptions {
    pub timeout_ms: Option<f64>,
    pub client_options: Option<GatewayClientStartReadinessClientOptions>,
    pub signal: Option<crate::event_loop_ready::AbortSignalShim>,
}

#[derive(Debug, Clone, Default)]
pub struct GatewayClientStartReadinessClientOptions {
    pub connect_challenge_timeout_ms: Option<f64>,
    pub env: Option<std::collections::HashMap<String, String>>,
    pub preauth_handshake_timeout_ms: Option<i64>,
}

fn resolve_gateway_client_start_readiness_timeout_ms(
    options: &GatewayClientStartReadinessOptions,
) -> i64 {
    if let Some(t) = options.timeout_ms {
        if t.is_finite() {
            return t as i64;
        }
    }
    let client_options = options.client_options.clone().unwrap_or_default();
    let timeout_override = client_options
        .connect_challenge_timeout_ms
        .filter(|v| v.is_finite())
        .map(|v| v as i64);
    resolve_connect_challenge_timeout_ms(
        timeout_override,
        Some(ResolveConnectChallengeParams {
            env: client_options.env,
            configured_timeout_ms: client_options.preauth_handshake_timeout_ms,
        }),
    )
}

/// Starts a gateway client only after the supplied readiness probe succeeds.
pub async fn start_gateway_client_with_readiness_wait(
    wait_for_ready: fn(Option<EventLoopReadyOptions>) -> std::pin::Pin<Box<dyn std::future::Future<Output = EventLoopReadyResult> + Send>>,
    client: &dyn GatewayClientStartable,
    options: GatewayClientStartReadinessOptions,
) -> EventLoopReadyResult {
    let max_wait_ms = resolve_gateway_client_start_readiness_timeout_ms(&options);
    let signal = options.signal.clone();
    let readiness = wait_for_ready(Some(EventLoopReadyOptions {
        max_wait_ms: Some(max_wait_ms as f64),
        signal,
        ..Default::default()
    }))
    .await;
    if readiness.ready && !readiness.aborted && !options.signal.as_ref().map(|s| s.aborted).unwrap_or(false) {
        client.start();
    }
    readiness
}

/// Starts a gateway client after the default event-loop readiness probe succeeds.
pub async fn start_gateway_client_when_event_loop_ready(
    client: &dyn GatewayClientStartable,
    options: GatewayClientStartReadinessOptions,
) -> EventLoopReadyResult {
    let wrapped_wait: fn(Option<EventLoopReadyOptions>) -> std::pin::Pin<Box<dyn std::future::Future<Output = EventLoopReadyResult> + Send>> =
        |opts| Box::pin(wait_for_event_loop_ready(opts.unwrap_or_default()));
    start_gateway_client_with_readiness_wait(wrapped_wait, client, options).await
}