// Gateway Protocol schema: terminal.
// 翻译自 packages/gateway-protocol/src/schema/terminal.ts
//
// Gateway Protocol schema module for the operator terminal surface.
// Terminal methods open a PTY-backed shell session bound to one authenticated
// operator connection and stream its bytes back over the existing WebSocket.
//
// TS 用 TypeBox 定义 schema（运行时验证 + 类型）。
// Rust 用 serde struct + 验证函数实现等价的序列化/反序列化语义。

use serde::{Deserialize, Serialize};

use super::primitives::NonEmptyString;

// ---------- Module-private bounds ----------

/// PTY grid dimension bounds: rows/cols must fall inside this closed range so a
/// hostile client cannot request an allocation that overflows the terminal
/// backend's row/column math.
/// 对齐 TS: `TerminalDimension = Type.Integer({ minimum: 1, maximum: 2000 })`.
pub const TERMINAL_DIMENSION_MIN: i64 = 1;
pub const TERMINAL_DIMENSION_MAX: i64 = 2000;

// ---------- 基础验证原语 (对齐 TypeBox: NonEmptyString / Integer{min,max}) ----------

/// 对齐 TS: `NonEmptyString = Type.String({ minLength: 1 })`
fn is_non_empty_string(s: &str) -> bool {
    !s.is_empty()
}

fn validate_non_empty_string(field: &str, value: &str) -> Result<(), String> {
    if is_non_empty_string(value) {
        Ok(())
    } else {
        Err(format!(
            "{}: expected non-empty string, got {:?}",
            field, value
        ))
    }
}

fn validate_optional_non_empty_string(field: &str, value: Option<&str>) -> Result<(), String> {
    if let Some(s) = value {
        validate_non_empty_string(field, s)?;
    }
    Ok(())
}

/// 对齐 TS: `Type.Integer({ minimum: 0 })`
fn validate_non_negative_integer(field: &str, n: i64) -> Result<(), String> {
    if n >= 0 {
        Ok(())
    } else {
        Err(format!("{}: expected integer >= 0, got {}", field, n))
    }
}

/// 对齐 TS: `Type.Integer({ minimum: 1, maximum: 2000 })` (TerminalDimension).
fn validate_terminal_dimension(field: &str, n: i64) -> Result<(), String> {
    if n < TERMINAL_DIMENSION_MIN || n > TERMINAL_DIMENSION_MAX {
        return Err(format!(
            "{}: expected integer in [{}, {}], got {}",
            field, TERMINAL_DIMENSION_MIN, TERMINAL_DIMENSION_MAX, n
        ));
    }
    Ok(())
}

// ---------- TerminalOpenParamsSchema ----------

/// Opens a shell session; the server picks the shell, cwd, and confinement.
/// 对齐 TS:
///   `Type.Object({
///      agentId: Type.Optional(NonEmptyString),
///      cols:    TerminalDimension,
///      rows:    TerminalDimension,
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TerminalOpenParams {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub agent_id: Option<NonEmptyString>,
    pub cols: i64,
    pub rows: i64,
}

impl TerminalOpenParams {
    pub fn validate(&self) -> Result<(), String> {
        validate_optional_non_empty_string("agentId", self.agent_id.as_deref())?;
        validate_terminal_dimension("cols", self.cols)?;
        validate_terminal_dimension("rows", self.rows)?;
        Ok(())
    }
}

// ---------- TerminalOpenResultSchema ----------

/// Result of a successful open; carries the facts the UI header renders.
/// 对齐 TS:
///   `Type.Object({
///      sessionId: NonEmptyString,
///      agentId:   NonEmptyString,
///      shell:     NonEmptyString,
///      cwd:       NonEmptyString,
///      confined:  Type.Boolean(),
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TerminalOpenResult {
    pub session_id: NonEmptyString,
    pub agent_id: NonEmptyString,
    pub shell: NonEmptyString,
    pub cwd: NonEmptyString,
    /// True when the shell runs inside the agent's sandbox and cannot escape the
    /// workspace; false for a host shell that can navigate the whole filesystem.
    pub confined: bool,
}

impl TerminalOpenResult {
    pub fn validate(&self) -> Result<(), String> {
        validate_non_empty_string("sessionId", &self.session_id)?;
        validate_non_empty_string("agentId", &self.agent_id)?;
        validate_non_empty_string("shell", &self.shell)?;
        validate_non_empty_string("cwd", &self.cwd)?;
        Ok(())
    }
}

