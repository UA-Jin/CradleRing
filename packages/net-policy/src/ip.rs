// Network Policy module implements ip behavior.
// 翻译自 packages/net-policy/src/ip.ts

use std::collections::HashSet;
use std::sync::OnceLock;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum IpKind {
    Ipv4,
    Ipv6,
}

/// Parsed IP address value returned by the net-policy parsing helpers.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ParsedIpAddress {
    Ipv4(Ipv4Addr),
    Ipv6(Ipv6Addr),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Ipv4Addr {
    octets: [u8; 4],
}

impl Ipv4Addr {
    pub fn new(a: u8, b: u8, c: u8, d: u8) -> Self {
        Self {
            octets: [a, b, c, d],
        }
    }
    pub fn octets(&self) -> [u8; 4] {
        self.octets
    }
    pub fn to_u32(&self) -> u32 {
        u32::from(self.octets[0]) << 24
            | u32::from(self.octets[1]) << 16
            | u32::from(self.octets[2]) << 8
            | u32::from(self.octets[3])
    }
    pub fn range(&self) -> Ipv4Range {
        ipv4_range(*self)
    }
    pub fn match_prefix(&self, base: Ipv4Addr, prefix: u8) -> bool {
        if prefix > 32 {
            return false;
        }
        let mask = if prefix == 0 { 0u32 } else { !0u32 << (32 - prefix) };
        (self.to_u32() & mask) == base.to_u32()
    }
}

impl std::fmt::Display for Ipv4Addr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}.{}.{}.{}",
            self.octets[0], self.octets[1], self.octets[2], self.octets[3]
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Ipv6Addr {
    hextets: [u16; 8],
}

impl Ipv6Addr {
    pub fn new(hextets: [u16; 8]) -> Self {
        Self { hextets }
    }
    pub fn parts(&self) -> [u16; 8] {
        self.hextets
    }
    pub fn range(&self) -> Ipv6Range {
        ipv6_range(*self)
    }
    pub fn is_ipv4_mapped_address(&self) -> bool {
        // ::ffff:a.b.c.d
        self.hextets[0] == 0
            && self.hextets[1] == 0
            && self.hextets[2] == 0
            && self.hextets[3] == 0
            && self.hextets[4] == 0
            && self.hextets[5] == 0xffff
    }
    pub fn to_ipv4_address(&self) -> Ipv4Addr {
        if self.is_ipv4_mapped_address() {
            Ipv4Addr::new(
                (self.hextets[6] >> 8) as u8,
                (self.hextets[6] & 0xff) as u8,
                (self.hextets[7] >> 8) as u8,
                (self.hextets[7] & 0xff) as u8,
            )
        } else {
            Ipv4Addr::new(0, 0, 0, 0)
        }
    }
    pub fn match_prefix(&self, base: Ipv6Addr, prefix: u8) -> bool {
        if prefix > 128 {
            return false;
        }
        let full = (prefix / 16) as usize;
        let rem = prefix % 16;
        for i in 0..full {
            if self.hextets[i] != base.hextets[i] {
                return false;
            }
        }
        if rem > 0 && full < 8 {
            let mask: u16 = !0u16 << (16 - rem);
            if (self.hextets[full] & mask) != (base.hextets[full] & mask) {
                return false;
            }
        }
        true
    }
}

