// Decodes a byte prefix without inventing a replacement character for a cut trailing sequence.
// 翻译自 packages/normalization-core/src/text-decoding.ts

/// DecodeTextPrefixOptions
#[derive(Debug, Clone, Default)]
pub struct DecodeTextPrefixOptions {
    pub encoding: Option<String>,
    pub truncated: bool,
}

/// Decodes a byte prefix without inventing a replacement character for a cut trailing sequence.
///
/// Rust 的 String::from_utf8 会拒绝无效序列；truncated 模式下我们丢弃尾部不完整序列。
pub fn decode_text_prefix(bytes: &[u8], options: DecodeTextPrefixOptions) -> String {
    // Rust 原生 UTF-8；TextDecoder 默认也是 utf-8
    if options.truncated {
        // streaming mode: 丢弃尾部不完整序列
        // 找到最后一个有效的 UTF-8 边界
        let mut valid_len = bytes.len();
        while valid_len > 0 {
            if std::str::from_utf8(&bytes[..valid_len]).is_ok() {
                break;
            }
            valid_len -= 1;
        }
        std::str::from_utf8(&bytes[..valid_len])
            .unwrap_or("")
            .to_string()
    } else {
        // one-shot: 用 lossy 解码（替换无效序列）
        String::from_utf8_lossy(bytes).to_string()
    }
}