// ---------- TerminalInputParamsSchema ----------

/// Writes client keystrokes to the session stdin.
/// 对齐 TS:
///   `Type.Object({
///      sessionId: NonEmptyString,
///      // Raw terminal input (already-encoded escape sequences from the emulator).
///      data:      Type.String(),
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TerminalInputParams {
    pub session_id: NonEmptyString,
    /// Raw terminal input (already-encoded escape sequences from the emulator).
    pub data: String,
}

impl TerminalInputParams {
    pub fn validate(&self) -> Result<(), String> {
        validate_non_empty_string("sessionId", &self.session_id)?;
        Ok(())
    }
}

// ---------- TerminalResizeParamsSchema ----------

/// Resizes the PTY grid after the client viewport changes.
/// 对齐 TS:
///   `Type.Object({
///      sessionId: NonEmptyString,
///      cols:       TerminalDimension,
///      rows:       TerminalDimension,
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TerminalResizeParams {
    pub session_id: NonEmptyString,
    pub cols: i64,
    pub rows: i64,
}

impl TerminalResizeParams {
    pub fn validate(&self) -> Result<(), String> {
        validate_non_empty_string("sessionId", &self.session_id)?;
        validate_terminal_dimension("cols", self.cols)?;
        validate_terminal_dimension("rows", self.rows)?;
        Ok(())
    }
}

// ---------- TerminalCloseParamsSchema ----------

/// Closes a session and kills its process tree.
/// 对齐 TS: `Type.Object({ sessionId: NonEmptyString }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TerminalCloseParams {
    pub session_id: NonEmptyString,
}

impl TerminalCloseParams {
    pub fn validate(&self) -> Result<(), String> {
        validate_non_empty_string("sessionId", &self.session_id)?;
        Ok(())
    }
}

// ---------- TerminalAttachParamsSchema ----------

/**
 * Rebinds a live-or-detached session to the calling admin connection.
 * Attach is take-over (tmux-like): the previous owner, if still connected,
 * receives `terminal.exit` with reason "detached".
 * 对齐 TS: `Type.Object({ sessionId: NonEmptyString }, { additionalProperties: false })`.
 */
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TerminalAttachParams {
    pub session_id: NonEmptyString,
}

impl TerminalAttachParams {
    pub fn validate(&self) -> Result<(), String> {
        validate_non_empty_string("sessionId", &self.session_id)?;
        Ok(())
    }
}

// ---------- TerminalAttachResultSchema ----------

/// Result of a successful attach; mirrors open plus the replay buffer.
/// 对齐 TS:
///   `Type.Object({
///      sessionId: NonEmptyString,
///      agentId:   NonEmptyString,
///      shell:     NonEmptyString,
///      cwd:       NonEmptyString,
///      confined:  Type.Boolean(),
///      // Recent raw output from the server's bounded ring buffer, replayed into
///      // the client emulator before live terminal.data resumes.
///      buffer:    Type.String(),
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TerminalAttachResult {
    pub session_id: NonEmptyString,
    pub agent_id: NonEmptyString,
    pub shell: NonEmptyString,
    pub cwd: NonEmptyString,
    pub confined: bool,
    /// Recent raw output from the server's bounded ring buffer, replayed into
    /// the client emulator before live terminal.data resumes. Not a true screen
    /// snapshot: after truncation it can start mid-escape-sequence; emulators
    /// recover on the next full repaint (prompt, clear, resize redraw).
    pub buffer: String,
}

impl TerminalAttachResult {
    pub fn validate(&self) -> Result<(), String> {
        validate_non_empty_string("sessionId", &self.session_id)?;
        validate_non_empty_string("agentId", &self.agent_id)?;
        validate_non_empty_string("shell", &self.shell)?;
        validate_non_empty_string("cwd", &self.cwd)?;
        Ok(())
    }
}

// ---------- TerminalSessionInfoSchema ----------

