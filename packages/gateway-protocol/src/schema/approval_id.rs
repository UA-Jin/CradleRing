// Gateway Protocol schema: approval_id.
// 翻译自 packages/gateway-protocol/src/schema/approval-id.ts
//
// Gateway protocol approval IDs stay exact and safe for encoded deep-link path segments.
//
// TS 导出:
//   - `APPROVAL_ID_WELL_FORMED_UNICODE_PATTERN`
//   - `isWellFormedApprovalId(value)`
// Rust 等价物:
//   - `APPROVAL_ID_WELL_FORMED_UNICODE_PATTERN`
//   - `is_well_formed_approval_id(value)`

// ---------- Pattern constants ----------

/// Regex source mirroring the TS constant.
/// 对齐 TS:
///   `APPROVAL_ID_WELL_FORMED_UNICODE_PATTERN =
///      "^(?!\\.{1,2}$)(?:[^\\uD800-\\uDFFF]|[\\uD800-\\uDBFF][\\uDC00-\\uDFFF])+$"`.
pub const APPROVAL_ID_WELL_FORMED_UNICODE_PATTERN: &str =
    r"^(?!\.{1,2}$)(?:[^\u{D800}-\u{DFFF}]|[\u{D800}-\u{DBFF}][\u{DC00}-\u{DFFF}])+$";

// ---------- Validation primitives ----------

/// Returns true when `value` is a valid approval id:
/// - non-empty
/// - not equal to "." or ".."
/// - contains no unpaired UTF-16 surrogate code units (mirrors the TS check
///   that iterates `charCodeAt` code units).
///
/// 对齐 TS: `isWellFormedApprovalId(value: string): boolean`.
pub fn is_well_formed_approval_id(value: &str) -> bool {
    if value.is_empty() || value == "." || value == ".." {
        return false;
    }
    // Iterate UTF-16 code units exactly like `value.charCodeAt(index)` does.
    let code_units: Vec<u16> = value.encode_utf16().collect();
    let mut index = 0;
    while index < code_units.len() {
        let code_unit = code_units[index];
        if (0xD800..=0xDBFF).contains(&code_unit) {
            // High surrogate: must be followed by a low surrogate code unit.
            if index + 1 >= code_units.len() {
                return false;
            }
            let next = code_units[index + 1];
            if !(0xDC00..=0xDFFF).contains(&next) {
                return false;
            }
            index += 2;
        } else if (0xDC00..=0xDFFF).contains(&code_unit) {
            // Stray low surrogate without a paired high surrogate.
            return false;
        } else {
            index += 1;
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_empty_and_dot_paths() {
        assert!(!is_well_formed_approval_id(""));
        assert!(!is_well_formed_approval_id("."));
        assert!(!is_well_formed_approval_id(".."));
    }

    #[test]
    fn accepts_ascii_identifier() {
        assert!(is_well_formed_approval_id("approval-123"));
    }

    #[test]
    fn accepts_paired_supplementary_char() {
        // U+1F600 GRINNING FACE — encoded as a valid surrogate pair in UTF-16.
        assert!(is_well_formed_approval_id("\u{1F600}"));
    }
}