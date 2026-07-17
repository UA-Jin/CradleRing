// Gateway Client module implements event loop ready behavior.
// 翻译自 packages/gateway-client/src/event-loop-ready.ts

use std::time::{Duration, Instant};

use crate::timeouts::resolve_finite_timeout_delay_ms;

#[derive(Debug, Clone)]
pub struct EventLoopReadyResult {
    pub ready: bool,
    pub elapsed_ms: i64,
    pub max_drift_ms: i64,
    pub checks: i64,
    pub aborted: bool,
}

#[derive(Debug, Clone, Default)]
pub struct EventLoopReadyOptions {
    pub max_wait_ms: Option<f64>,
    pub interval_ms: Option<f64>,
    pub drift_threshold_ms: Option<f64>,
    pub consecutive_ready_checks: Option<f64>,
    pub signal: Option<AbortSignalShim>,
}

#[derive(Debug, Clone, Default)]
pub struct AbortSignalShim {
    pub aborted: bool,
}

const DEFAULT_MAX_WAIT_MS: f64 = 10_000.0;
const DEFAULT_INTERVAL_MS: f64 = 1.0;
const DEFAULT_DRIFT_THRESHOLD_MS: f64 = 200.0;
const DEFAULT_CONSECUTIVE_READY_CHECKS: f64 = 2.0;

fn resolve_positive_integer(value: Option<f64>, fallback: f64) -> i64 {
    match value {
        Some(v) if v.is_finite() => 1.max(v.floor() as i64),
        _ => fallback as i64,
    }
}

/// Waits until timer drift stays low for consecutive checks, or aborts/times out.
pub async fn wait_for_event_loop_ready(options: EventLoopReadyOptions) -> EventLoopReadyResult {
    let max_wait_ms = resolve_finite_timeout_delay_ms(options.max_wait_ms, DEFAULT_MAX_WAIT_MS as i64, None);
    let interval_ms = resolve_finite_timeout_delay_ms(options.interval_ms, DEFAULT_INTERVAL_MS as i64, None);
    let drift_threshold_ms = resolve_positive_integer(options.drift_threshold_ms, DEFAULT_DRIFT_THRESHOLD_MS) as f64;
    let consecutive_ready_checks = resolve_positive_integer(options.consecutive_ready_checks, DEFAULT_CONSECUTIVE_READY_CHECKS);

    let started_at = Instant::now();
    let mut ready_checks: i64 = 0;
    let mut checks: i64 = 0;
    let mut max_drift_ms: i64 = 0;
    let signal = options.signal;

    loop {
        if signal.as_ref().map(|s| s.aborted).unwrap_or(false) {
            return EventLoopReadyResult {
                ready: false,
                elapsed_ms: started_at.elapsed().as_millis() as i64,
                max_drift_ms,
                checks,
                aborted: true,
            };
        }

        let elapsed_ms = started_at.elapsed().as_millis() as i64;
        let remaining_ms = max_wait_ms - elapsed_ms;
        if remaining_ms <= 0 {
            return EventLoopReadyResult {
                ready: false,
                elapsed_ms,
                max_drift_ms,
                checks,
                aborted: false,
            };
        }

        let delay_ms = interval_ms.min(remaining_ms);
        let scheduled_at = Instant::now();
        tokio::time::sleep(Duration::from_millis(delay_ms as u64)).await;

        checks += 1;
        let drift_ms = (scheduled_at.elapsed().as_millis() as i64 - delay_ms).max(0);
        max_drift_ms = max_drift_ms.max(drift_ms);
        if drift_ms as f64 > drift_threshold_ms {
            ready_checks = 0;
        } else {
            ready_checks += 1;
        }
        if ready_checks >= consecutive_ready_checks {
            return EventLoopReadyResult {
                ready: true,
                elapsed_ms: started_at.elapsed().as_millis() as i64,
                max_drift_ms,
                checks,
                aborted: false,
            };
        }
    }
}