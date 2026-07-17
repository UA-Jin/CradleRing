// Network Policy module implements ipv4 behavior.
// 翻译自 packages/net-policy/src/ipv4.ts

use crate::ip::is_canonical_dotted_decimal_ipv4;

/// Validates the custom-bind IPv4 input and returns the user-facing error text.
pub fn validate_dotted_decimal_ipv4_input(value: Option<&str>) -> Option<String> {
    match value {
        None => return Some("IP address is required for custom bind mode".to_string()),
        Some(v) if v.is_empty() => return Some("IP address is required for custom bind mode".to_string()),
        _ => {}
    }
    if is_canonical_dotted_decimal_ipv4(value) {
        None
    } else {
        Some("Invalid IPv4 address (e.g., 192.168.1.100)".to_string())
    }
}