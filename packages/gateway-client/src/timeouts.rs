// Gateway Client module implements timeouts behavior.
// 翻译自 packages/gateway-client/src/timeouts.ts

use std::collections::HashMap;

fn parse_strict_positive_integer(value: &str) -> Option<i64> {
    let trimmed = value.trim();
    if !regex::Regex::new(r"^\+?\d+$").unwrap().is_match(trimmed) {
        return None;
    }
    let parsed = trimmed.parse::<i64>().ok()?;
    if parsed > 0 {
        Some(parsed)
    } else {
        None
    }
}

/// Maximum delay Node timers can represent without overflow warnings.
pub const MAX_SAFE_TIMEOUT_DELAY_MS: i64 = 2_147_483_647;
/// Default server-side window for gateway preauth handshakes.
pub const DEFAULT_PREAUTH_HANDSHAKE_TIMEOUT_MS: i64 = 15_000;
/// Minimum client watchdog delay for connect challenge setup.
pub const MIN_CONNECT_CHALLENGE_TIMEOUT_MS: i64 = 250;
/// Default maximum client watchdog delay, aligned with the preauth server timeout.
pub const MAX_CONNECT_CHALLENGE_TIMEOUT_MS: i64 = DEFAULT_PREAUTH_HANDSHAKE_TIMEOUT_MS;

/// Clamps arbitrary timer delays to Node's safe range and an optional floor.
pub fn resolve_safe_timeout_delay_ms(delay_ms: f64, opts: Option<TimeoutDelayOpts>) -> i64 {
    let raw_min_ms = opts.as_ref().and_then(|o| o.min_ms).unwrap_or(1.0);
    let bounded_min_floor: i64 = if raw_min_ms.is_finite() {
        let f = raw_min_ms.floor();
        if f < 0.0 { 0 } else { f as i64 }
    } else {
        1_i64
    };
    let min_ms = MAX_SAFE_TIMEOUT_DELAY_MS.min(bounded_min_floor);
    let candidate_ms = if delay_ms.is_finite() { delay_ms.floor() as i64 } else { min_ms };
    MAX_SAFE_TIMEOUT_DELAY_MS.min(min_ms.max(candidate_ms))
}

#[derive(Debug, Clone, Default)]
pub struct TimeoutDelayOpts {
    pub min_ms: Option<f64>,
}

/// Adds grace time while preserving safe timer bounds if inputs overflow or are invalid.
pub fn add_safe_timeout_delay_grace_ms(delay_ms: f64, grace_ms: f64, opts: Option<TimeoutDelayOpts>) -> i64 {
    if !delay_ms.is_finite() || !grace_ms.is_finite() {
        return resolve_safe_timeout_delay_ms(MAX_SAFE_TIMEOUT_DELAY_MS as f64, opts);
    }
    let with_grace = delay_ms + grace_ms;
    resolve_safe_timeout_delay_ms(
        if with_grace.is_finite() { with_grace } else { MAX_SAFE_TIMEOUT_DELAY_MS as f64 },
        opts,
    )
}

/// Resolves optional timeout values through a fallback and safe timer clamp.
pub fn resolve_finite_timeout_delay_ms(delay_ms: Option<f64>, fallback_ms: i64, opts: Option<TimeoutDelayOpts>) -> i64 {
    let candidate_ms = match delay_ms {
        Some(d) if d.is_finite() => d as i64,
        _ => fallback_ms,
    };
    resolve_safe_timeout_delay_ms(candidate_ms as f64, opts)
}

/// Clamps connect challenge watchdog timeouts to the gateway-supported range.
pub fn clamp_connect_challenge_timeout_ms(timeout_ms: i64, max_timeout_ms: i64) -> i64 {
    let max_bounded = MIN_CONNECT_CHALLENGE_TIMEOUT_MS.max(max_timeout_ms);
    let timeout_bounded = MIN_CONNECT_CHALLENGE_TIMEOUT_MS.max(timeout_ms);
    max_bounded.min(timeout_bounded)
}