/// One attachable session, as reported by terminal.list.
/// 对齐 TS:
///   `Type.Object({
///      sessionId:    NonEmptyString,
///      agentId:      NonEmptyString,
///      shell:        NonEmptyString,
///      cwd:          NonEmptyString,
///      confined:     Type.Boolean(),
///      attached:     Type.Boolean(),
///      createdAtMs:  Type.Integer({ minimum: 0 }),
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TerminalSessionInfo {
    pub session_id: NonEmptyString,
    pub agent_id: NonEmptyString,
    pub shell: NonEmptyString,
    pub cwd: NonEmptyString,
    pub confined: bool,
    /// False while the session is detached (no connection owns its stream).
    pub attached: bool,
    pub created_at_ms: i64,
}

impl TerminalSessionInfo {
    pub fn validate(&self) -> Result<(), String> {
        validate_non_empty_string("sessionId", &self.session_id)?;
        validate_non_empty_string("agentId", &self.agent_id)?;
        validate_non_empty_string("shell", &self.shell)?;
        validate_non_empty_string("cwd", &self.cwd)?;
        validate_non_negative_integer("createdAtMs", self.created_at_ms)?;
        Ok(())
    }
}

// ---------- TerminalListResultSchema ----------

/**
 * Sessions a reconnecting admin client can attach. All admin connections see
 * the same list: the terminal surface is already operator.admin (full host
 * access), so cross-connection visibility adds no privilege.
 * 对齐 TS: `Type.Object({ sessions: Type.Array(TerminalSessionInfoSchema) },
 *                       { additionalProperties: false })`.
 */
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TerminalListResult {
    pub sessions: Vec<TerminalSessionInfo>,
}

impl TerminalListResult {
    pub fn validate(&self) -> Result<(), String> {
        for (i, s) in self.sessions.iter().enumerate() {
            s.validate().map_err(|e| format!("sessions[{}]: {}", i, e))?;
        }
        Ok(())
    }
}

// ---------- TerminalTextParamsSchema ----------

/// Reads the current output buffer as plain text without attaching.
/// 对齐 TS: `Type.Object({ sessionId: NonEmptyString }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TerminalTextParams {
    pub session_id: NonEmptyString,
}

impl TerminalTextParams {
    pub fn validate(&self) -> Result<(), String> {
        validate_non_empty_string("sessionId", &self.session_id)?;
        Ok(())
    }
}

// ---------- TerminalTextResultSchema ----------

/// Plain-text buffer contents (ANSI stripped); an agent/LLM affordance.
/// 对齐 TS: `Type.Object({ text: Type.String() }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TerminalTextResult {
    pub text: String,
}

// ---------- TerminalAckResultSchema ----------

/// Shared ok/void result for input, resize, and close.
/// 对齐 TS: `Type.Object({ ok: Type.Boolean() }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TerminalAckResult {
    pub ok: bool,
}

// ---------- TerminalDataEventSchema ----------

/// Streamed output chunk; seq lets the client detect gaps and preserve order.
/// 对齐 TS:
///   `Type.Object({
///      sessionId: NonEmptyString,
///      seq:       Type.Integer({ minimum: 0 }),
///      data:      Type.String(),
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TerminalDataEvent {
    pub session_id: NonEmptyString,
    pub seq: i64,
    pub data: String,
}

impl TerminalDataEvent {
    pub fn validate(&self) -> Result<(), String> {
        validate_non_empty_string("sessionId", &self.session_id)?;
        validate_non_negative_integer("seq", self.seq)?;
        Ok(())
    }
}

// ---------- TerminalExitReason (closed enum) ----------

/// Stable reason codes so clients can distinguish process exit from a
/// server-side teardown (disconnect, idle sweep, config disable).
/// 对齐 TS:
///   `Type.Union([
///      Type.Literal("process_exit"),
///      Type.Literal("closed"),
///      Type.Literal("disconnected"),
///      Type.Literal("detached"),
///      Type.Literal("error"),
///   ])`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TerminalExitReason {
    #[serde(rename = "process_exit")]
    ProcessExit,
    #[serde(rename = "closed")]
    Closed,
    #[serde(rename = "disconnected")]
    Disconnected,
    /// Another admin connection attached the session away; the session is
    /// still alive server-side, but no longer streams to this connection.
    #[serde(rename = "detached")]
    Detached,
    #[serde(rename = "error")]
    Error,
}

impl TerminalExitReason {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::ProcessExit => "process_exit",
            Self::Closed => "closed",
            Self::Disconnected => "disconnected",
            Self::Detached => "detached",
            Self::Error => "error",
        }
    }
}

