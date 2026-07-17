// Agent Core module implements error types.
// 翻译自 packages/agent-core/src/errors.ts

pub const TRANSCRIPT_NOT_CONTINUABLE_ERROR_CODE: &str = "cradle_ring_transcript_not_continuable";

/// Error type thrown when the agent loop cannot continue from the current transcript role.
#[derive(Debug, Clone)]
pub struct TranscriptNotContinuableError {
    pub code: &'static str,
    pub role: String,
    pub message: String,
}

impl TranscriptNotContinuableError {
    pub fn new(role: &str) -> Self {
        Self {
            code: TRANSCRIPT_NOT_CONTINUABLE_ERROR_CODE,
            role: role.to_string(),
            message: format!("Cannot continue from message role: {}", role),
        }
    }
}

impl std::fmt::Display for TranscriptNotContinuableError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for TranscriptNotContinuableError {}