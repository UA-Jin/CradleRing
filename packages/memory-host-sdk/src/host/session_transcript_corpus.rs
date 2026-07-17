// Session transcript corpus helper.
// 翻译自 packages/memory-host-sdk/src/host/session-transcript-corpus.ts

pub fn session_transcript_corpus_path(_session_key: &str) -> std::path::PathBuf {
    std::path::PathBuf::from(".cradle-ring/transcripts")
}