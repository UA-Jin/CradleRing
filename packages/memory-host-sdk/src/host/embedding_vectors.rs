// Embedding vectors helper.
// 翻译自 packages/memory-host-sdk/src/host/embedding-vectors.ts

pub fn normalize_vector(v: &mut Vec<f32>) {
    let norm: f32 = v.iter().map(|x| x * x).sum::<f32>().sqrt();
    if norm > 0.0 {
        for x in v.iter_mut() {
            *x /= norm;
        }
    }
}