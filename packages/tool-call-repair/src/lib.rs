// Tool Call Repair module - public surface.
// 1:1 port of openclaw-main/packages/tool-call-repair/src/index.ts
// openclaw -> cradle-ring renames applied.

pub mod grammar;
pub mod payload;
pub mod promote;
pub mod stream_normalizer;

pub use payload::{
    parse_standalone_plain_text_tool_call_blocks, strip_plain_text_tool_call_blocks,
    PlainTextToolCallBlock, PlainTextToolCallParseOptions, PlainTextToolCallNameMatcher,
};
pub use stream_normalizer::{
    normalize_plain_text_tool_call_stream_events, project_scrubbed_plain_text_tool_call_message,
    PlainTextToolCallMessageNormalization, PlainTextToolCallStreamNormalizerOptions,
};
pub use promote::{
    create_promoted_plain_text_tool_call_block, create_promoted_plain_text_tool_call_events,
    project_standalone_plain_text_tool_call_message, PlainTextToolCallMessageProjection,
    PlainTextToolCallPromotionOptions, PromotedPlainTextToolCallBlockFactory,
    ToolCallRepairNameResolver,
};
