// Normalizes an unknown thrown value into an Error.
// 翻译自 packages/normalization-core/src/error-coercion.ts

use std::fmt;

/// CradleRing 错误类型（对标 JS Error）
#[derive(Debug, Clone)]
pub struct CrError {
    pub message: String,
    pub cause: Option<String>,
}

impl fmt::Display for CrError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for CrError {}

impl CrError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            cause: None,
        }
    }

    pub fn with_cause(message: impl Into<String>, cause: impl ToString) -> Self {
        Self {
            message: message.into(),
            cause: Some(cause.to_string()),
        }
    }
}

/// Normalizes an unknown thrown value into an Error. Non-Error objects become
/// the `cause` and have their enumerable fields copied so structured details
/// (codes, statuses) survive the coercion.
pub fn to_error_object(value: &serde_json::Value, fallback_message: &str) -> CrError {
    if let Some(s) = value.as_str() {
        return CrError::new(s);
    }
    // Rust 没有 instanceof Error；任何非字符串值都走 fallback + cause
    CrError::with_cause(fallback_message, value.to_string())
}
