//! Hash helpers.
//! 翻译自 packages/ai/src/utils/hash.ts
//!
//! Computes a short hex-encoded SHA-1 fingerprint of arbitrary bytes
//! (mirrors the TS implementation that uses crypto.createHash("sha1")).

use sha1::{Digest, Sha1};

/// Returns the hex-encoded SHA-1 hash of `value`.
pub fn sha1_hex(value: &str) -> String {
    let mut hasher = Sha1::new();
    hasher.update(value.as_bytes());
    hex::encode(hasher.finalize())
}

/// Returns the first 12 hex characters of SHA-1(value), matching the TS short-hash helper.
pub fn short_sha1(value: &str) -> String {
    let digest = sha1_hex(value);
    digest.chars().take(12).collect()
}