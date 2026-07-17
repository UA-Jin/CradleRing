// Gateway Protocol schema: artifacts.
// 翻译自 packages/gateway-protocol/src/schema/artifacts.ts
//
// TS 用 TypeBox 定义 schema（运行时验证 + 类型）。
// Rust 用 serde struct + 验证函数实现等价的序列化/反序列化语义。
//
// Artifact lookup and download protocol schemas.
//
// Artifacts are files or payloads produced by sessions, runs, tasks, or agents;
// these schemas keep lookup filters explicit and download results transport-safe.

use serde::{Deserialize, Serialize};

// ---------- 基础验证原语 (对齐 TypeBox: NonEmptyString) ----------

/// 对齐 TS: `NonEmptyString = Type.String({ minLength: 1 })`
fn is_non_empty_string(s: &str) -> bool {
    !s.is_empty()
}

fn validate_non_empty_string(s: &str) -> Result<(), String> {
    if is_non_empty_string(s) {
        Ok(())
    } else {
        Err(format!("expected non-empty string, got {:?}", s))
    }
}

fn validate_optional_non_empty_string(value: Option<&str>) -> Result<(), String> {
    if let Some(s) = value {
        validate_non_empty_string(s)?;
    }
    Ok(())
}

/// 对齐 TS: `Type.Integer({ minimum: 0 })` / `Type.Integer({ minimum: 1 })`
fn validate_non_negative_integer(n: i64) -> Result<(), String> {
    if n < 0 {
        Err(format!("expected non-negative integer, got {}", n))
    } else {
        Ok(())
    }
}

fn validate_positive_integer(n: i64) -> Result<(), String> {
    if n < 1 {
        Err(format!("expected positive integer (>= 1), got {}", n))
    } else {
        Ok(())
    }
}

fn validate_optional_non_negative_integer(value: Option<i64>) -> Result<(), String> {
    if let Some(n) = value {
        validate_non_negative_integer(n)?;
    }
    Ok(())
}

fn validate_optional_positive_integer(value: Option<i64>) -> Result<(), String> {
    if let Some(n) = value {
        validate_positive_integer(n)?;
    }
    Ok(())
}

// ---------- ArtifactQueryParams ----------

/// Shared artifact filter payload used by list-style requests.
/// 对齐 TS:
/// ```ts
/// const ArtifactQueryParamsProperties = {
///   sessionKey: Type.Optional(NonEmptyString),
///   runId: Type.Optional(NonEmptyString),
///   taskId: Type.Optional(NonEmptyString),
///   agentId: Type.Optional(NonEmptyString),
/// };
///
/// export const ArtifactQueryParamsSchema = Type.Object(ArtifactQueryParamsProperties, {
///   additionalProperties: false,
/// });
/// ```
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArtifactQueryParams {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub session_key: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub run_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub task_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub agent_id: Option<String>,
}

impl ArtifactQueryParams {
    pub fn validate(&self) -> Result<(), String> {
        validate_optional_non_empty_string(self.session_key.as_deref())?;
        validate_optional_non_empty_string(self.run_id.as_deref())?;
        validate_optional_non_empty_string(self.task_id.as_deref())?;
        validate_optional_non_empty_string(self.agent_id.as_deref())?;
        Ok(())
    }
}

// ---------- ArtifactGetParams ----------

/// Artifact lookup payload with a required artifact id plus optional scope filters.
/// 对齐 TS:
/// ```ts
/// export const ArtifactGetParamsSchema = Type.Object(
///   {
///     ...ArtifactQueryParamsProperties,
///     artifactId: NonEmptyString,
///   },
///   { additionalProperties: false },
/// );
/// ```
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArtifactGetParams {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub session_key: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub run_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub task_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub agent_id: Option<String>,
    pub artifact_id: String,
}