impl std::fmt::Display for Ipv6Addr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Find longest run of zeros
        let mut best_start: Option<usize> = None;
        let mut best_len = 0usize;
        let mut cur_start: Option<usize> = None;
        let mut cur_len = 0usize;
        for (i, h) in self.hextets.iter().enumerate() {
            if *h == 0 {
                if cur_start.is_none() {
                    cur_start = Some(i);
                }
                cur_len += 1;
                if cur_len > best_len && cur_len >= 2 {
                    best_len = cur_len;
                    best_start = cur_start;
                }
            } else {
                cur_start = None;
                cur_len = 0;
            }
        }
        let parts: Vec<String> = self
            .hextets
            .iter()
            .map(|h| format!("{:x}", h))
            .collect();
        match best_start {
            Some(start) => {
                let mut s = String::new();
                if start > 0 {
                    s.push_str(&parts[..start].join(":"));
                }
                s.push_str("::");
                let end = start + best_len;
                if end < 8 {
                    s.push_str(&parts[end..].join(":"));
                }
                write!(f, "{}", s)
            }
            None => write!(f, "{}", parts.join(":")),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Ipv4Range {
    Unspecified,
    Broadcast,
    Multicast,
    LinkLocal,
    Loopback,
    CarrierGradeNat,
    Private,
    Reserved,
    Unicast,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Ipv6Range {
    Unspecified,
    Loopback,
    LinkLocal,
    UniqueLocal,
    Multicast,
    Reserved,
    Benchmarking,
    Discard,
    Orchid2,
    Rfc6145,
    Rfc6052,
    Unicast,
}

fn ipv4_range(addr: Ipv4Addr) -> Ipv4Range {
    let v = addr.to_u32();
    if v == 0 {
        return Ipv4Range::Unspecified;
    }
    if v == 0xffffffff {
        return Ipv4Range::Broadcast;
    }
    // 224.0.0.0/4 multicast
    if (v >> 28) == 0xe {
        return Ipv4Range::Multicast;
    }
    // 169.254.0.0/16 link-local
    if (v >> 16) == 0xa9fe {
        return Ipv4Range::LinkLocal;
    }
    // 127.0.0.0/8 loopback
    if (v >> 24) == 0x7f {
        return Ipv4Range::Loopback;
    }
    // 100.64.0.0/10 carrier-grade NAT
    if (v & 0xffc0_0000) == 0x6440_0000 {
        return Ipv4Range::CarrierGradeNat;
    }
    // 10.0.0.0/8, 172.16.0.0/12, 192.168.0.0/16 private
    if (v >> 24) == 10 || (v >> 20) == 0xac1 || (v >> 16) == 0xc0a8 {
        return Ipv4Range::Private;
    }
    // 240.0.0.0/4 reserved (includes 255.255.255.255 broadcast handled above)
    if (v >> 28) == 0xf {
        return Ipv4Range::Reserved;
    }
    Ipv4Range::Unicast
}

fn ipv6_range(addr: Ipv6Addr) -> Ipv6Range {
    let h = addr.hextets;
    if h == [0, 0, 0, 0, 0, 0, 0, 0] {
        return Ipv6Range::Unspecified;
    }
    if h == [0, 0, 0, 0, 0, 0, 0, 1] {
        return Ipv6Range::Loopback;
    }
    // fe80::/10 link-local
    if (h[0] & 0xffc0) == 0xfe80 {
        return Ipv6Range::LinkLocal;
    }
    // fc00::/7 unique local
    if (h[0] & 0xfe00) == 0xfc00 {
        return Ipv6Range::UniqueLocal;
    }
    // ff00::/8 multicast
    if (h[0] & 0xff00) == 0xff00 {
        return Ipv6Range::Multicast;
    }
    // 2001:db8::/32 benchmarking/documentation (used as example)
    if h[0] == 0x2001 && h[1] == 0x0db8 {
        return Ipv6Range::Benchmarking;
    }
    // 2001:0000::/32 Teredo
    if h[0] == 0x2001 && h[1] == 0x0000 {
        return Ipv6Range::Reserved;
    }
    // 100::/64 discard
    if h[0] == 0x0100 && h[1] == 0 && h[2] == 0 && h[3] == 0 {
        return Ipv6Range::Discard;
    }
    // 2001:20::/28 orchid2
    if h[0] == 0x2001 && (h[1] >> 4) == 0x2 {
        return Ipv6Range::Orchid2;
    }
    // 64:ff9b:1::/48 NAT64
    if h[0] == 0x0064 && h[1] == 0xff9b && h[2] == 0x0001 {
        return Ipv6Range::Rfc6052;
    }
    // rfc6145 well-known prefix 64:ff9b::/96
    if h[0] == 0x0064 && h[1] == 0xff9b && h[2] == 0 && h[3] == 0 && h[4] == 0 && h[5] == 0 {
        return Ipv6Range::Rfc6145;
    }
    Ipv6Range::Unicast
}

fn normalize_optional_string(value: Option<&str>) -> Option<String> {
    let trimmed = value?.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

fn normalize_lowercase_string_or_empty(value: &str) -> String {
    value.trim().to_lowercase()
}

/// Type guard for parsed IPv4 addresses.
pub fn is_ipv4_address(address: &ParsedIpAddress) -> bool {
    matches!(address, ParsedIpAddress::Ipv4(_))
}

/// Type guard for parsed IPv6 addresses.
pub fn is_ipv6_address(address: &ParsedIpAddress) -> bool {
    matches!(address, ParsedIpAddress::Ipv6(_))
}

fn normalize_ipv4_mapped_address(address: &ParsedIpAddress) -> ParsedIpAddress {
    if let ParsedIpAddress::Ipv6(addr) = address {
        if addr.is_ipv4_mapped_address() {
            return ParsedIpAddress::Ipv4(addr.to_ipv4_address());
        }
    }
    address.clone()
}

fn strip_ipv6_brackets(value: &str) -> String {
    if value.starts_with('[') && value.ends_with(']') {
        value[1..value.len() - 1].to_string()
    } else {
        value.to_string()
    }
}

fn normalize_ip_parse_input(raw: Option<&str>) -> Option<String> {
    let trimmed = normalize_optional_string(raw)?;
    Some(strip_ipv6_brackets(&trimmed))
}

fn is_numeric_ipv4_literal_part(value: &str) -> bool {
    if value.is_empty() {
        return false;
    }
    if value.starts_with("0x") || value.starts_with("0X") {
        let rest = &value[2..];
        !rest.is_empty() && rest.chars().all(|c| c.is_ascii_hexdigit())
    } else {
        value.chars().all(|c| c.is_ascii_digit())
    }
}

fn parse_ipv4_strict(s: &str) -> Option<Ipv4Addr> {
    let parts: Vec<&str> = s.split('.').collect();
    if parts.len() != 4 {
        return None;
    }
    let mut octets = [0u8; 4];
    for (i, p) in parts.iter().enumerate() {
        if p.is_empty() {
            return None;
        }
        let v = if p.starts_with("0x") || p.starts_with("0X") {
            u32::from_str_radix(&p[2..], 16).ok()?
        } else {
            p.parse::<u32>().ok()?
        };
        if v > 255 {
            return None;
        }
        octets[i] = v as u8;
    }
    Some(Ipv4Addr::new(octets[0], octets[1], octets[2], octets[3]))
}

fn parse_ipv4_loose(s: &str) -> Option<Ipv4Addr> {
    let parts: Vec<&str> = s.split('.').collect();
    if parts.is_empty() || parts.len() > 4 {
        return None;
    }
    if parts.iter().any(|p| p.is_empty()) {
        return None;
    }
    if !parts.iter().all(|p| is_numeric_ipv4_literal_part(p)) {
        return None;
    }
    let mut numbers = [0u32; 4];
    let mut idx = 0;
    for (i, p) in parts.iter().enumerate() {
        let v = if p.starts_with("0x") || p.starts_with("0X") {
            u32::from_str_radix(&p[2..], 16).ok()?
        } else {
            p.parse::<u32>().ok()?
        };
        // Match ipaddr.js: legacy forms pack left-to-right, but the last
        // group can be 8-bit, others up to 8-bit per position (or 32-bit
        // when alone).
        if i < parts.len() - 1 {
            if v > 0xff {
                return None;
            }
        } else if parts.len() != 1 {
            if v > 0xff {
                return None;
            }
        }
        numbers[idx] = v;
        idx += 1;
    }
    let n = parts.len();
    let mut octets = [0u8; 4];
    if n == 1 {
        let v = numbers[0];
        octets[0] = ((v >> 24) & 0xff) as u8;
        octets[1] = ((v >> 16) & 0xff) as u8;
        octets[2] = ((v >> 8) & 0xff) as u8;
        octets[3] = (v & 0xff) as u8;
    } else if n == 2 {
        octets[0] = numbers[0] as u8;
        octets[1] = numbers[1] as u8;
    } else if n == 3 {
        octets[0] = numbers[0] as u8;
        octets[1] = numbers[1] as u8;
        octets[2] = numbers[2] as u8;
    } else {
        octets[0] = numbers[0] as u8;
        octets[1] = numbers[1] as u8;
        octets[2] = numbers[2] as u8;
        octets[3] = numbers[3] as u8;
    }
    Some(Ipv4Addr::new(octets[0], octets[1], octets[2], octets[3]))
}

fn parse_ipv6_loose(s: &str) -> Option<Ipv6Addr> {
    // Handle :: shorthand
    if s.contains("::") {
        let parts: Vec<&str> = s.split("::").collect();
        if parts.len() > 2 {
            return None;
        }
        let left_str = parts[0];
        let right_str = parts[1];
        let left: Vec<u16> = if left_str.is_empty() {
            vec![]
        } else {
            left_str.split(':').map(|p| u16::from_str_radix(p, 16).ok()).collect::<Option<Vec<_>>>()?
        };
        let right: Vec<u16> = if right_str.is_empty() {
            vec![]
        } else {
            right_str.split(':').map(|p| u16::from_str_radix(p, 16).ok()).collect::<Option<Vec<_>>>()?
        };
        if left.len() + right.len() > 8 {
            return None;
        }
        let mut hextets = [0u16; 8];
        for (i, h) in left.iter().enumerate() {
            hextets[i] = *h;
        }
        for (i, h) in right.iter().enumerate() {
            hextets[8 - right.len() + i] = *h;
        }
        return Some(Ipv6Addr::new(hextets));
    }
    let parts: Vec<&str> = s.split(':').collect();
    if parts.len() != 8 {
        return None;
    }
    let mut hextets = [0u16; 8];
    for (i, p) in parts.iter().enumerate() {
        let v = u16::from_str_radix(p, 16).ok()?;
        hextets[i] = v;
    }
    Some(Ipv6Addr::new(hextets))
}

fn parse_ipv6_with_embedded_ipv4(raw: &str) -> Option<ParsedIpAddress> {
    if !raw.contains(':') || !raw.contains('.') {
        return None;
    }
    let re = regex::Regex::new(r"^(.*:)([^:%]+(?:\.[^:%]+){3})(%[0-9A-Za-z]+)?$").ok()?;
    let caps = re.captures(raw)?;
    let prefix = caps.get(1)?.as_str();
    let embedded_ipv4 = caps.get(2)?.as_str();
    let zone_suffix = caps.get(3).map(|m| m.as_str()).unwrap_or("");
    if !is_canonical_four_part_decimal(embedded_ipv4) {
        return None;
    }
    let parts: Vec<&str> = embedded_ipv4.split('.').collect();
    let a: u32 = parts[0].parse().ok()?;
    let b: u32 = parts[1].parse().ok()?;
    let c: u32 = parts[2].parse().ok()?;
    let d: u32 = parts[3].parse().ok()?;
    let high = format!("{:x}", (a << 8) | b);
    let low = format!("{:x}", (c << 8) | d);
    let normalized = format!("{}{}:{}{}", prefix, high, low, zone_suffix);
    let parsed = parse_ipv6_loose(&normalized)?;
    Some(ParsedIpAddress::Ipv6(parsed))
}

fn is_canonical_four_part_decimal(s: &str) -> bool {
    let parts: Vec<&str> = s.split('.').collect();
    if parts.len() != 4 {
        return false;
    }
    for p in &parts {
        if p.is_empty() {
            return false;
        }
        if !p.chars().all(|c| c.is_ascii_digit()) {
            return false;
        }
        // disallow leading zeros (canonical form only)
        if p.len() > 1 && p.starts_with('0') {
            return false;
        }
        let v: u32 = match p.parse() {
            Ok(v) => v,
            Err(_) => return false,
        };
        if v > 255 {
            return false;
        }
    }
    true
}

/// Parses canonical IPv4/IPv6 literals, rejecting legacy IPv4 shorthand forms.
pub fn parse_canonical_ip_address(raw: Option<&str>) -> Option<ParsedIpAddress> {
    let normalized = normalize_ip_parse_input(raw)?;
    if let Some(v4) = parse_ipv4_strict(&normalized) {
        if !is_canonical_four_part_decimal(&normalized) {
            return None;
        }
        return Some(ParsedIpAddress::Ipv4(v4));
    }
    if let Some(v6) = parse_ipv6_loose(&normalized) {
        return Some(ParsedIpAddress::Ipv6(v6));
    }
    parse_ipv6_with_embedded_ipv4(&normalized)
}

/// Parses canonical IP literals plus legacy IPv4 forms needed for SSRF checks.
pub fn parse_loose_ip_address(raw: Option<&str>) -> Option<ParsedIpAddress> {
    let normalized = normalize_ip_parse_input(raw)?;
    if let Some(v4) = parse_ipv4_loose(&normalized) {
        return Some(ParsedIpAddress::Ipv4(v4));
    }
    if let Some(v6) = parse_ipv6_loose(&normalized) {
        return Some(ParsedIpAddress::Ipv6(v6));
    }
    parse_ipv6_with_embedded_ipv4(&normalized)
}

/// Normalizes canonical IP literals and maps IPv4-mapped IPv6 addresses to IPv4 text.
pub fn normalize_ip_address(raw: Option<&str>) -> Option<String> {
    let parsed = parse_canonical_ip_address(raw)?;
    let normalized = normalize_ipv4_mapped_address(&parsed);
    let s = normalize_lowercase_string_or_empty(&normalized.to_string());
    if s.is_empty() {
        None
    } else {
        Some(s)
    }
}

/// True only for canonical four-part dotted-decimal IPv4 literals.
pub fn is_canonical_dotted_decimal_ipv4(raw: Option<&str>) -> bool {
    let trimmed = normalize_optional_string(raw);
    let Some(trimmed) = trimmed else {
        return false;
    };
    let normalized = strip_ipv6_brackets(&trimmed);
    if normalized.is_empty() {
        return false;
    }
    is_canonical_four_part_decimal(&normalized)
}

/// Detects legacy numeric IPv4 forms that canonical parsing deliberately rejects.
pub fn is_legacy_ipv4_literal(raw: Option<&str>) -> bool {
    let trimmed = normalize_optional_string(raw);
    let Some(trimmed) = trimmed else {
        return false;
    };
    let normalized = strip_ipv6_brackets(&trimmed);
    if normalized.is_empty() || normalized.contains(':') {
        return false;
    }
    if is_canonical_dotted_decimal_ipv4(Some(&normalized)) {
        return false;
    }
    let parts: Vec<&str> = normalized.split('.').collect();
    if parts.is_empty() || parts.len() > 4 {
        return false;
    }
    if parts.iter().any(|p| p.is_empty()) {
        return false;
    }
    if !parts.iter().all(|p| is_numeric_ipv4_literal_part(p)) {
        return false;
    }
    true
}

/// True when a canonical IP literal is loopback, including IPv4-mapped IPv6.
pub fn is_loopback_ip_address(raw: Option<&str>) -> bool {
    let Some(parsed) = parse_canonical_ip_address(raw) else {
        return false;
    };
    let normalized = normalize_ipv4_mapped_address(&parsed);
    matches!(normalized.range_ip(), Some(RangeClass::Ipv4Loopback))
}

/// True for link-local IPs, including legacy and embedded-IPv4 forms.
pub fn is_link_local_ip_address(raw: Option<&str>) -> bool {
    let Some(parsed) = parse_loose_ip_address(raw) else {
        return false;
    };
    let normalized = normalize_ipv4_mapped_address(&parsed);
    if let ParsedIpAddress::Ipv4(v4) = &normalized {
        return matches!(v4.range(), Ipv4Range::LinkLocal);
    }
    if let ParsedIpAddress::Ipv6(v6) = &normalized {
        if let Some(emb) = extract_embedded_ipv4_from_ipv6(v6) {
            if matches!(emb.range(), Ipv4Range::LinkLocal) {
                return true;
            }
        }
        return matches!(v6.range(), Ipv6Range::LinkLocal);
    }
    false
}

/// True for cloud metadata IP literals, including mapped and embedded forms.
pub fn is_cloud_metadata_ip_address(raw: Option<&str>) -> bool {
    let Some(parsed) = parse_loose_ip_address(raw) else {
        return false;
    };
    let normalized = normalize_ipv4_mapped_address(&parsed);
    if let ParsedIpAddress::Ipv6(v6) = &normalized {
        if let Some(emb) = extract_embedded_ipv4_from_ipv6(v6) {
            if cloud_metadata_ip_addresses().contains(&emb.to_string().as_str()) {
                return true;
            }
        }
    }
    cloud_metadata_ip_addresses().contains(&normalized.to_string().as_str())
}

fn cloud_metadata_ip_addresses() -> &'static HashSet<&'static str> {
    static SET: OnceLock<HashSet<&'static str>> = OnceLock::new();
    SET.get_or_init(|| {
        let mut s = HashSet::new();
        s.insert("100.100.100.200");
        s.insert("fd00:ec2::254");
        s
    })
}

/// True for canonical private, loopback, link-local, or blocked special-use IPs.
pub fn is_private_or_loopback_ip_address(raw: Option<&str>) -> bool {
    let Some(parsed) = parse_canonical_ip_address(raw) else {
        return false;
    };
    let normalized = normalize_ipv4_mapped_address(&parsed);
    if let ParsedIpAddress::Ipv4(v4) = &normalized {
        return private_or_loopback_ipv4_ranges().contains(&v4.range());
    }
    if let ParsedIpAddress::Ipv6(v6) = &normalized {
        return is_blocked_special_use_ipv6_address(v6, &Ipv6SpecialUseBlockOptions::default());
    }
    false
}

/// Applies the SSRF block policy for parsed IPv6 special-use ranges.
pub fn is_blocked_special_use_ipv6_address(
    address: &Ipv6Addr,
    options: &Ipv6SpecialUseBlockOptions,
) -> bool {
    let range = address.range();
    if range == Ipv6Range::UniqueLocal && options.allow_unique_local_range == Some(true) {
        return false;
    }
    if blocked_ipv6_special_use_ranges().contains(&range) {
        return true;
    }
    // Deprecated site-local fec0::/10
    (address.hextets[0] & 0xffc0) == 0xfec0
}

/// True for canonical IPv4 literals in RFC 1918 private ranges.
pub fn is_rfc1918_ipv4_address(raw: Option<&str>) -> bool {
    let Some(parsed) = parse_canonical_ip_address(raw) else {
        return false;
    };
    if let ParsedIpAddress::Ipv4(v4) = parsed {
        return matches!(v4.range(), Ipv4Range::Private);
    }
    false
}

/// True for canonical IPv4 literals in the carrier-grade NAT range.
pub fn is_carrier_grade_nat_ipv4_address(raw: Option<&str>) -> bool {
    let Some(parsed) = parse_canonical_ip_address(raw) else {
        return false;
    };
    if let ParsedIpAddress::Ipv4(v4) = parsed {
        return matches!(v4.range(), Ipv4Range::CarrierGradeNat);
    }
    false
}

/// Applies the SSRF block policy for parsed IPv4 special-use ranges.
pub fn is_blocked_special_use_ipv4_address(
    address: &Ipv4Addr,
    options: &Ipv4SpecialUseBlockOptions,
) -> bool {
    let in_rfc2544 = address.match_prefix(rfc2544_benchmark_prefix(), 15);
    if in_rfc2544 && options.allow_rfc2544_benchmark_range == Some(true) {
        return false;
    }
    blocked_ipv4_special_use_ranges().contains(&address.range()) || in_rfc2544
}

fn rfc2544_benchmark_prefix() -> Ipv4Addr {
    Ipv4Addr::new(198, 18, 0, 0)
}

fn blocked_ipv4_special_use_ranges() -> &'static HashSet<Ipv4Range> {
    static SET: OnceLock<HashSet<Ipv4Range>> = OnceLock::new();
    SET.get_or_init(|| {
        let mut s = HashSet::new();
        s.insert(Ipv4Range::Unspecified);
        s.insert(Ipv4Range::Broadcast);
        s.insert(Ipv4Range::Multicast);
        s.insert(Ipv4Range::LinkLocal);
        s.insert(Ipv4Range::Loopback);
        s.insert(Ipv4Range::CarrierGradeNat);
        s.insert(Ipv4Range::Private);
        s.insert(Ipv4Range::Reserved);
        s
    })
}

fn private_or_loopback_ipv4_ranges() -> &'static HashSet<Ipv4Range> {
    static SET: OnceLock<HashSet<Ipv4Range>> = OnceLock::new();
    SET.get_or_init(|| {
        let mut s = HashSet::new();
        s.insert(Ipv4Range::Loopback);
        s.insert(Ipv4Range::Private);
        s.insert(Ipv4Range::LinkLocal);
        s.insert(Ipv4Range::CarrierGradeNat);
        s
    })
}

fn blocked_ipv6_special_use_ranges() -> &'static HashSet<Ipv6Range> {
    static SET: OnceLock<HashSet<Ipv6Range>> = OnceLock::new();
    SET.get_or_init(|| {
        let mut s = HashSet::new();
        s.insert(Ipv6Range::Unspecified);
        s.insert(Ipv6Range::Loopback);
        s.insert(Ipv6Range::LinkLocal);
        s.insert(Ipv6Range::UniqueLocal);
        s.insert(Ipv6Range::Multicast);
        s.insert(Ipv6Range::Reserved);
        s.insert(Ipv6Range::Benchmarking);
        s.insert(Ipv6Range::Discard);
        s.insert(Ipv6Range::Orchid2);
        s
    })
}

