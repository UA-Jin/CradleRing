// Gateway Client module implements device auth behavior.
// 翻译自 packages/gateway-client/src/device-auth.ts

pub fn normalize_device_metadata_for_auth(value: Option<&str>) -> String {
    match value {
        None => String::new(),
        Some(v) => {
            let trimmed = v.trim();
            if trimmed.is_empty() {
                return String::new();
            }
            trimmed
                .chars()
                .map(|c| {
                    if c.is_ascii_uppercase() {
                        (c as u8 + 32) as char
                    } else {
                        c
                    }
                })
                .collect()
        }
    }
}

#[derive(Debug, Clone)]
pub struct DeviceAuthPayloadParams {
    pub device_id: String,
    pub client_id: String,
    pub client_mode: String,
    pub role: String,
    pub scopes: Vec<String>,
    pub signed_at_ms: i64,
    pub token: Option<String>,
    pub nonce: String,
}

#[derive(Debug, Clone)]
pub struct DeviceAuthPayloadV3Params {
    pub device_id: String,
    pub client_id: String,
    pub client_mode: String,
    pub role: String,
    pub scopes: Vec<String>,
    pub signed_at_ms: i64,
    pub token: Option<String>,
    pub nonce: String,
    pub platform: Option<String>,
    pub device_family: Option<String>,
}

pub fn build_device_auth_payload(params: DeviceAuthPayloadParams) -> String {
    let scopes = params.scopes.join(",");
    let token = params.token.unwrap_or_default();
    [
        "v2",
        &params.device_id,
        &params.client_id,
        &params.client_mode,
        &params.role,
        &scopes,
        &params.signed_at_ms.to_string(),
        &token,
        &params.nonce,
    ]
    .join("|")
}

pub fn build_device_auth_payload_v3(params: DeviceAuthPayloadV3Params) -> String {
    let scopes = params.scopes.join(",");
    let token = params.token.unwrap_or_default();
    let platform = normalize_device_metadata_for_auth(params.platform.as_deref());
    let device_family = normalize_device_metadata_for_auth(params.device_family.as_deref());
    [
        "v3",
        &params.device_id,
        &params.client_id,
        &params.client_mode,
        &params.role,
        &scopes,
        &params.signed_at_ms.to_string(),
        &token,
        &params.nonce,
        &platform,
        &device_family,
    ]
    .join("|")
}