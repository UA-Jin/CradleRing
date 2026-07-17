// Gateway Protocol: secret-ref contract constants.
// 翻译自 packages/gateway-protocol/src/secret-ref-contract.ts
//
// Shared alias grammar and JSON-schema regex fragments used by env/file/exec
// secret provider implementations. Centralized so multiple schemas (secrets,
// primitives, etc.) can reference one canonical source.

/// Canonical id for file secret providers that expose exactly one value.
/// 对齐 TS: `export const SINGLE_VALUE_FILE_REF_ID = "value";`.
pub const SINGLE_VALUE_FILE_REF_ID: &str = "value";

/// Shared alias grammar for env/file/exec secret provider names.
/// 对齐 TS: `export const SECRET_PROVIDER_ALIAS_PATTERN = /^[a-z][a-z0-9_-]{0,63}$/;`.
pub const SECRET_PROVIDER_ALIAS_PATTERN: &str = r"^[a-z][a-z0-9_-]{0,63}$";

/// JSON-schema fragment that rejects absolute file secret ref ids.
/// 对齐 TS: `export const FILE_SECRET_REF_ID_ABSOLUTE_JSON_SCHEMA_PATTERN = "^/";`.
pub const FILE_SECRET_REF_ID_ABSOLUTE_JSON_SCHEMA_PATTERN: &str = r"^/";

/// JSON-schema fragment that rejects invalid JSON-pointer escape sequences.
/// 对齐 TS: `export const FILE_SECRET_REF_ID_INVALID_ESCAPE_JSON_SCHEMA_PATTERN = "~(?:[^01]|$)";`.
pub const FILE_SECRET_REF_ID_INVALID_ESCAPE_JSON_SCHEMA_PATTERN: &str = r"~(?:[^01]|$)";

/// JSON-schema pattern for exec secret ref ids, excluding dot-path traversal.
/// 对齐 TS:
///   `export const EXEC_SECRET_REF_ID_JSON_SCHEMA_PATTERN =
///      "^(?!.*(?:^|/)\\.{1,2}(?:/|$))[A-Za-z0-9][A-Za-z0-9._:/#-]{0,255}$";`.
pub const EXEC_SECRET_REF_ID_JSON_SCHEMA_PATTERN: &str =
    r"^(?!.*(?:^|/)\.{1,2}(?:/|$))[A-Za-z0-9][A-Za-z0-9._:/#-]{0,255}$";