/// Reads the connect challenge watchdog override from the process environment.
pub fn get_connect_challenge_timeout_ms_from_env(env: Option<&HashMap<String, String>>) -> Option<i64> {
    let env = env?;
    let raw = env.get("CRADLE_RING_CONNECT_CHALLENGE_TIMEOUT_MS")?;
    let parsed = parse_strict_positive_integer(raw)?;
    Some(resolve_safe_timeout_delay_ms(parsed as f64, None))
}

fn normalize_positive_timeout_ms(timeout_ms: Option<i64>) -> Option<i64> {
    match timeout_ms {
        Some(t) if t > 0 => Some(resolve_safe_timeout_delay_ms(t as f64, None)),
        _ => None,
    }
}

#[derive(Debug, Clone, Default)]
pub struct ResolveConnectChallengeParams {
    pub env: Option<HashMap<String, String>>,
    pub configured_timeout_ms: Option<i64>,
}

/// Resolves the client watchdog timeout using explicit, env, then preauth defaults.
pub fn resolve_connect_challenge_timeout_ms(timeout_ms: Option<i64>, params: Option<ResolveConnectChallengeParams>) -> i64 {
    let params = params.unwrap_or_default();
    let configured_preauth_timeout_ms = resolve_preauth_handshake_timeout_ms(ResolvePreauthParams {
        env: params.env.clone(),
        configured_timeout_ms: params.configured_timeout_ms,
    });
    let max_timeout_ms = DEFAULT_PREAUTH_HANDSHAKE_TIMEOUT_MS.max(configured_preauth_timeout_ms);
    if let Some(t) = timeout_ms {
        return clamp_connect_challenge_timeout_ms(t, max_timeout_ms);
    }
    let env_override = get_connect_challenge_timeout_ms_from_env(params.env.as_ref());
    if let Some(env_val) = env_override {
        return clamp_connect_challenge_timeout_ms(env_val, max_timeout_ms.max(env_val));
    }
    clamp_connect_challenge_timeout_ms(configured_preauth_timeout_ms, max_timeout_ms)
}

#[derive(Debug, Clone, Default)]
pub struct ResolvePreauthParams {
    pub env: Option<HashMap<String, String>>,
    pub configured_timeout_ms: Option<i64>,
}

/// Reads the preauth handshake timeout override from environment variables.
pub fn get_preauth_handshake_timeout_ms_from_env(env: Option<&HashMap<String, String>>) -> i64 {
    let env = match env {
        Some(e) => e,
        None => return DEFAULT_PREAUTH_HANDSHAKE_TIMEOUT_MS,
    };
    let configured_timeout = env
        .get("CRADLE_RING_HANDSHAKE_TIMEOUT_MS")
        .or_else(|| {
            if env.contains_key("VITEST") {
                env.get("CRADLE_RING_TEST_HANDSHAKE_TIMEOUT_MS")
            } else {
                None
            }
        });
    if let Some(value) = configured_timeout {
        if let Some(parsed) = parse_strict_positive_integer(value) {
            return resolve_safe_timeout_delay_ms(parsed as f64, None);
        }
    }
    DEFAULT_PREAUTH_HANDSHAKE_TIMEOUT_MS
}

/// Resolves the server preauth timeout from env, explicit config, or default.
pub fn resolve_preauth_handshake_timeout_ms(params: ResolvePreauthParams) -> i64 {
    let env_value = get_preauth_handshake_timeout_ms_from_env(params.env.as_ref());
    if env_value != DEFAULT_PREAUTH_HANDSHAKE_TIMEOUT_MS {
        return env_value;
    }
    if let Some(configured) = normalize_positive_timeout_ms(params.configured_timeout_ms) {
        return configured;
    }
    DEFAULT_PREAUTH_HANDSHAKE_TIMEOUT_MS
}