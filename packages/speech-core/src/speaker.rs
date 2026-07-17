// Speech Core module implements speaker behavior.
// 1:1 port of openclaw-main/packages/speech-core/speaker.ts
// openclaw -> cradle-ring renames applied. Logic preserved line-by-line.

use serde_json::Value as JsonValue;

pub fn with_speaker_selection_compat(config: JsonValue) -> JsonValue {
    config
}
