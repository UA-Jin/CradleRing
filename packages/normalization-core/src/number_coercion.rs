// Number coercion helpers.
// 翻译自 packages/normalization-core/src/number-coercion.ts

use serde_json::Value;

/// Returns a number only when the input is already finite.
pub fn as_finite_number(value: &Value) -> Option<f64> {
    let n = value.as_f64()?;
    if n.is_finite() {
        Some(n)
    } else {
        None
    }
}

/// Range bounds for as_finite_number_in_range
#[derive(Debug, Clone, Default)]
pub struct NumberRange {
    pub min: Option<f64>,
    pub max: Option<f64>,
    pub min_exclusive: bool,
    pub max_exclusive: bool,
}

/// Returns a finite number only when it satisfies the supplied inclusive/exclusive bounds.
pub fn as_finite_number_in_range(value: &Value, range: &NumberRange) -> Option<f64> {
    let number = as_finite_number(value)?;
    if let Some(min) = range.min {
        if range.min_exclusive {
            if number <= min {
                return None;
            }
        } else if number < min {
            return None;
        }
    }
    if let Some(max) = range.max {
        if range.max_exclusive {
            if number >= max {
                return None;
            }
        } else if number > max {
            return None;
        }
    }
    Some(number)
}

/// Returns a safe integer only when it satisfies the supplied inclusive bounds.
pub fn as_safe_integer_in_range(value: &Value, min: Option<f64>, max: Option<f64>) -> Option<i64> {
    let n = value.as_i64()?;
    if let Some(min) = min {
        if (n as f64) < min {
            return None;
        }
    }
    if let Some(max) = max {
        if (n as f64) > max {
            return None;
        }
    }
    Some(n)
}

fn normalize_numeric_string(value: &str) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

/// Parses finite numbers from number values or strict numeric string tokens.
pub fn parse_finite_number(value: &Value) -> Option<f64> {
    if let Some(n) = value.as_f64() {
        return if n.is_finite() { Some(n) } else { None };
    }
    parse_strict_finite_number(value)
}

/// Parses only safe integer numbers or base-10 integer strings.
pub fn parse_strict_integer(value: &Value) -> Option<i64> {
    if let Some(n) = value.as_i64() {
        return Some(n);
    }
    let s = value.as_str()?;
    let normalized = normalize_numeric_string(s)?;
    let re = regex::Regex::new(r"^[+-]?\d+$").unwrap();
    if !re.is_match(&normalized) {
        return None;
    }
    normalized.parse::<i64>().ok()
}

/// Parses only finite decimal/scientific string tokens, rejecting partial numbers.
pub fn parse_strict_finite_number(value: &Value) -> Option<f64> {
    if let Some(n) = value.as_f64() {
        return if n.is_finite() { Some(n) } else { None };
    }
    let s = value.as_str()?;
    let normalized = normalize_numeric_string(s)?;
    let re = regex::Regex::new(r"(?i)^[+-]?(?:(?:\d+\.?\d*)|(?:\.\d+))(?:e[+-]?\d+)?$").unwrap();
    if !re.is_match(&normalized) {
        return None;
    }
    let parsed: f64 = normalized.parse().ok()?;
    if parsed.is_finite() {
        Some(parsed)
    } else {
        None
    }
}

/// Returns positive safe integers without string coercion.
pub fn as_positive_safe_integer(value: &Value) -> Option<i64> {
    let n = value.as_i64()?;
    if n > 0 {
        Some(n)
    } else {
        None
    }
}

/// Conservative upper bound for Node timer delays.
pub const MAX_TIMER_TIMEOUT_MS: i64 = 2_147_000_000;
/// Timer bound expressed in whole seconds for env/config inputs.
pub const MAX_TIMER_TIMEOUT_SECONDS: i64 = MAX_TIMER_TIMEOUT_MS / 1000;
/// Largest timestamp accepted by JavaScript Date.
pub const MAX_DATE_TIMESTAMP_MS: f64 = 8_640_000_000_000_000.0;
/// Fallback ISO value for invalid timestamp inputs.
pub const UNIX_EPOCH_ISO_STRING: &str = "1970-01-01T00:00:00.000Z";

