// ACP record normalization facade shared with older imports.
// 翻译自 packages/acp-core/src/record-shared.ts

use normalization_core::record_coerce;
use serde_json::{Map, Value};

/// Returns a non-array record or None.
pub fn as_record(value: &Value) -> Option<&Map<String, Value>> {
    record_coerce::as_optional_record(value)
}