/// Per-call exemptions for `is_blocked_special_use_ipv4_address`.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct Ipv4SpecialUseBlockOptions {
    #[serde(rename = "allowRfc2544BenchmarkRange", skip_serializing_if = "Option::is_none")]
    pub allow_rfc2544_benchmark_range: Option<bool>,
}

/// Per-call exemptions for `is_blocked_special_use_ipv6_address`. Mirror of
/// `Ipv4SpecialUseBlockOptions` for the IPv6 side.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct Ipv6SpecialUseBlockOptions {
    /// When true, exempt addresses in `fc00::/7` from the SSRF block.
    #[serde(rename = "allowUniqueLocalRange", skip_serializing_if = "Option::is_none")]
    pub allow_unique_local_range: Option<bool>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RangeClass {
    Ipv4Loopback,
}

impl ParsedIpAddress {
    pub fn kind(&self) -> IpKind {
        match self {
            ParsedIpAddress::Ipv4(_) => IpKind::Ipv4,
            ParsedIpAddress::Ipv6(_) => IpKind::Ipv6,
        }
    }
    fn range_ip(&self) -> Option<RangeClass> {
        match self {
            ParsedIpAddress::Ipv4(v4) if matches!(v4.range(), Ipv4Range::Loopback) => {
                Some(RangeClass::Ipv4Loopback)
            }
            ParsedIpAddress::Ipv6(v6) if matches!(v6.range(), Ipv6Range::Loopback) => {
                Some(RangeClass::Ipv4Loopback)
            }
            _ => None,
        }
    }
}