/// Returns a Date-valid millisecond timestamp.
pub fn as_date_timestamp_ms(value: &Value) -> Option<i64> {
    let n = as_finite_number_in_range(
        value,
        &NumberRange {
            min: Some(-MAX_DATE_TIMESTAMP_MS),
            max: Some(MAX_DATE_TIMESTAMP_MS),
            ..Default::default()
        },
    )?;
    Some(n as i64)
}

/// Checks whether a Date-valid timestamp is after the supplied/current time.
pub fn is_future_date_timestamp_ms(value: &Value, now_ms: Option<i64>) -> bool {
    let timestamp_ms = as_date_timestamp_ms(value);
    let now = now_ms.or_else(|| Some(chrono::Utc::now().timestamp_millis()));
    match (timestamp_ms, now) {
        (Some(ts), Some(n)) => ts > n,
        _ => false,
    }
}

/// Converts Date-valid millisecond timestamps to ISO strings.
pub fn timestamp_ms_to_iso_string(value: &Value) -> Option<String> {
    let timestamp_ms = as_date_timestamp_ms(value)?;
    use chrono::TimeZone;
    chrono::Utc
        .timestamp_millis_opt(timestamp_ms)
        .single()
        .map(|dt| dt.to_rfc3339())
}

/// Resolves a Date-valid timestamp with a Date-valid fallback.
pub fn resolve_date_timestamp_ms(value: &Value, fallback_value: &Value) -> i64 {
    as_date_timestamp_ms(value)
        .or_else(|| as_date_timestamp_ms(fallback_value))
        .unwrap_or(0)
}

/// Resolves a Date-valid timestamp to ISO, falling back to Unix epoch if needed.
pub fn resolve_timestamp_ms_to_iso_string(value: &Value, fallback_value: &Value) -> String {
    timestamp_ms_to_iso_string(value)
        .or_else(|| timestamp_ms_to_iso_string(fallback_value))
        .unwrap_or_else(|| UNIX_EPOCH_ISO_STRING.to_string())
}

/// Formats Date-valid timestamps for filenames by replacing colon separators.
pub fn timestamp_ms_to_iso_file_stamp(value: &Value, fallback_value: &Value) -> String {
    resolve_timestamp_ms_to_iso_string(value, fallback_value).replace(':', "-")
}

/// Clamps finite millisecond values into the Node-safe timer range.
pub fn clamp_timer_timeout_ms(value_ms: &Value, min_ms: i64) -> Option<i64> {
    let value = as_finite_number(value_ms)?;
    let min = std::cmp::max(1, min_ms);
    let clamped = (value.floor() as i64).max(min).min(MAX_TIMER_TIMEOUT_MS);
    Some(clamped)
}

/// Clamps positive finite millisecond values into the Node-safe timer range.
pub fn clamp_positive_timer_timeout_ms(value_ms: &Value) -> Option<i64> {
    let value = as_finite_number(value_ms)?;
    if value <= 0.0 {
        return None;
    }
    clamp_timer_timeout_ms(&Value::from(value), 1)
}

/// Resolves a positive timer timeout or falls back through safe timer clamping.
pub fn resolve_positive_timer_timeout_ms(value_ms: &Value, fallback_ms: i64) -> i64 {
    clamp_positive_timer_timeout_ms(value_ms)
        .unwrap_or_else(|| resolve_timer_timeout_ms(&Value::from(fallback_ms), 1, 0))
}

