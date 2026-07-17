// LLM core diagnostics helpers.
// 翻译自 packages/llm-core/src/utils/diagnostics.ts

use std::collections::BTreeMap;

/// Diagnostic error code (string or numeric).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(untagged)]
pub enum DiagnosticErrorCode {
    Str(String),
    Num(f64),
}

/// Diagnostic error info extracted from a thrown value.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiagnosticErrorInfo {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stack: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<DiagnosticErrorCode>,
}

/// Assistant message diagnostic entry.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AssistantMessageDiagnostic {
    #[serde(rename = "type")]
    pub type_: String,
    pub timestamp: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<DiagnosticErrorInfo>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<BTreeMap<String, serde_json::Value>>,
}

/// A simple error type implementing the diagnostic-error shape used by
/// `extract_diagnostic_error` and `create_assistant_message_diagnostic`.
#[derive(Debug, Clone)]
pub struct DiagnosticError {
    pub name: String,
    pub message: String,
    pub stack: Option<String>,
    pub code: Option<DiagnosticErrorCode>,
}

impl std::fmt::Display for DiagnosticError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.name, self.message)
    }
}

impl std::error::Error for DiagnosticError {}

/// Extracts serializable diagnostic error fields from a DiagnosticError.
pub fn extract_diagnostic_error(error: &DiagnosticError) -> DiagnosticErrorInfo {
    DiagnosticErrorInfo {
        name: if error.name.is_empty() {
            None
        } else {
            Some(error.name.clone())
        },
        message: if !error.message.is_empty() {
            error.message.clone()
        } else {
            error.name.clone()
        },
        stack: error.stack.clone(),
        code: error.code.clone(),
    }
}

/// Creates a timestamped assistant-message diagnostic entry.
pub fn create_assistant_message_diagnostic(
    diagnostic_type: &str,
    error: &DiagnosticError,
    details: Option<BTreeMap<String, serde_json::Value>>,
) -> AssistantMessageDiagnostic {
    AssistantMessageDiagnostic {
        type_: diagnostic_type.to_string(),
        timestamp: chrono::Utc::now().timestamp_millis(),
        error: Some(extract_diagnostic_error(error)),
        details,
    }
}

/// Appends a diagnostic while preserving existing message diagnostics.
pub fn append_assistant_message_diagnostic(
    diagnostics: &mut Option<Vec<AssistantMessageDiagnostic>>,
    diagnostic: AssistantMessageDiagnostic,
) {
    let list = diagnostics.get_or_insert_with(Vec::new);
    list.push(diagnostic);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_diagnostic_error_uses_name_fallback() {
        let err = DiagnosticError {
            name: "X".to_string(),
            message: String::new(),
            stack: None,
            code: None,
        };
        let info = extract_diagnostic_error(&err);
        assert_eq!(info.name.as_deref(), Some("X"));
        assert_eq!(info.message, "X");
    }

    #[test]
    fn create_assistant_message_diagnostic_populates_timestamp() {
        let err = DiagnosticError {
            name: "TypeError".to_string(),
            message: "boom".to_string(),
            stack: None,
            code: Some(DiagnosticErrorCode::Str("EBOOM".to_string())),
        };
        let diag = create_assistant_message_diagnostic("error", &err, None);
        assert_eq!(diag.type_, "error");
        assert!(diag.timestamp > 0);
        assert!(diag.error.is_some());
    }
}