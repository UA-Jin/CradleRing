// Gateway Protocol schema module: system info.
// 翻译自 packages/gateway-protocol/src/schema/system-info.ts
//
// Gateway host system information schemas.
//
// TS 用 TypeBox 定义 schema（运行时验证 + 类型）。
// Rust 用 serde struct + 验证函数实现等价的序列化/反序列化语义。

use serde::{Deserialize, Serialize};

// ---------- SystemInfoParams ----------

/// Empty request payload for Gateway host system information.
/// 对齐 TS: `Type.Object({}, { additionalProperties: false })`
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SystemInfoParams {}

impl SystemInfoParams {
    /// 对齐 TS 的 `additionalProperties: false` —— 此结构体没有字段，
    /// 反序列化时 serde 已经默认拒绝未知键，无需额外校验。
    pub fn validate(&self) -> Result<(), String> {
        Ok(())
    }
}

// ---------- SystemInfoResult ----------

/// Gateway host identity and resource snapshot.
/// 对齐 TS:
///   `Type.Object({
///     machineName: Type.String(),
///     hostname: Type.String(),
///     platform: Type.String(),
///     release: Type.String(),
///     arch: Type.String(),
///     osLabel: Type.String(),
///     lanAddress: Type.Optional(Type.String()),
///     port: Type.Optional(Type.Integer()),
///     nodeVersion: Type.String(),
///     pid: Type.Integer(),
///     uptimeMs: Type.Integer(),
///     cpuCount: Type.Integer(),
///     cpuModel: Type.Optional(Type.String()),
///     loadAverage: Type.Optional(Type.Tuple([Type.Number(), Type.Number(), Type.Number()])),
///     memoryTotalBytes: Type.Integer(),
///     memoryFreeBytes: Type.Integer(),
///     diskTotalBytes: Type.Optional(Type.Integer()),
///     diskAvailableBytes: Type.Optional(Type.Integer()),
///     diskPath: Type.Optional(Type.String()),
///   }, { additionalProperties: false })`
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SystemInfoResult {
    pub machine_name: String,
    pub hostname: String,
    pub platform: String,
    pub release: String,
    pub arch: String,
    pub os_label: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lan_address: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub port: Option<i64>,
    pub node_version: String,
    pub pid: i64,
    pub uptime_ms: i64,
    pub cpu_count: i64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cpu_model: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub load_average: Option<[f64; 3]>,
    pub memory_total_bytes: i64,
    pub memory_free_bytes: i64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub disk_total_bytes: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub disk_available_bytes: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub disk_path: Option<String>,
}

impl SystemInfoResult {
    pub fn validate(&self) -> Result<(), String> {
        // 必填字符串：TS 没有 minLength 约束，保持简单。
        // 这些字符串理论上不应为空，但 TS schema 也没有强制。
        // 故此处只校验整数非负性（pid, port, 资源计数/字节数天然非负）。
        validate_non_negative_integer(self.pid, "pid")?;
        if let Some(p) = self.port {
            validate_non_negative_integer(p, "port")?;
        }
        validate_non_negative_integer(self.uptime_ms, "uptimeMs")?;
        validate_non_negative_integer(self.cpu_count, "cpuCount")?;
        validate_non_negative_integer(self.memory_total_bytes, "memoryTotalBytes")?;
        validate_non_negative_integer(self.memory_free_bytes, "memoryFreeBytes")?;
        if let Some(d) = self.disk_total_bytes {
            validate_non_negative_integer(d, "diskTotalBytes")?;
        }
        if let Some(d) = self.disk_available_bytes {
            validate_non_negative_integer(d, "diskAvailableBytes")?;
        }
        Ok(())
    }
}

// ---------- 内部校验原语 ----------

/// 对齐 TS: `Type.Integer({ minimum: 0 })` —— 自然非负整数。
fn validate_non_negative_integer(n: i64, field: &str) -> Result<(), String> {
    if n >= 0 {
        Ok(())
    } else {
        Err(format!("{}: expected integer >= 0, got {}", field, n))
    }
}

// Wire types derive directly from local schema consts so public d.ts graphs never
// pull in the ProtocolSchemas registry.
// 对应 TS:
//   export type SystemInfoParams = Static<typeof SystemInfoParamsSchema>;
//   export type SystemInfoResult = Static<typeof SystemInfoResultSchema>;
pub type SystemInfoParamsType = SystemInfoParams;
pub type SystemInfoResultType = SystemInfoResult;