impl std::fmt::Display for ParsedIpAddress {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParsedIpAddress::Ipv4(v4) => write!(f, "{}", v4),
            ParsedIpAddress::Ipv6(v6) => write!(f, "{}", v6),
        }
    }
}

fn decode_ipv4_from_hextets(high: u16, low: u16) -> Ipv4Addr {
    Ipv4Addr::new(
        ((high >> 8) & 0xff) as u8,
        (high & 0xff) as u8,
        ((low >> 8) & 0xff) as u8,
        (low & 0xff) as u8,
    )
}

/// Extracts embedded IPv4 addresses from mapped and transition IPv6 prefixes.
pub fn extract_embedded_ipv4_from_ipv6(address: &Ipv6Addr) -> Option<Ipv4Addr> {
    if address.is_ipv4_mapped_address() {
        return Some(address.to_ipv4_address());
    }
    let parts = address.parts();
    if matches!(address.range(), Ipv6Range::Rfc6145) {
        return Some(decode_ipv4_from_hextets(parts[6], parts[7]));
    }
    if matches!(address.range(), Ipv6Range::Rfc6052) {
        return Some(decode_ipv4_from_hextets(parts[6], parts[7]));
    }
    for rule in embedded_ipv4_sentinel_rules() {
        if !(rule.matches)(&parts) {
            continue;
        }
        let (high, low) = (rule.to_hextets)(&parts);
        return Some(decode_ipv4_from_hextets(high, low));
    }
    None
}

