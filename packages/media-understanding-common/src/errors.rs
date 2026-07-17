// Media-understanding skip error used for non-fatal attachment omissions.
// 翻译自 packages/media-understanding-common/src/errors.ts

use std::fmt;

/** Reason a media-understanding attachment was skipped. */
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MediaUnderstandingSkipReason {
    MaxBytes,
    Timeout,
    Unsupported,
    Empty,
    Blocked,
    TooSmall,
}

/** Error used when a media attachment should be skipped without failing the whole request. */
#[derive(Debug)]
pub struct MediaUnderstandingSkipError {
    pub reason: MediaUnderstandingSkipReason,
    pub message: String,
}

impl MediaUnderstandingSkipError {
    pub fn new(reason: MediaUnderstandingSkipReason, message: impl Into<String>) -> Self {
        Self {
            reason,
            message: message.into(),
        }
    }
}

impl fmt::Display for MediaUnderstandingSkipError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for MediaUnderstandingSkipError {}

/** Narrow unknown errors to media-understanding skip errors. */
pub fn is_media_understanding_skip_error(err: &(dyn std::error::Error + 'static)) -> bool {
    err.downcast_ref::<MediaUnderstandingSkipError>().is_some()
}