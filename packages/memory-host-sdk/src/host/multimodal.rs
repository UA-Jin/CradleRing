// Multimodal helper.
// 翻译自 packages/memory-host-sdk/src/host/multimodal.ts

#[derive(Debug, Clone, Default)]
pub struct MultimodalInput {
    pub text: Option<String>,
    pub image_paths: Vec<String>,
}

pub fn encode_multimodal(_input: &MultimodalInput) -> Vec<u8> {
    vec![]
}