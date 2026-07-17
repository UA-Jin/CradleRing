// Active media-understanding model selection contract.
// 翻译自 packages/media-understanding-common/src/active-model.ts

/** Provider/model pair selected for one media-understanding request. */
#[derive(Debug, Clone, Default)]
pub struct ActiveMediaModel {
    pub provider: String,
    pub model: Option<String>,
}