/// Resolves arbitrary timeout input with fallback and minimum timer bounds.
pub fn resolve_timer_timeout_ms(value_ms: &Value, fallback_ms: i64, min_ms: i64) -> i64 {
    let value = as_finite_number(value_ms).or_else(|| as_finite_number(&Value::from(fallback_ms)));
    let min = std::cmp::max(0, min_ms);
    match value {
        Some(v) => (v.floor() as i64).max(min).min(MAX_TIMER_TIMEOUT_MS),
        None => min,
    }
}

/// Adds grace time to a finite timeout and clamps the result to Node-safe bounds.
pub fn add_timer_timeout_grace_ms(timeout_ms: &Value, grace_ms: f64) -> Option<i64> {
    let timeout = as_finite_number(timeout_ms)?;
    let with_grace = timeout + grace_ms;
    if with_grace.is_finite() {
        clamp_timer_timeout_ms(&Value::from(with_grace), 1)
    } else {
        Some(MAX_TIMER_TIMEOUT_MS)
    }
}

/// finiteSecondsToTimerSafeMilliseconds options
#[derive(Debug, Clone, Default)]
pub struct FiniteSecondsOpts {
    pub floor_seconds: bool,
}

/// Converts finite positive seconds to Node-safe milliseconds.
pub fn finite_seconds_to_timer_safe_milliseconds(value: &Value, opts: &FiniteSecondsOpts) -> Option<i64> {
    let seconds = as_finite_number(value)?;
    if seconds <= 0.0 {
        return None;
    }
    let bounded_seconds = if opts.floor_seconds { seconds.floor() } else { seconds };
    let milliseconds = (bounded_seconds * 1000.0).floor() as i64;
    if milliseconds <= 0 {
        return None;
    }
    Some(milliseconds.min(MAX_TIMER_TIMEOUT_MS))
}

/// Resolves an integer option from finite numeric input or fallback, then clamps bounds.
pub fn resolve_integer_option(value: &Value, fallback: i64, min: Option<i64>, max: Option<i64>) -> i64 {
    let candidate = if let Some(n) = value.as_f64() {
        if n.is_finite() {
            n.floor() as i64
        } else {
            fallback
        }
    } else {
        fallback
    };
    let min_bounded = min.map(|m| m.max(candidate)).unwrap_or(candidate);
    max.map(|mx| mx.min(min_bounded)).unwrap_or(min_bounded)
}

/// Resolves an optional integer option, returning None for non-finite input.
pub fn resolve_optional_integer_option(value: &Value, min: Option<i64>, max: Option<i64>) -> Option<i64> {
    let n = value.as_f64()?;
    if !n.is_finite() {
        return None;
    }
    Some(resolve_integer_option(value, n as i64, min, max))
}

/// Resolves an integer option with a non-negative lower bound.
pub fn resolve_non_negative_integer_option(value: &Value, fallback: i64) -> i64 {
    resolve_integer_option(value, fallback, Some(0), None)
}

/// Parses strict positive integer values from numbers or strings.
pub fn parse_strict_positive_integer(value: &Value) -> Option<i64> {
    let parsed = parse_strict_integer(value)?;
    if parsed > 0 {
        Some(parsed)
    } else {
        None
    }
}

/// Parses strict non-negative integer values from numbers or strings.
pub fn parse_strict_non_negative_integer(value: &Value) -> Option<i64> {
    let parsed = parse_strict_integer(value)?;
    if parsed >= 0 {
        Some(parsed)
    } else {
        None
    }
}

/// Converts strict positive seconds to safe millisecond counts.
pub fn positive_seconds_to_safe_milliseconds(value: &Value) -> Option<i64> {
    let seconds = parse_strict_positive_integer(value)?;
    let milliseconds = seconds.checked_mul(1000)?;
    Some(milliseconds)
}

/// Converts strict non-negative seconds to safe millisecond counts.
pub fn non_negative_seconds_to_safe_milliseconds(value: &Value) -> Option<i64> {
    let seconds = parse_strict_non_negative_integer(value)?;
    let milliseconds = seconds.checked_mul(1000)?;
    Some(milliseconds)
}