struct EmbeddedRule {
    matches: fn(&[u16; 8]) -> bool,
    to_hextets: fn(&[u16; 8]) -> (u16, u16),
}

fn embedded_ipv4_sentinel_rules() -> &'static [EmbeddedRule] {
    static RULES: OnceLock<Vec<EmbeddedRule>> = OnceLock::new();
    RULES.get_or_init(|| {
        vec![
            EmbeddedRule {
                // ::w.x.y.z
                matches: |p| {
                    p[0] == 0 && p[1] == 0 && p[2] == 0 && p[3] == 0 && p[4] == 0 && p[5] == 0
                },
                to_hextets: |p| (p[6], p[7]),
            },
            EmbeddedRule {
                // 64:ff9b:1::/48 NAT64
                matches: |p| {
                    p[0] == 0x0064 && p[1] == 0xff9b && p[2] == 0x0001 && p[3] == 0 && p[4] == 0 && p[5] == 0
                },
                to_hextets: |p| (p[6], p[7]),
            },
            EmbeddedRule {
                // 2002::/16 6to4
                matches: |p| p[0] == 0x2002,
                to_hextets: |p| (p[1], p[2]),
            },
            EmbeddedRule {
                // 2001:0000::/32 Teredo
                matches: |p| p[0] == 0x2001 && p[1] == 0x0000,
                to_hextets: |p| (p[6] ^ 0xffff, p[7] ^ 0xffff),
            },
            EmbeddedRule {
                // ISATAP ....:0000:5efe:w.x.y.z (u/g bits allowed in hextet 4)
                matches: |p| (p[4] & 0xfcff) == 0 && p[5] == 0x5efe,
                to_hextets: |p| (p[6], p[7]),
            },
        ]
    })
}