impl ArtifactGetParams {
    pub fn validate(&self) -> Result<(), String> {
        validate_optional_non_empty_string(self.session_key.as_deref())?;
        validate_optional_non_empty_string(self.run_id.as_deref())?;
        validate_optional_non_empty_string(self.task_id.as_deref())?;
        validate_optional_non_empty_string(self.agent_id.as_deref())?;
        validate_non_empty_string(&self.artifact_id)?;
        Ok(())
    }
}

// ---------- ArtifactDownloadMode (union of literals) ----------

/// Artifact transport mode advertised alongside artifact metadata.
/// 对齐 TS: `Type.Union([Type.Literal("bytes"), Type.Literal("url"),
///                       Type.Literal("unsupported")])`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ArtifactDownloadMode {
    #[serde(rename = "bytes")]
    Bytes,
    #[serde(rename = "url")]
    Url,
    #[serde(rename = "unsupported")]
    Unsupported,
}

impl ArtifactDownloadMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Bytes => "bytes",
            Self::Url => "url",
            Self::Unsupported => "unsupported",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "bytes" => Some(Self::Bytes),
            "url" => Some(Self::Url),
            "unsupported" => Some(Self::Unsupported),
            _ => None,
        }
    }

    pub fn all() -> &'static [ArtifactDownloadMode] {
        &[Self::Bytes, Self::Url, Self::Unsupported]
    }
}

// ---------- ArtifactSummary ----------