/// resolveExpiresAt options
#[derive(Debug, Clone, Default)]
pub struct ExpiresAtOpts {
    pub now_ms: Option<i64>,
    pub buffer_ms: Option<f64>,
    pub min_remaining_ms: Option<i64>,
}

/// Resolves an absolute expiration timestamp from a positive duration in milliseconds.
pub fn resolve_expires_at_ms_from_duration_ms(value: &Value, opts: &ExpiresAtOpts) -> Option<i64> {
    let duration_ms = as_positive_safe_integer(value)?;
    let now_ms = opts
        .now_ms
        .or_else(|| Some(chrono::Utc::now().timestamp_millis()))
        .and_then(|n| as_date_timestamp_ms(&Value::from(n)))?;
    let buffer_ms = opts.buffer_ms.unwrap_or(0.0);

    let expires_at = now_ms + duration_ms - buffer_ms as i64;
    // 检查是否是有效日期
    if timestamp_ms_to_iso_string(&Value::from(expires_at)).is_none() {
        return None;
    }

    if let Some(min_remaining) = opts.min_remaining_ms {
        let min_expires_at = now_ms + min_remaining;
        if timestamp_ms_to_iso_string(&Value::from(min_expires_at)).is_none() {
            return Some(expires_at);
        }
        return Some(expires_at.max(min_expires_at));
    }
    Some(expires_at)
}

/// Resolves an absolute expiration timestamp from a positive duration in seconds.
pub fn resolve_expires_at_ms_from_duration_seconds(value: &Value, opts: &ExpiresAtOpts) -> Option<i64> {
    let duration_ms = positive_seconds_to_safe_milliseconds(value)?;
    resolve_expires_at_ms_from_duration_ms(&Value::from(duration_ms), opts)
}

/// Resolves an absolute expiration timestamp from Unix epoch seconds.
pub fn resolve_expires_at_ms_from_epoch_seconds(
    value: &Value,
    buffer_ms: Option<f64>,
    max_ms: Option<i64>,
) -> Option<i64> {
    let epoch_ms = if let Some(n) = value.as_f64() {
        if n.is_finite() && n > 0.0 {
            (n.trunc() as i64) * 1000
        } else {
            positive_seconds_to_safe_milliseconds(value)?
        }
    } else {
        positive_seconds_to_safe_milliseconds(value)?
    };

    let expires_at = epoch_ms - buffer_ms.unwrap_or(0.0) as i64;
    if timestamp_ms_to_iso_string(&Value::from(expires_at)).is_none() {
        return None;
    }
    match max_ms {
        Some(mx) if expires_at <= mx => Some(expires_at),
        Some(_) => None,
        None => Some(expires_at),
    }
}

/// Resolves expiration input that may be relative seconds, epoch seconds, or epoch milliseconds.
#[derive(Debug, Clone, Default)]
pub struct DurationOrEpochOpts {
    pub now_ms: Option<i64>,
    pub relative_seconds_threshold: Option<i64>,
    pub absolute_milliseconds_threshold: Option<i64>,
}

/// Resolves expiration input that may be relative seconds, epoch seconds, or epoch milliseconds.
pub fn resolve_expires_at_ms_from_duration_or_epoch(value: &Value, opts: &DurationOrEpochOpts) -> Option<i64> {
    let parsed = parse_strict_positive_integer(value)?;
    let relative_threshold = opts.relative_seconds_threshold.unwrap_or(1_000_000_000);
    if parsed < relative_threshold {
        return resolve_expires_at_ms_from_duration_seconds(value, &ExpiresAtOpts {
            now_ms: opts.now_ms,
            ..Default::default()
        });
    }
    let abs_threshold = opts.absolute_milliseconds_threshold.unwrap_or(1_000_000_000_000);
    if parsed < abs_threshold {
        return resolve_expires_at_ms_from_epoch_seconds(value, None, None);
    }
    as_date_timestamp_ms(value)
}
