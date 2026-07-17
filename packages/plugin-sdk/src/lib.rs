// Public package facade for plugin SDK re-exports.
// 翻译自 packages/plugin-sdk/src/index.ts (implicit barrel)
//
// Each module under this crate is a 1:1 translation of the openclaw
// `packages/plugin-sdk/src/*.ts` re-export shims that forward to
// `../../../src/plugin-sdk/<name>.js`. The actual plugin SDK implementations
// (transcript / RSS / memory card / security verdict / etc.) live under
// `src/plugin-sdk/` in openclaw and are translated as they arrive in
// CradleRing's `src/` tree; for now each sub-module exposes a stub
// re-export surface so the package compiles in isolation.

pub mod browser_config;
pub mod config_runtime;
pub mod exec_approvals_runtime;
pub mod gateway_method_runtime;
pub mod outbound_media;
pub mod plugin_entry;
pub mod plugin_runtime;
pub mod provider_auth;
pub mod provider_auth_login_flow_runtime;
pub mod provider_auth_runtime;
pub mod provider_entry;
pub mod provider_http;
pub mod provider_model_shared;
pub mod provider_model_types;
pub mod provider_onboard;
pub mod provider_stream_shared;
pub mod provider_tools;
pub mod provider_web_search;
pub mod provider_web_search_config_contract;
pub mod runtime_doctor;
pub mod runtime_env;
pub mod secret_input;
pub mod security_runtime;
pub mod testing;
pub mod text_runtime;
pub mod video_generation;

pub use browser_config::*;
pub use config_runtime::*;
pub use exec_approvals_runtime::*;
pub use gateway_method_runtime::*;
pub use outbound_media::*;
pub use plugin_entry::*;
pub use plugin_runtime::*;
pub use provider_auth::*;
pub use provider_auth_login_flow_runtime::*;
pub use provider_auth_runtime::*;
pub use provider_entry::*;
pub use provider_http::*;
pub use provider_model_shared::*;
pub use provider_model_types::*;
pub use provider_onboard::*;
pub use provider_stream_shared::*;
pub use provider_tools::*;
pub use provider_web_search::*;
pub use provider_web_search_config_contract::*;
pub use runtime_doctor::*;
pub use runtime_env::*;
pub use secret_input::*;
pub use security_runtime::*;
pub use testing::*;
pub use text_runtime::*;
pub use video_generation::*;