/// Public artifact metadata returned before or alongside download data.
/// 对齐 TS:
/// ```ts
/// export const ArtifactSummarySchema = Type.Object(
///   {
///     id: NonEmptyString,
///     type: NonEmptyString,
///     title: NonEmptyString,
///     mimeType: Type.Optional(NonEmptyString),
///     sizeBytes: Type.Optional(Type.Integer({ minimum: 0 })),
///     sessionKey: Type.Optional(NonEmptyString),
///     runId: Type.Optional(NonEmptyString),
///     taskId: Type.Optional(NonEmptyString),
///     messageSeq: Type.Optional(Type.Integer({ minimum: 1 })),
///     source: Type.Optional(NonEmptyString),
///     download: Type.Object(
///       {
///         mode: Type.Union([Type.Literal("bytes"), Type.Literal("url"), Type.Literal("unsupported")]),
///       },
///       { additionalProperties: false },
///     ),
///   },
///   { additionalProperties: false },
/// );
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArtifactSummary {
    pub id: String,
    #[serde(rename = "type")]
    pub artifact_type: String,
    pub title: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mime_type: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub size_bytes: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub session_key: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub run_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub task_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message_seq: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    pub download: ArtifactDownload,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArtifactDownload {
    pub mode: ArtifactDownloadMode,
}

impl ArtifactSummary {
    pub fn validate(&self) -> Result<(), String> {
        validate_non_empty_string(&self.id)?;
        validate_non_empty_string(&self.artifact_type)?;
        validate_non_empty_string(&self.title)?;
        validate_optional_non_empty_string(self.mime_type.as_deref())?;
        validate_optional_non_negative_integer(self.size_bytes)?;
        validate_optional_non_empty_string(self.session_key.as_deref())?;
        validate_optional_non_empty_string(self.run_id.as_deref())?;
        validate_optional_non_empty_string(self.task_id.as_deref())?;
        validate_optional_positive_integer(self.message_seq)?;
        validate_optional_non_empty_string(self.source.as_deref())?;
        // `download.mode` is a closed enum; deserialization rejects unknown values,
        // but mirror the constraint here for in-memory validation.
        if ArtifactDownloadMode::from_str(self.download.mode.as_str()).is_none() {
            return Err(format!("invalid artifact download mode: {}", self.download.mode.as_str()));
        }
        Ok(())
    }
}

// ---------- ArtifactsList ----------

/// List request payload for artifacts visible in the selected scope.
/// 对齐 TS: `export const ArtifactsListParamsSchema = ArtifactQueryParamsSchema;`
pub type ArtifactsListParams = ArtifactQueryParams;

/// List response containing artifact summaries only.
/// 对齐 TS:
/// ```ts
/// export const ArtifactsListResultSchema = Type.Object(
///   {
///     artifacts: Type.Array(ArtifactSummarySchema),
///   },
///   { additionalProperties: false },
/// );
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArtifactsListResult {
    pub artifacts: Vec<ArtifactSummary>,
}

impl ArtifactsListResult {
    pub fn validate(&self) -> Result<(), String> {
        for (i, a) in self.artifacts.iter().enumerate() {
            a.validate().map_err(|e| format!("artifacts[{}]: {}", i, e))?;
        }
        Ok(())
    }
}

// ---------- ArtifactsGet ----------

/// Get request payload for one artifact summary.
/// 对齐 TS: `export const ArtifactsGetParamsSchema = ArtifactGetParamsSchema;`
pub type ArtifactsGetParams = ArtifactGetParams;

/// Get response containing one artifact summary.
/// 对齐 TS:
/// ```ts
/// export const ArtifactsGetResultSchema = Type.Object(
///   {
///     artifact: ArtifactSummarySchema,
///   },
///   { additionalProperties: false },
/// );
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArtifactsGetResult {
    pub artifact: ArtifactSummary,
}

impl ArtifactsGetResult {
    pub fn validate(&self) -> Result<(), String> {
        self.artifact.validate()
    }
}

// ---------- ArtifactsDownload ----------

/// Download request payload for one artifact.
/// 对齐 TS: `export const ArtifactsDownloadParamsSchema = ArtifactGetParamsSchema;`
pub type ArtifactsDownloadParams = ArtifactGetParams;

/// Download response, either inline base64 bytes, URL, or metadata for unsupported modes.
/// 对齐 TS:
/// ```ts
/// export const ArtifactsDownloadResultSchema = Type.Object(
///   {
///     artifact: ArtifactSummarySchema,
///     encoding: Type.Optional(Type.Literal("base64")),
///     data: Type.Optional(Type.String()),
///     url: Type.Optional(NonEmptyString),
///   },
///   { additionalProperties: false },
/// );
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArtifactsDownloadResult {
    pub artifact: ArtifactSummary,
    /// When present, the only accepted literal value is `"base64"`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub encoding: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub data: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
}

impl ArtifactsDownloadResult {
    pub fn validate(&self) -> Result<(), String> {
        self.artifact.validate()?;
        if let Some(enc) = self.encoding.as_deref() {
            if enc != "base64" {
                return Err(format!("invalid download encoding: {}", enc));
            }
        }
        if let Some(d) = self.data.as_deref() {
            validate_non_empty_string(d).map_err(|_| "download data must be non-empty".to_string())?;
        }
        validate_optional_non_empty_string(self.url.as_deref())?;
        Ok(())
    }
}

// Wire types derive directly from local schema consts so public d.ts graphs never
// pull in the ProtocolSchemas registry.
// 对应 TS:
//   export type ArtifactSummary = Static<typeof ArtifactSummarySchema>;
//   export type ArtifactsListParams = Static<typeof ArtifactsListParamsSchema>;
//   export type ArtifactsListResult = Static<typeof ArtifactsListResultSchema>;
//   export type ArtifactsGetParams = Static<typeof ArtifactsGetParamsSchema>;
//   export type ArtifactsGetResult = Static<typeof ArtifactsGetResultSchema>;
//   export type ArtifactsDownloadParams = Static<typeof ArtifactsDownloadParamsSchema>;
//   export type ArtifactsDownloadResult = Static<typeof ArtifactsDownloadResultSchema>;
pub type ArtifactSummaryType = ArtifactSummary;
pub type ArtifactsListParamsType = ArtifactsListParams;
pub type ArtifactsListResultType = ArtifactsListResult;
pub type ArtifactsGetParamsType = ArtifactsGetParams;
pub type ArtifactsGetResultType = ArtifactsGetResult;
pub type ArtifactsDownloadParamsType = ArtifactsDownloadParams;
pub type ArtifactsDownloadResultType = ArtifactsDownloadResult;