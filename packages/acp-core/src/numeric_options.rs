// ACP Core module implements numeric options behavior.
use normalization_core::number_coercion;

/// Resolves ACP integer options through the shared normalization contract.
pub fn resolve_integer_option(
    value: Option<f64>,
    fallback: i64,
    min: f64,
) -> i64 {
    let val = match value {
        Some(v) if v.is_finite() => serde_json::Value::from(v),
        _ => serde_json::Value::Null,
    };
    let min_i = if min.is_finite() { Some(min as i64) } else { None };
    number_coercion::resolve_integer_option(&val, fallback, min_i, None)
}