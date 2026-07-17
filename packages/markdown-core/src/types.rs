// Markdown Core module implements types behavior.
// 翻译自 packages/markdown-core/src/types.ts

/// Table rendering modes used when markdown tables need plaintext-safe output.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MarkdownTableMode {
    Off,
    Bullets,
    Code,
    Block,
}

impl MarkdownTableMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            MarkdownTableMode::Off => "off",
            MarkdownTableMode::Bullets => "bullets",
            MarkdownTableMode::Code => "code",
            MarkdownTableMode::Block => "block",
        }
    }
}

impl std::str::FromStr for MarkdownTableMode {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "off" => Ok(MarkdownTableMode::Off),
            "bullets" => Ok(MarkdownTableMode::Bullets),
            "code" => Ok(MarkdownTableMode::Code),
            "block" => Ok(MarkdownTableMode::Block),
            _ => Err(()),
        }
    }
}