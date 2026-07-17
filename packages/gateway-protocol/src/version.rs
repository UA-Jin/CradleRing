// Gateway Protocol: version constants.
// 翻译自 packages/gateway-protocol/src/version.ts
//
// The current and minimum supported gateway protocol versions. Exposed both as
// `i64` constants and as compile-time literals (via inline assertions) so
// consumers can pattern-match on them.

/// Current gateway protocol version emitted by modern clients and servers.
/// 对齐 TS: `export const PROTOCOL_VERSION = 4 as const;`.
pub const PROTOCOL_VERSION: i64 = 4;

/// Lowest general client protocol version accepted by the gateway.
/// 对齐 TS: `export const MIN_CLIENT_PROTOCOL_VERSION = 4 as const;`.
pub const MIN_CLIENT_PROTOCOL_VERSION: i64 = 4;

/// Lowest authenticated node protocol version accepted by the gateway.
/// 对齐 TS: `export const MIN_NODE_PROTOCOL_VERSION = 3 as const;`.
pub const MIN_NODE_PROTOCOL_VERSION: i64 = 3;

/// Lowest lightweight probe protocol version accepted by the gateway.
/// 对齐 TS: `export const MIN_PROBE_PROTOCOL_VERSION = 3 as const;`.
pub const MIN_PROBE_PROTOCOL_VERSION: i64 = 3;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn versions_are_positive() {
        assert!(PROTOCOL_VERSION > 0);
        assert!(MIN_CLIENT_PROTOCOL_VERSION > 0);
        assert!(MIN_NODE_PROTOCOL_VERSION > 0);
        assert!(MIN_PROBE_PROTOCOL_VERSION > 0);
    }

    #[test]
    fn minimums_do_not_exceed_current() {
        assert!(MIN_CLIENT_PROTOCOL_VERSION <= PROTOCOL_VERSION);
        assert!(MIN_NODE_PROTOCOL_VERSION <= PROTOCOL_VERSION);
        assert!(MIN_PROBE_PROTOCOL_VERSION <= PROTOCOL_VERSION);
    }
}