/// Checks an IP literal against an exact IP or CIDR range, normalizing mapped IPv4.
pub fn is_ip_in_cidr(ip: &str, cidr: &str) -> bool {
    let Some(normalized_ip) = parse_canonical_ip_address(Some(ip)) else {
        return false;
    };
    let candidate = cidr.trim();
    if candidate.is_empty() {
        return false;
    }
    let comparable_ip = normalize_ipv4_mapped_address(&normalized_ip);
    if !candidate.contains('/') {
        let Some(exact) = parse_canonical_ip_address(Some(candidate)) else {
            return false;
        };
        let comparable_exact = normalize_ipv4_mapped_address(&exact);
        return comparable_ip.kind() == comparable_exact.kind()
            && comparable_ip.to_string() == comparable_exact.to_string();
    }
    let parsed_cidr = match parse_cidr(candidate) {
        Some(v) => v,
        None => return false,
    };
    let (base_address, prefix_length) = parsed_cidr;
    let comparable_base = normalize_ipv4_mapped_address(&base_address);
    if comparable_ip.kind() != comparable_base.kind() {
        return false;
    }
    match (&comparable_ip, &comparable_base) {
        (ParsedIpAddress::Ipv4(ip4), ParsedIpAddress::Ipv4(base4)) => {
            ip4.match_prefix(*base4, prefix_length)
        }
        (ParsedIpAddress::Ipv6(ip6), ParsedIpAddress::Ipv6(base6)) => {
            ip6.match_prefix(*base6, prefix_length)
        }
        _ => false,
    }
}

fn parse_cidr(s: &str) -> Option<(ParsedIpAddress, u8)> {
    let (addr_part, prefix_part) = s.split_once('/')?;
    let prefix: u8 = prefix_part.parse().ok()?;
    let addr = if addr_part.contains(':') {
        let v6 = parse_ipv6_loose(addr_part)?;
        ParsedIpAddress::Ipv6(v6)
    } else {
        let v4 = parse_ipv4_strict(addr_part)?;
        ParsedIpAddress::Ipv4(v4)
    };
    Some((addr, prefix))
}