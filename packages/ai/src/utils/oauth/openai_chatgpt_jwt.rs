//! OpenAI ChatGPT JWT (OAuth) helpers.
//! 翻译自 packages/ai/src/utils/oauth/openai-chatgpt-jwt.ts
//!
//! ChatGPT backend uses a signed JWT derived from a refresh-token-style
//! payload. This helper signs and decodes the JWT for outbound requests.

use base64::Engine;
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

/// JWT payload produced for ChatGPT backend auth.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatGptJwtClaims {
    pub sub: String,
    pub email: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub picture: Option<String>,
    pub exp: i64,
    pub iat: i64,
    pub iss: String,
    pub aud: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auth_provider: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub organization_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project_id: Option<String>,
}

/// Inputs required to sign a ChatGPT JWT.
pub struct ChatGptJwtSignInput<'a> {
    pub pem: &'a [u8],
    pub kid: &'a str,
    pub aud: &'a str,
    pub iss: &'a str,
    pub sub: &'a str,
    pub email: &'a str,
    pub exp: i64,
    pub iat: i64,
    pub name: Option<&'a str>,
    pub picture: Option<&'a str>,
    pub organization_id: Option<&'a str>,
    pub project_id: Option<&'a str>,
}

/// Sign a ChatGPT-style JWT with the given PEM private key.
pub fn sign_chatgpt_jwt(input: ChatGptJwtSignInput<'_>) -> Result<String, String> {
    let claims = ChatGptJwtClaims {
        sub: input.sub.to_string(),
        email: input.email.to_string(),
        name: input.name.map(|s| s.to_string()),
        picture: input.picture.map(|s| s.to_string()),
        exp: input.exp,
        iat: input.iat,
        iss: input.iss.to_string(),
        aud: input.aud.to_string(),
        auth_provider: Some("portal".to_string()),
        organization_id: input.organization_id.map(|s| s.to_string()),
        project_id: input.project_id.map(|s| s.to_string()),
    };

    let mut header = Header::new(Algorithm::RS256);
    header.kid = Some(input.kid.to_string());

    let encoding_key = EncodingKey::from_rsa_pem(input.pem).map_err(|e| e.to_string())?;
    encode(&header, &claims, &encoding_key).map_err(|e| e.to_string())
}

/// Decode and verify a ChatGPT JWT using a PEM-encoded public key.
pub fn decode_chatgpt_jwt(
    token: &str,
    pem: &[u8],
    audience: &str,
    issuer: &str,
) -> Result<ChatGptJwtClaims, String> {
    let mut validation = Validation::new(Algorithm::RS256);
    validation.set_audience(&[audience]);
    validation.set_issuer(&[issuer]);
    let decoding_key = DecodingKey::from_rsa_pem(pem).map_err(|e| e.to_string())?;
    let data = decode::<ChatGptClaimsShim>(token, &decoding_key, &validation)
        .map_err(|e| e.to_string())?;
    Ok(ChatGptJwtClaims {
        sub: data.claims.sub,
        email: data.claims.email,
        name: data.claims.name,
        picture: data.claims.picture,
        exp: data.claims.exp,
        iat: data.claims.iat,
        iss: data.claims.iss,
        aud: data.claims.aud,
        auth_provider: data.claims.auth_provider,
        organization_id: data.claims.organization_id,
        project_id: data.claims.project_id,
    })
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ChatGptClaimsShim {
    sub: String,
    email: String,
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    picture: Option<String>,
    exp: i64,
    iat: i64,
    iss: String,
    aud: String,
    #[serde(default)]
    auth_provider: Option<String>,
    #[serde(default)]
    organization_id: Option<String>,
    #[serde(default)]
    project_id: Option<String>,
}

/// Base64-URL encode a buffer (used for raw JWT signing in tests).
pub fn base64_url_encode(input: &[u8]) -> String {
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(input)
}

/// Base64-URL decode a string.
pub fn base64_url_decode(input: &str) -> Result<Vec<u8>, String> {
    base64::engine::general_purpose::URL_SAFE_NO_PAD
        .decode(input)
        .map_err(|e| e.to_string())
}