pub fn is_valid_terminal_exit_reason(s: &str) -> bool {
    matches!(
        s,
        "process_exit" | "closed" | "disconnected" | "detached" | "error"
    )
}

// ---------- TerminalExitEventSchema ----------

/// Terminal end-of-life notice; the session id is invalid after this event.
/// 对齐 TS:
///   `Type.Object({
///      sessionId: NonEmptyString,
///      exitCode:  Type.Optional(Type.Union([Type.Integer(), Type.Null()])),
///      signal:    Type.Optional(Type.Union([Type.Integer(), Type.Null()])),
///      reason:    Type.Optional(Type.Union([...])),
///      error:     Type.Optional(Type.String()),
///   }, { additionalProperties: false })`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TerminalExitEvent {
    pub session_id: NonEmptyString,
    /// Process exit code (null when the session ended via signal/disconnect).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub exit_code: Option<i64>,
    /// Signal that terminated the process (null for normal exit).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub signal: Option<i64>,
    /// Stable reason code so clients can distinguish process exit from a
    /// server-side teardown (disconnect, idle sweep, config disable).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<TerminalExitReason>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl TerminalExitEvent {
    pub fn validate(&self) -> Result<(), String> {
        validate_non_empty_string("sessionId", &self.session_id)?;
        Ok(())
    }
}

// ---------- TerminalEventSchema ----------

/// Union of every event a terminal session can emit.
/// 对齐 TS: `Type.Union([TerminalDataEventSchema, TerminalExitEventSchema])`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum TerminalEvent {
    Data(TerminalDataEvent),
    Exit(TerminalExitEvent),
}

impl TerminalEvent {
    pub fn validate(&self) -> Result<(), String> {
        match self {
            TerminalEvent::Data(e) => e.validate().map_err(|e| format!("data: {}", e)),
            TerminalEvent::Exit(e) => e.validate().map_err(|e| format!("exit: {}", e)),
        }
    }
}

// Wire types derive directly from local schema consts so public d.ts graphs never
// pull in the ProtocolSchemas registry.
// 对应 TS:
//   export type TerminalOpenParams      = Static<typeof TerminalOpenParamsSchema>;
//   export type TerminalOpenResult      = Static<typeof TerminalOpenResultSchema>;
//   export type TerminalInputParams     = Static<typeof TerminalInputParamsSchema>;
//   export type TerminalResizeParams    = Static<typeof TerminalResizeParamsSchema>;
//   export type TerminalCloseParams     = Static<typeof TerminalCloseParamsSchema>;
//   export type TerminalAttachParams    = Static<typeof TerminalAttachParamsSchema>;
//   export type TerminalAttachResult    = Static<typeof TerminalAttachResultSchema>;
//   export type TerminalSessionInfo     = Static<typeof TerminalSessionInfoSchema>;
//   export type TerminalListResult      = Static<typeof TerminalListResultSchema>;
//   export type TerminalTextParams      = Static<typeof TerminalTextParamsSchema>;
//   export type TerminalTextResult      = Static<typeof TerminalTextResultSchema>;
//   export type TerminalAckResult       = Static<typeof TerminalAckResultSchema>;
//   export type TerminalDataEvent       = Static<typeof TerminalDataEventSchema>;
//   export type TerminalExitEvent       = Static<typeof TerminalExitEventSchema>;
//   export type TerminalEvent           = Static<typeof TerminalEventSchema>;
pub type TerminalOpenParamsType = TerminalOpenParams;
pub type TerminalOpenResultType = TerminalOpenResult;
pub type TerminalInputParamsType = TerminalInputParams;
pub type TerminalResizeParamsType = TerminalResizeParams;
pub type TerminalCloseParamsType = TerminalCloseParams;
pub type TerminalAttachParamsType = TerminalAttachParams;
pub type TerminalAttachResultType = TerminalAttachResult;
pub type TerminalSessionInfoType = TerminalSessionInfo;
pub type TerminalListResultType = TerminalListResult;
pub type TerminalTextParamsType = TerminalTextParams;
pub type TerminalTextResultType = TerminalTextResult;
pub type TerminalAckResultType = TerminalAckResult;
pub type TerminalDataEventType = TerminalDataEvent;
pub type TerminalExitEventType = TerminalExitEvent;
pub type TerminalEventType = TerminalEvent;