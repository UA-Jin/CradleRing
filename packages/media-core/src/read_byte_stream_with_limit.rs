// 翻译自 packages/media-core/src/read-byte-stream-with-limit.ts

use std::sync::Arc;

/** Details passed to byte-stream overflow error factories. */
#[derive(Debug, Clone)]
pub struct ByteStreamLimitOverflow {
    pub size: usize,
    pub max_bytes: usize,
}

/** Options for reading an async byte stream under a hard byte cap. */
#[derive(Clone)]
pub struct ReadByteStreamWithLimitOptions {
    pub max_bytes: usize,
    pub on_overflow: Option<Arc<dyn Fn(ByteStreamLimitOverflow) -> String + Send + Sync>>,
}

impl std::fmt::Debug for ReadByteStreamWithLimitOptions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ReadByteStreamWithLimitOptions")
            .field("max_bytes", &self.max_bytes)
            .field("on_overflow", &self.on_overflow.as_ref().map(|_| "<fn>"))
            .finish()
    }
}

/// A byte stream chunk. The TypeScript version accepts `Buffer | string | ArrayBuffer | ArrayBufferView`.
/// In Rust we accept anything that can be turned into bytes via the [`ChunkBytes`] trait.
pub trait ChunkBytes: Clone {
    fn into_chunk_bytes(self) -> Vec<u8>;
}

impl ChunkBytes for Vec<u8> {
    fn into_chunk_bytes(self) -> Vec<u8> {
        self
    }
}

impl ChunkBytes for String {
    fn into_chunk_bytes(self) -> Vec<u8> {
        self.into_bytes()
    }
}

impl ChunkBytes for &str {
    fn into_chunk_bytes(self) -> Vec<u8> {
        self.as_bytes().to_vec()
    }
}

impl ChunkBytes for &[u8] {
    fn into_chunk_bytes(self) -> Vec<u8> {
        self.to_vec()
    }
}

impl<const N: usize> ChunkBytes for &[u8; N] {
    fn into_chunk_bytes(self) -> Vec<u8> {
        self.to_vec()
    }
}

fn destroy_readable_on_overflow<C>(_stream: &[C], _err: &str) {
    // The TypeScript version calls stream.destroy(err) or stream.cancel(err).
    // In our generic Rust version the caller controls the stream lifecycle,
    // so we simply no-op here to preserve 1:1 shape with the surrounding logic.
}

/** Reads and concatenates an async byte stream, throwing once the byte cap is exceeded. */
pub async fn read_byte_stream_with_limit<C, S>(
    stream: S,
    opts: ReadByteStreamWithLimitOptions,
) -> Result<Vec<u8>, String>
where
    S: AsRef<[C]>,
    C: ChunkBytes,
{
    let max_bytes = opts.max_bytes;
    let on_overflow: Arc<dyn Fn(ByteStreamLimitOverflow) -> String + Send + Sync> =
        opts.on_overflow.unwrap_or_else(|| {
            Arc::new(|params: ByteStreamLimitOverflow| {
                format!(
                    "Content too large: {} bytes (limit: {} bytes)",
                    params.size, params.max_bytes
                )
            })
        });
    let stream_slice = stream.as_ref();
    let mut chunks: Vec<Vec<u8>> = Vec::new();
    let mut total: usize = 0;
    for item in stream_slice {
        let buffer = item.clone().into_chunk_bytes();
        if buffer.is_empty() {
            continue;
        }
        let next_total = total + buffer.len();
        if next_total > max_bytes {
            let err = on_overflow(ByteStreamLimitOverflow {
                size: next_total,
                max_bytes,
            });
            destroy_readable_on_overflow(stream_slice, &err);
            return Err(err);
        }
        chunks.push(buffer);
        total = next_total;
    }

    let mut out = Vec::with_capacity(total);
    for c in chunks {
        out.extend(c);
    }
    Ok(out)
}