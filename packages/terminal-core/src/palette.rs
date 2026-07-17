// Lobster palette tokens for CLI/UI theming. Use this palette for all CLI color output.
// Keep in sync with docs/cli/index.md (CLI palette section).
// 翻译自 packages/terminal-core/src/palette.ts

pub const LOBSTER_PALETTE: LobsterPalette = LobsterPalette {
    accent: "#FF5A2D",
    accent_bright: "#FF7A3D",
    accent_dim: "#D14A22",
    info: "#FF8A5B",
    success: "#2FBF71",
    warn: "#FFB020",
    error: "#E23D2D",
    muted: "#8B7F77",
};

#[derive(Debug, Clone, Copy)]
pub struct LobsterPalette {
    pub accent: &'static str,
    pub accent_bright: &'static str,
    pub accent_dim: &'static str,
    pub info: &'static str,
    pub success: &'static str,
    pub warn: &'static str,
    pub error: &'static str,
    pub muted: &'static str,
}
