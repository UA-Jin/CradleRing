// Embedding inputs helper.
// 翻译自 packages/memory-host-sdk/src/host/embedding-inputs.ts

pub fn chunk_embedding_inputs(text: &str, max_chars: usize) -> Vec<String> {
    let mut out = vec![];
    let mut current = String::new();
    for word in text.split_whitespace() {
        if current.len() + word.len() + 1 > max_chars {
            if !current.is_empty() {
                out.push(std::mem::take(&mut current));
            }
        }
        if !current.is_empty() {
            current.push(' ');
        }
        current.push_str(word);
    }
    if !current.is_empty() {
        out.push(current);
    }
    out
}