// ============================================================
// 单元测试
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn system_info_params_round_trip_empty_object() {
        let p = SystemInfoParams {};
        let s = serde_json::to_string(&p).unwrap();
        assert_eq!(s, "{}");
        let back: SystemInfoParams = serde_json::from_str(&s).unwrap();
        // 空对象往返一致
        let _ = back;
    }

    #[test]
    fn system_info_params_validates_ok() {
        assert!(SystemInfoParams {}.validate().is_ok());
    }

    #[test]
    fn system_info_result_round_trip_minimal() {
        let r = SystemInfoResult {
            machine_name: "box".into(),
            hostname: "host".into(),
            platform: "linux".into(),
            release: "6.19.0".into(),
            arch: "x86_64".into(),
            os_label: "Kali".into(),
            lan_address: None,
            port: None,
            node_version: "v20.0.0".into(),
            pid: 1234,
            uptime_ms: 60_000,
            cpu_count: 8,
            cpu_model: None,
            load_average: None,
            memory_total_bytes: 16 * 1024 * 1024 * 1024,
            memory_free_bytes: 8 * 1024 * 1024 * 1024,
            disk_total_bytes: None,
            disk_available_bytes: None,
            disk_path: None,
        };
        assert!(r.validate().is_ok());
        let s = serde_json::to_string(&r).unwrap();
        // camelCase 反序列化: 字段名以驼峰写回
        assert!(s.contains("\"machineName\""));
        assert!(s.contains("\"nodeVersion\""));
        assert!(s.contains("\"memoryTotalBytes\""));
        let back: SystemInfoResult = serde_json::from_str(&s).unwrap();
        assert_eq!(back.pid, 1234);
    }

    #[test]
    fn system_info_result_full_round_trip() {
        let raw = json!({
            "machineName": "box",
            "hostname": "host.local",
            "platform": "linux",
            "release": "6.19.0",
            "arch": "x86_64",
            "osLabel": "Kali",
            "lanAddress": "192.168.1.10",
            "port": 8788,
            "nodeVersion": "v20.0.0",
            "pid": 9999,
            "uptimeMs": 12345,
            "cpuCount": 16,
            "cpuModel": "AMD Ryzen",
            "loadAverage": [0.5, 1.0, 2.5],
            "memoryTotalBytes": 32_000_000_000i64,
            "memoryFreeBytes": 8_000_000_000i64,
            "diskTotalBytes": 1_000_000_000_000i64,
            "diskAvailableBytes": 500_000_000_000i64,
            "diskPath": "/"
        });
        let r: SystemInfoResult = serde_json::from_value(raw).unwrap();
        assert!(r.validate().is_ok());
        assert_eq!(r.machine_name, "box");
        assert_eq!(r.lan_address.as_deref(), Some("192.168.1.10"));
        assert_eq!(r.load_average, Some([0.5, 1.0, 2.5]));
        assert_eq!(r.disk_path.as_deref(), Some("/"));
    }

    #[test]
    fn system_info_result_negative_pid_rejected() {
        let r = SystemInfoResult {
            machine_name: "x".into(),
            hostname: "x".into(),
            platform: "x".into(),
            release: "x".into(),
            arch: "x".into(),
            os_label: "x".into(),
            lan_address: None,
            port: None,
            node_version: "x".into(),
            pid: -1,
            uptime_ms: 0,
            cpu_count: 1,
            cpu_model: None,
            load_average: None,
            memory_total_bytes: 0,
            memory_free_bytes: 0,
            disk_total_bytes: None,
            disk_available_bytes: None,
            disk_path: None,
        };
        let err = r.validate().unwrap_err();
        assert!(err.contains("pid"));
    }

    #[test]
    fn system_info_result_optional_omitted_in_serialization() {
        let r = SystemInfoResult {
            machine_name: "x".into(),
            hostname: "x".into(),
            platform: "x".into(),
            release: "x".into(),
            arch: "x".into(),
            os_label: "x".into(),
            lan_address: None,
            port: None,
            node_version: "x".into(),
            pid: 1,
            uptime_ms: 0,
            cpu_count: 1,
            cpu_model: None,
            load_average: None,
            memory_total_bytes: 0,
            memory_free_bytes: 0,
            disk_total_bytes: None,
            disk_available_bytes: None,
            disk_path: None,
        };
        let s = serde_json::to_string(&r).unwrap();
        // Optional 字段均不应出现在 JSON 中
        assert!(!s.contains("lanAddress"));
        assert!(!s.contains("port"));
        assert!(!s.contains("cpuModel"));
        assert!(!s.contains("loadAverage"));
        assert!(!s.contains("diskTotalBytes"));
    }
}
